//! Comparing what runs with what the database says should run.

use std::collections::HashMap;
use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotState, Crew};
use botloft_store::BotRecord;
use tokio::time::Instant;
use tracing::warn;

use super::slot::{Backoff, Slot, StopIntent};
use super::{ClaudeSource, ClaudeStatus, Inner, REPROBE_AFTER, Supervisor, SupervisorSettings};
use crate::platform;
use crate::runtime::claude::{self, Claude, ClaudeError};
use crate::state::Daemon;

impl Supervisor {
    /// Starts bots that should run and stops those that should not.
    pub async fn reconcile(&self) {
        let _one_at_a_time = self.reconcile_lock.lock().await;
        let Some(daemon) = self.daemon.upgrade() else {
            return;
        };
        self.ensure_claude().await;
        self.ensure_sign_in().await;
        // Read before taking the supervisor lock (lock order, see mod.rs).
        let read = {
            let store = daemon.store();
            store.bots(None, true).and_then(|bots| {
                let crews = store.crews(true)?;
                let mut sessions = HashMap::new();
                for bot in &bots {
                    if let Some(session) = store.session_id(&bot.id)? {
                        sessions.insert(bot.id.clone(), session);
                    }
                }
                Ok((bots, crews, sessions))
            })
        };
        let (bots, crews, sessions) = match read {
            Ok(read) => read,
            Err(err) => {
                warn!("reconcile could not read the database: {err}");
                return;
            }
        };
        let crews: HashMap<_, _> = crews
            .into_iter()
            .map(|crew| (crew.id.clone(), crew))
            .collect();
        self.release_limits(daemon.clock.now_ms());
        let now = Instant::now();
        let mut inner = self.lock();
        for (bot, session) in sessions {
            inner.sessions.entry(bot).or_insert(session);
        }
        for bot in &bots {
            if let Some(crew) = crews.get(&bot.crew_id) {
                self.reconcile_bot(&mut inner, &daemon, crew, bot, now);
            }
        }
    }

    fn reconcile_bot(
        &self,
        inner: &mut Inner,
        daemon: &Arc<Daemon>,
        crew: &Crew,
        bot: &BotRecord,
        now: Instant,
    ) {
        let archived = bot.archived_at.is_some() || crew.archived_at.is_some();
        if archived && !inner.slots.contains_key(&bot.id) {
            return;
        }
        let wanted = !archived && !bot.paused && !crew.paused;
        let slot = slot_entry(&mut inner.slots, &bot.id, &self.settings);
        if !wanted {
            let target = if archived {
                BotState::Archived
            } else {
                BotState::Offline
            };
            slot.restart_at = None;
            match &slot.running {
                Some(running) if slot.stop.is_none() => {
                    slot.stop = Some(StopIntent::Halt(target));
                    if let Err(err) = running.control.kill() {
                        warn!(bot = %bot.id, "could not stop the bot: {err}");
                    }
                }
                Some(_) => {}
                None => self.set_state(&bot.id, slot, target),
            }
            return;
        }
        // A process on its way out settles first; its exit wakes the loop.
        let waiting = slot.restart_at.is_some_and(|at| at > now);
        if slot.running.is_some() || slot.stop.is_some() || waiting {
            return;
        }
        if slot.state == BotState::AuthError {
            return;
        }
        self.start(inner, daemon, crew, bot);
    }

    /// Earliest scheduled restart, so the run loop can wake for it.
    pub(super) fn next_restart(&self) -> Option<Instant> {
        let inner = self.lock();
        inner
            .slots
            .values()
            .filter(|slot| slot.running.is_none())
            .filter_map(|slot| slot.restart_at)
            .min()
    }

    /// Finds and checks Claude Code when it is unknown or failed a while ago.
    async fn ensure_claude(&self) {
        let (due, previous) = match &self.lock().claude {
            ClaudeStatus::Unknown => (true, None),
            ClaudeStatus::Ready(_) => (false, None),
            ClaudeStatus::Failed { error, at } => {
                (at.elapsed() >= REPROBE_AFTER, Some(error.clone()))
            }
        };
        if !due {
            return;
        }
        let source = self.settings.claude.clone();
        let probed = match source {
            ClaudeSource::Fixed(claude) => Ok(Ok(claude)),
            // Runs `claude --version`; keep it off the async workers.
            source @ ClaudeSource::Discover { .. } => {
                tokio::task::spawn_blocking(move || probe(&source)).await
            }
        };
        let status = match probed {
            Ok(Ok(claude)) => ClaudeStatus::Ready(claude),
            Ok(Err(err)) => failed(err.to_string(), previous.as_deref()),
            Err(err) => failed(
                format!("the Claude Code check crashed: {err}"),
                previous.as_deref(),
            ),
        };
        self.lock().claude = status;
    }
}

fn failed(error: String, previous: Option<&str>) -> ClaudeStatus {
    if previous != Some(error.as_str()) {
        warn!("bots cannot start: {error}");
    }
    ClaudeStatus::Failed {
        error,
        at: Instant::now(),
    }
}

fn probe(source: &ClaudeSource) -> Result<Claude, ClaudeError> {
    match source {
        ClaudeSource::Fixed(claude) => Ok(claude.clone()),
        ClaudeSource::Discover { configured } => {
            let env = platform::user_environment().map_err(ClaudeError::Environment)?;
            let path = claude::locate(configured, &env)?;
            claude::probe(&path, &env)
        }
    }
}

pub(super) fn slot_entry<'a>(
    slots: &'a mut HashMap<BotId, Slot>,
    bot: &BotId,
    settings: &SupervisorSettings,
) -> &'a mut Slot {
    slots.entry(bot.clone()).or_insert_with(|| {
        let backoff = Backoff::new(settings.backoff_initial, settings.backoff_max);
        Slot::new(backoff)
    })
}
