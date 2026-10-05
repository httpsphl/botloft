//! Starting a bot process (spec 7.4) and following it until it exits.

use std::ffi::OsString;
use std::io;
use std::sync::{Arc, Weak};

use botloft_core::ids::{BotId, random_uuid};
use botloft_core::protocol::{BotState, Crew};
use botloft_store::BotRecord;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::debug;

use super::slot::StopIntent;
use super::{Inner, STABLE_AFTER, Supervisor};
use crate::chat::{LIVE_TEXT_EVERY, StreamReader};
use crate::platform;
use crate::runtime::claude::Claude;
use crate::runtime::{ProcessEvent, SpawnSpec};
use crate::secrets::{self, TokenHash};
use crate::state::Daemon;
use crate::{approvals, context, courier, mcp_secrets, routines, workspace};

/// Tools the bot uses without asking: its crew tools (spec 7.4).
const ALLOWED_TOOLS: &str = "mcp__botloft";
/// Claude Code's own schedulers: they die with the process and Botloft never
/// sees them. Work at set times is a routine (spec 7.4, 20).
const DISALLOWED_TOOLS: &str = "CronCreate,CronDelete,CronList,ScheduleWakeup,RemoteTrigger";
/// Makes Claude Code load `CLAUDE.md` from `--add-dir` folders (spec 5).
const ADDITIONAL_MEMORY: &str = "CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD";
/// Where permission requests go (spec 10.1).
const PERMISSION_TOOL: &str = "mcp__botloft__permission_prompt";

impl Supervisor {
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
        slot.clear_work();
        slot.limited_until = None;
        let ran = running.started.elapsed();
        let resumed = running.resumed;
        // Closing the job kills anything the bot left behind.
        drop(running);
        debug!(bot = %bot, generation, ?code, ran_ms = ran.as_millis() as u64, "bot exited");

        match slot.stop.take() {
            Some(StopIntent::Restart { fresh }) => {
                slot.fresh_next |= fresh;
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
        context::process_ended(daemon, bot);
        crate::screens::turn_ended(daemon, bot);
        daemon.crew_access.end_turn(bot);
        self.wake();
    }
}

pub(super) struct Launch {
    pub token_hash: String,
    pub resumed: bool,
}

/// Command line and environment of spec 7.4, with a fresh bot token.
/// `session` is the conversation to resume; `None` starts a new one.
pub(super) fn launch_spec(
    daemon: &Daemon,
    crew: &Crew,
    bot: &BotRecord,
    claude: &Claude,
    session: Option<&str>,
) -> io::Result<(SpawnSpec, Launch)> {
    // settings.json and the rules are rewritten on every start.
    let crews = daemon.store().crews(true).map_err(io::Error::other)?;
    let servers = daemon
        .store()
        .bot_mcp_servers(&bot.id)
        .map_err(io::Error::other)?;
    let workspace = workspace::prepare_bot(daemon.workspace_env(), crew, &crews, bot, &servers)?;
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
    args.push(session.into());
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
        "--disallowedTools",
        DISALLOWED_TOOLS,
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
    // Without the flag, Claude Code uses the level it sets for the model.
    if let Some(effort) = bot.effort.cli_value() {
        args.push("--effort".into());
        args.push(effort.into());
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

    // The values of the headers and variables of the bot's connected tools,
    // which its `mcp.json` expands (spec 25.2).
    env.extend(mcp_secrets::environment(&daemon.paths.secrets(), &servers)?);

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
        },
    ))
}

/// Claude Code says nothing until the first message, so a process that
/// is still alive after a moment counts as ready (spec 7.2).
pub(super) async fn settle(
    daemon: Weak<Daemon>,
    bot: BotId,
    generation: u64,
    after: std::time::Duration,
) {
    tokio::time::sleep(after).await;
    if let Some(daemon) = daemon.upgrade() {
        daemon.supervisor.ready(&bot, generation);
    }
}

/// Feeds stdout to the chat until the process exits, then reports it.
pub(super) async fn pump(
    daemon: Weak<Daemon>,
    mut reader: StreamReader,
    mut events: mpsc::Receiver<ProcessEvent>,
) {
    // When the live text gathered so far goes out (spec 8.3).
    let mut flush_at: Option<Instant> = None;
    let code = loop {
        let event = tokio::select! {
            event = events.recv() => event,
            () = tokio::time::sleep_until(flush_at.unwrap_or_else(Instant::now)),
                if flush_at.is_some() =>
            {
                flush_at = None;
                if let Some(daemon) = daemon.upgrade() {
                    reader.flush(&daemon);
                }
                continue;
            }
        };
        match event {
            Some(ProcessEvent::Output(data)) => {
                let Some(daemon) = daemon.upgrade() else {
                    return;
                };
                reader.feed(&daemon, &data);
                if !reader.holds_live_text() {
                    flush_at = None;
                } else if flush_at.is_none() {
                    flush_at = Some(Instant::now() + LIVE_TEXT_EVERY);
                }
            }
            Some(ProcessEvent::Exited(code)) => break code,
            None => break None,
        }
    };
    if let Some(daemon) = daemon.upgrade() {
        reader.flush(&daemon);
        daemon
            .supervisor
            .on_exit(&daemon, reader.bot(), reader.generation(), code);
    }
}
