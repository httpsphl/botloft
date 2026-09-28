//! Starting a bot process (spec 7.4) and following it until it exits.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotState, Crew};
use botloft_store::BotRecord;
use bytes::BytesMut;
use tokio::sync::mpsc;
use tokio::time::Instant;
use tracing::{debug, warn};

use super::slot::{Running, StopIntent};
use super::{ClaudeStatus, Inner, STABLE_AFTER, Supervisor};
use crate::platform;
use crate::runtime::claude::Claude;
use crate::runtime::{ProcessEvent, SpawnSpec};
use crate::secrets::{self, TokenHash};
use crate::state::Daemon;
use crate::terminal::{CursorQueryWatch, Terminal};
use crate::workspace;

/// Output read within this window goes out as one chunk (spec 8).
const COALESCE_WINDOW: Duration = Duration::from_millis(8);
const COALESCE_MAX: usize = 32 * 1024;
/// Written after the first `SessionStart`; without it there is no
/// conversation to `--continue`.
const STARTED_MARKER: &str = ".botloft/started";

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
            claude,
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
        let launched = launch_spec(daemon, crew, bot, claude, slot.fresh_next, slot.size)
            .and_then(|(spec, launch)| Ok((self.runtime.spawn(spec)?, launch)));
        match launched {
            Ok((process, launch)) => {
                slot.generation = Some(generation);
                slot.terminal.reset(generation);
                slot.restart_at = None;
                slot.fresh_next = false;
                slot.inbox = None;
                tokens.insert(launch.token_hash.clone(), (bot.id.clone(), generation));
                slot.running = Some(Running {
                    control: process.control,
                    started: Instant::now(),
                    resumed: launch.resumed,
                    token_hash: launch.token_hash,
                    workspace: launch.workspace,
                });
                slot.state = BotState::Launching;
                self.announce(&bot.id, slot);
                debug!(bot = %bot.id, generation, pid = ?process.pid, resumed = launch.resumed, "bot started");
                tokio::spawn(pump(
                    self.daemon.clone(),
                    bot.id.clone(),
                    generation,
                    Arc::clone(&slot.terminal),
                    process.events,
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
    /// daemon stopped it on purpose (spec 7.3).
    pub(super) fn on_exit(&self, bot: &BotId, generation: u64, code: Option<u32>) {
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
        slot.inbox = None;
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
            // Restarting cannot fix credentials; the owner restarts it.
            None if slot.state == BotState::AuthError => slot.restart_at = None,
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
        self.wake();
    }
}

struct Launch {
    token_hash: String,
    resumed: bool,
    workspace: PathBuf,
}

/// Command line and environment of spec 7.4, with a fresh bot token.
fn launch_spec(
    daemon: &Daemon,
    crew: &Crew,
    bot: &BotRecord,
    claude: &Claude,
    fresh: bool,
    size: crate::runtime::TermSize,
) -> io::Result<(SpawnSpec, Launch)> {
    // settings.json and the rules are rewritten on every start.
    let workspace = workspace::prepare_bot(daemon.workspace_env(), crew, bot)?;
    let resumed = !fresh && workspace.join(STARTED_MARKER).exists();
    let token = secrets::random_token()?;

    let mut args: Vec<OsString> = Vec::new();
    if resumed {
        args.push("--continue".into());
    }
    args.push("--settings".into());
    args.push(workspace.join(".claude").join("settings.json").into());
    args.push("--mcp-config".into());
    args.push(workspace.join(".botloft").join("mcp.json").into());

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
        ("BOTLOFT_BIN", daemon.bin.clone().into_os_string()),
        ("BOTLOFT_HOME", daemon.paths.home.clone().into_os_string()),
    ] {
        env.push((OsString::from(name), value));
    }

    let spec = SpawnSpec {
        program: claude.path.clone(),
        args,
        cwd: workspace.clone(),
        env,
        size,
    };
    Ok((
        spec,
        Launch {
            token_hash: TokenHash::of(&token).to_hex(),
            resumed,
            workspace,
        },
    ))
}

pub(super) fn mark_started(workspace: &Path) {
    let marker = workspace.join(STARTED_MARKER);
    if !marker.exists()
        && let Err(err) = std::fs::write(&marker, b"")
    {
        warn!("could not write {}: {err}", marker.display());
    }
}

/// Moves output into the terminal until the process exits, then reports it.
async fn pump(
    daemon: Weak<Daemon>,
    bot: BotId,
    generation: u64,
    terminal: Arc<Terminal>,
    mut events: mpsc::Receiver<ProcessEvent>,
) {
    let mut cursor_query = CursorQueryWatch::default();
    let code = coalesce(&mut events, |data| {
        terminal.push(generation, data);
        if cursor_query.feed(data)
            && let Some(daemon) = daemon.upgrade()
        {
            daemon
                .supervisor
                .reply(&bot, generation, CursorQueryWatch::ANSWER);
        }
    })
    .await;
    if let Some(daemon) = daemon.upgrade() {
        daemon.supervisor.on_exit(&bot, generation, code);
    }
}

/// Joins output arriving within [`COALESCE_WINDOW`] (up to
/// [`COALESCE_MAX`]) and hands each batch to `sink`. Returns the exit code.
pub(crate) async fn coalesce(
    events: &mut mpsc::Receiver<ProcessEvent>,
    mut sink: impl FnMut(&[u8]),
) -> Option<u32> {
    loop {
        let mut batch = match events.recv().await {
            Some(ProcessEvent::Output(data)) => BytesMut::from(&data[..]),
            Some(ProcessEvent::Exited(code)) => return code,
            None => return None,
        };
        let deadline = Instant::now() + COALESCE_WINDOW;
        let mut end = None;
        while batch.len() < COALESCE_MAX {
            match tokio::time::timeout_at(deadline, events.recv()).await {
                Ok(Some(ProcessEvent::Output(data))) => batch.extend_from_slice(&data),
                Ok(Some(ProcessEvent::Exited(code))) => {
                    end = Some(code);
                    break;
                }
                Ok(None) => {
                    end = Some(None);
                    break;
                }
                Err(_) => break,
            }
        }
        sink(&batch);
        if let Some(code) = end {
            return code;
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[tokio::test(start_paused = true)]
    async fn output_within_the_window_is_one_chunk() {
        let (tx, mut rx) = mpsc::channel(8);
        tx.send(ProcessEvent::Output(Bytes::from_static(b"ab")))
            .await
            .expect("send");
        tx.send(ProcessEvent::Output(Bytes::from_static(b"cd")))
            .await
            .expect("send");
        let late = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            tx.send(ProcessEvent::Output(Bytes::from_static(b"ef")))
                .await
                .expect("send");
            tx.send(ProcessEvent::Exited(Some(3))).await.expect("send");
        });
        let mut chunks = Vec::new();
        let code = coalesce(&mut rx, |data| chunks.push(data.to_vec())).await;
        late.await.expect("sender");
        assert_eq!(code, Some(3));
        assert_eq!(chunks, vec![b"abcd".to_vec(), b"ef".to_vec()]);
    }
}
