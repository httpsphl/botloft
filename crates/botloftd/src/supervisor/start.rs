//! Starting a bot in three steps, so the supervisor lock is never held
//! while its files are written and its process is created (spec 7.4):
//! what to start is decided under the lock, the launch runs without it,
//! and the new process is recorded under the lock again.

use std::io;
use std::sync::Arc;

use botloft_core::protocol::{BotState, Crew};
use botloft_store::BotRecord;
use tokio::time::Instant;
use tracing::{debug, warn};

use super::slot::Running;
use super::spawn::{Launch, launch_spec, pump, settle};
use super::{ClaudeStatus, Inner, Supervisor};
use crate::chat::{StreamReader, control};
use crate::context;
use crate::runtime::Process;
use crate::runtime::claude::Claude;
use crate::state::Daemon;

/// A bot the reconcile pass decided to start.
pub(super) struct Start {
    crew: Crew,
    bot: BotRecord,
    claude: Claude,
    /// The conversation to resume; `None` starts a new one.
    session: Option<String>,
}

impl Supervisor {
    /// Decides to start `bot`, or marks it offline when Claude Code is not
    /// usable. The slot stays `launching` until [`Self::launch`] ends.
    pub(super) fn plan_start(
        &self,
        inner: &mut Inner,
        crew: &Crew,
        bot: &BotRecord,
    ) -> Option<Start> {
        let Inner {
            slots,
            sessions,
            claude,
            ..
        } = inner;
        let slot = slots.get_mut(&bot.id)?;
        let ClaudeStatus::Ready(claude) = claude else {
            self.set_state(&bot.id, slot, BotState::Offline);
            return None;
        };
        slot.launching = true;
        // This start reads the bot's launch settings.
        slot.restart_when_idle = false;
        let session = if slot.fresh_next {
            None
        } else {
            sessions.get(&bot.id).cloned()
        };
        Some(Start {
            crew: crew.clone(),
            bot: bot.clone(),
            claude: claude.clone(),
            session,
        })
    }

    /// Writes the bot's files and creates its process without the
    /// supervisor lock, then records the process.
    pub(super) fn launch(&self, daemon: &Arc<Daemon>, start: Start) {
        let launched = launch_spec(
            daemon,
            &start.crew,
            &start.bot,
            &start.claude,
            start.session.as_deref(),
        )
        .and_then(|(spec, launch)| Ok((self.runtime.spawn(spec)?, launch)));
        let mut inner = self.lock();
        self.commit(&mut inner, daemon, &start.bot, launched);
    }

    fn commit(
        &self,
        inner: &mut Inner,
        daemon: &Arc<Daemon>,
        bot: &BotRecord,
        launched: io::Result<(Process, Launch)>,
    ) {
        let Inner {
            slots,
            tokens,
            sessions,
            replaced,
            gone,
            ..
        } = inner;
        // Deleted while it started: the process goes with it.
        let slot = match slots.get_mut(&bot.id) {
            Some(slot) if !gone.contains(&bot.id) => slot,
            _ => {
                if let Ok((process, _)) = launched
                    && let Err(err) = process.control.kill()
                {
                    warn!(bot = %bot.id, "could not stop a deleted bot: {err}");
                }
                return;
            }
        };
        slot.launching = false;
        let (process, launch) = match launched {
            Ok(launched) => launched,
            Err(err) => {
                warn!(bot = %bot.id, "could not start the bot: {err}");
                slot.restart_at = Some(Instant::now() + slot.backoff.next_delay());
                self.set_state(&bot.id, slot, BotState::Backoff);
                return;
            }
        };
        let generation = self
            .next_generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // What the session applies and how full it is (spec 9.2); Claude
        // Code answers as soon as it is up.
        for ask in [control::settings_request(), control::context_request()] {
            let _ = process.control.write(ask);
        }
        // Each connected tool says whether it came up (spec 25.5).
        crate::service::mcp_state::forget(daemon, &bot.id);
        if launch.connected {
            let _ = process.control.write(control::mcp_request());
        }
        context::process_started(daemon, &bot.id, launch.resumed);
        // A new conversation is on disk only once it has a turn (spec
        // 7.3); the one it takes the place of is left behind.
        if !launch.resumed
            && let Some(old) = sessions.remove(&bot.id)
        {
            replaced.push((bot.id.clone(), old));
        }
        slot.generation = Some(generation);
        slot.restart_at = None;
        slot.fresh_next = false;
        slot.clear_work();
        slot.limited_until = None;
        tokens.insert(launch.token_hash.clone(), (bot.id.clone(), generation));
        slot.running = Some(Running {
            control: process.control,
            started: Instant::now(),
            resumed: launch.resumed,
            token_hash: launch.token_hash,
            permission_mode: bot.permission_mode,
            effort: bot.effort,
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
        // A launch setting changed while it started (spec 7.4).
        self.relaunch_if_idle(&bot.id, slot);
    }
}
