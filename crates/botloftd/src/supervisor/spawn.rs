//! Starting a bot process (spec 7.4) and following it until it exits.

use std::ffi::OsString;
use std::io;
use std::sync::{Arc, Weak};

use botloft_core::ids::{BotId, random_uuid};
use botloft_core::protocol::{BotState, Crew};
use botloft_store::BotRecord;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{debug, warn};

use super::slot::{Running, StopIntent};
use super::{ClaudeStatus, Inner, STABLE_AFTER, Supervisor};
use crate::chat::StreamReader;
use crate::platform;
use crate::runtime::claude::Claude;
use crate::runtime::{ProcessEvent, SpawnSpec};
use crate::secrets::{self, TokenHash};
use crate::state::Daemon;
use crate::{approvals, courier, routines, workspace};

/// Tools the bot uses without asking: its crew tools (spec 7.4).
const ALLOWED_TOOLS: &str = "mcp__botloft";
/// Makes Claude Code load `CLAUDE.md` from `--add-dir` folders (spec 5).
const ADDITIONAL_MEMORY: &str = "CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD";
/// Where permission requests go (spec 10.1).
const PERMISSION_TOOL: &str = "mcp__botloft__permission_prompt";

impl Supervisor {
    pub(super) fn start(
        &self,
        inner: &mut Inner,
        daemon: &Arc<Daemon>,
        crew: &Crew,
        bot: &BotRecord,
    ) {
        let Inner {
            slots,
            tokens,
            sessions,
            claude,
            ..
        } = inner;
        let Some(slot) = slots.get_mut(&bot.id) else {
            return;
        };
        let ClaudeStatus::Ready(claude) = claude else {
            self.set_state(&bot.id, slot, BotState::Offline);
            return;
        };
        let generation = self
            .next_generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let session = if slot.fresh_next {
            None
        } else {
            sessions.get(&bot.id).cloned()
        };
        let launched = launch_spec(daemon, crew, bot, claude, session.as_deref())
            .and_then(|(spec, launch)| Ok((self.runtime.spawn(spec)?, launch)));
        match launched {
            Ok((process, launch)) => {
                sessions.insert(bot.id.clone(), launch.session.clone());
                slot.generation = Some(generation);
                slot.restart_at = None;
                slot.fresh_next = false;
                slot.turns = 0;
                slot.approvals = 0;
                slot.limited_until = None;
                slot.restart_when_idle = false;
                tokens.insert(launch.token_hash.clone(), (bot.id.clone(), generation));
                slot.running = Some(Running {
                    control: process.control,
                    started: Instant::now(),
                    resumed: launch.resumed,
                    token_hash: launch.token_hash,
                    permission_mode: bot.permission_mode,
                });
                self.count_busy(slot.state, BotState::Launching);
                slot.state = BotState::Launching;
                self.announce(&bot.id, slot);
                debug!(bot = %bot.id, generation, pid = ?process.pid, resumed = launch.resumed, "bot started");
                let reader = StreamReader::new(bot.id.clone(), generation);
                tokio::spawn(pump(self.daemon.clone(), reader, process.events));
                tokio::spawn(settle(
                    self.daemon.clone(),
                    bot.id.clone(),
                    generation,
                    self.settings.ready_after,
                ));
            }
            Err(err) => {
                warn!(bot = %bot.id, "could not start the bot: {err}");
                slot.restart_at = Some(Instant::now() + slot.backoff.next_delay());
                self.set_state(&bot.id, slot, BotState::Backoff);
            }
        }
    }

    /// The process of `generation` ended: restart it, or settle if the
    /// daemon stopped it on purpose (spec 7.3). What it had not begun goes
    /// back in the queue, and open approvals expire.
    pub(super) fn on_exit(
        &self,
        daemon: &Arc<Daemon>,
        bot: &BotId,
        generation: u64,
        code: Option<u32>,
    ) {
        let mut inner = self.lock();
        let Inner { slots, tokens, .. } = &mut *inner;
        let Some(slot) = slots.get_mut(bot) else {
            return;
        };
        if slot.generation != Some(generation) {
            return;
        }
        let Some(running) = slot.running.take() else {
            return;
        };
        tokens.remove(&running.token_hash);
        slot.turns = 0;
        slot.approvals = 0;
        slot.limited_until = None;
        let ran = running.started.elapsed();
        let resumed = running.resumed;
        // Closing the job kills anything the bot left behind.
        drop(running);
        debug!(bot = %bot, generation, ?code, ran_ms = ran.as_millis() as u64, "bot exited");

        match slot.stop.take() {
            Some(StopIntent::Restart { fresh }) => {
                slot.fresh_next = fresh;
                slot.restart_at = Some(Instant::now());
            }
            Some(StopIntent::Halt(state)) => {
                slot.restart_at = None;
                self.set_state(bot, slot, state);
            }
            None => {
                if resumed && ran < self.settings.fresh_start_if_dies_within {
                    slot.fresh_next = true;
                }
                if ran >= STABLE_AFTER {
                    slot.backoff.reset();
                }
                slot.restart_at = Some(Instant::now() + slot.backoff.next_delay());
                self.set_state(bot, slot, BotState::Backoff);
            }
        }
        drop(inner);
        approvals::expire_for_bot(daemon, bot);
        courier::requeue_unread(daemon, bot, generation);
        routines::process_ended(daemon, bot);
        self.wake();
    }
}

struct Launch {
    token_hash: String,
    resumed: bool,
    session: String,
}

/// Command line and environment of spec 7.4, with a fresh bot token.
/// `session` is the conversation to resume; `None` starts a new one.
fn launch_spec(
    daemon: &Daemon,
    crew: &Crew,
    bot: &BotRecord,
    claude: &Claude,
    session: Option<&str>,
) -> io::Result<(SpawnSpec, Launch)> {
    // settings.json and the rules are rewritten on every start.
    let workspace = workspace::prepare_bot(daemon.workspace_env(), crew, bot)?;
    let token = secrets::random_token()?;
    let (session, resumed) = match session {
        Some(session) => (session.to_owned(), true),
        None => (random_uuid(), false),
    };

    let mcp = workspace.join(".botloft").join("mcp.json");
    let mut args: Vec<OsString> = [
        "-p",
        "--input-format",
        "stream-json",
        "--output-format",
        "stream-json",
        "--verbose",
        "--include-partial-messages",
        "--replay-user-messages",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    args.push(if resumed { "--resume" } else { "--session-id" }.into());
    args.push(session.clone().into());
    for arg in [
        "--setting-sources",
        "project,local",
        "--strict-mcp-config",
        "--permission-mode",
        bot.permission_mode.cli_value(),
        "--permission-prompt-tool",
        PERMISSION_TOOL,
        "--allowedTools",
        ALLOWED_TOOLS,
        "--mcp-config",
    ] {
        args.push(arg.into());
    }
    args.push(mcp.into_os_string());
    // The crew's work folder, which the bot edits like its own (spec 5).
    args.push("--add-dir".into());
    args.push(daemon.paths.work_folder(crew).into_os_string());
    // Without the flag, Claude Code uses the default of the owner's plan.
    if let Some(model) = bot.model.cli_value() {
        args.push("--model".into());
        args.push(model.into());
    }

    let mut env = platform::user_environment()?;
    env.retain(|(name, _)| {
        !name
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("BOTLOFT_")
    });
    for (name, value) in [
        ("BOTLOFT_BOT_ID", OsString::from(bot.id.as_str())),
        ("BOTLOFT_BOT_TOKEN", OsString::from(&token)),
        ("BOTLOFT_PORT", OsString::from(daemon.port.to_string())),
        // Loads the work folder's CLAUDE.md with the bot's memory.
        (ADDITIONAL_MEMORY, OsString::from("1")),
    ] {
        env.push((OsString::from(name), value));
    }

    let spec = SpawnSpec {
        program: claude.path.clone(),
        args,
        cwd: workspace.clone(),
        env,
    };
    Ok((
        spec,
        Launch {
            token_hash: TokenHash::of(&token).to_hex(),
            resumed,
            session,
        },
    ))
}

/// Claude Code says nothing until the first message, so a process that
/// is still alive after a moment counts as ready (spec 7.2).
async fn settle(daemon: Weak<Daemon>, bot: BotId, generation: u64, after: std::time::Duration) {
    tokio::time::sleep(after).await;
    if let Some(daemon) = daemon.upgrade() {
        daemon.supervisor.ready(&bot, generation);
    }
}

/// Feeds stdout to the chat until the process exits, then reports it.
async fn pump(
    daemon: Weak<Daemon>,
    mut reader: StreamReader,
    mut events: mpsc::Receiver<ProcessEvent>,
) {
    let code = loop {
        match events.recv().await {
            Some(ProcessEvent::Output(data)) => {
                let Some(daemon) = daemon.upgrade() else {
                    return;
                };
                reader.feed(&daemon, &data);
            }
            Some(ProcessEvent::Exited(code)) => break code,
            None => break None,
        }
    };
    if let Some(daemon) = daemon.upgrade() {
        daemon
            .supervisor
            .on_exit(&daemon, reader.bot(), reader.generation(), code);
    }
}
