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
use crate::agent::{self, AttachInput, LaunchFiles, LaunchPlan};
use crate::chat::{LIVE_TEXT_EVERY, StreamReader};
use crate::platform;
use crate::runtime::claude::Claude;
use crate::runtime::{ProcessEvent, SpawnSpec};
use crate::secrets::{self, TokenHash};
use crate::state::Daemon;
use crate::{approvals, context, courier, mcp_secrets, routines, workspace};

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
    /// What the agent needs once the process exists.
    pub attach: AttachInput,
    pub token_hash: String,
    pub resumed: bool,
    /// Whether the bot starts with connected tools (spec 25).
    pub connected: bool,
}

/// Command line and environment of spec 7.4, with a fresh bot token.
/// `session` is the conversation to resume; `None` starts a new one.
pub(super) fn launch_spec(
    daemon: &Daemon,
    crew: &Crew,
    bot: &BotRecord,
    claude: Option<&Claude>,
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

    let agent = agent::of(bot.agent).ok_or_else(|| {
        io::Error::other(format!("bots on {} cannot start yet", bot.agent.as_str()))
    })?;
    let program = agent.locate(
        daemon.supervisor.program_setting(bot.agent),
        claude.map(|claude| claude.path.as_path()),
    )?;
    let mcp = workspace.join(".botloft").join("mcp.json");
    let work_folder = daemon.paths.work_folder(crew);
    let args = agent.args(&LaunchPlan {
        session: &session,
        resumed,
        mcp_config: &mcp,
        work_folder: &work_folder,
        permission_mode: bot.permission_mode,
        model: bot.model,
        effort: bot.effort,
        agent_model: bot.agent_model.as_deref(),
    });

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
    ] {
        env.push((OsString::from(name), value));
    }
    // What the agent sets replaces what the user environment has.
    for (name, value) in agent.extra_env(&workspace) {
        env.retain(|(have, _)| !have.eq_ignore_ascii_case(&name));
        env.push((name, value));
    }
    let mut fenced = workspace::fences(&daemon.paths, crew, &crews);
    fenced.push(daemon.paths.secrets());
    agent.write_launch_files(&LaunchFiles {
        workspace: &workspace,
        port: daemon.port,
        token: &token,
        fenced: &fenced,
        allowed_commands: &bot.allowed_commands,
    })?;

    // The values of the headers and variables of the bot's connected tools,
    // which its `mcp.json` expands (spec 25.2).
    env.extend(mcp_secrets::environment(&daemon.paths.secrets(), &servers)?);

    // The connected tools in the agent's own configuration, secrets in place.
    let connected = workspace::connected::codex_entries(&servers, &env);
    let spec = SpawnSpec {
        program,
        args,
        cwd: workspace.clone(),
        env,
    };
    Ok((
        spec,
        Launch {
            attach: AttachInput {
                workspace: workspace.clone(),
                port: daemon.port,
                token: token.clone(),
                resume: resumed.then(|| session.clone()),
                model: bot.agent_model.clone(),
                effort: bot.effort.cli_value().map(str::to_owned),
                fenced: fenced.clone(),
                may_ask: bot.permission_mode != botloft_core::protocol::PermissionMode::Plan,
                connected,
            },
            token_hash: TokenHash::of(&token).to_hex(),
            resumed,
            connected: !servers.is_empty(),
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
