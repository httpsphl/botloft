//! Comparing what runs with what the database says should run.

use std::collections::HashMap;

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::{BotState, Crew};
use botloft_store::BotRecord;
use tokio::time::Instant;
use tracing::warn;

use super::sleep::Rest;
use super::slot::{Backoff, Slot, StopIntent};
use super::start::Start;
use super::{
    CONVERSATIONS_AWAKE, ClaudeSource, ClaudeStatus, Inner, OTHER_CONVERSATIONS_NAP, REPROBE_AFTER,
    Supervisor, SupervisorSettings,
};
use crate::platform;
use crate::runtime::claude::{self, Claude, ClaudeError};

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
                // Every conversation that is not archived (spec 40).
                let mut sessions = HashMap::new();
                let mut others: HashMap<BotId, Vec<ConversationId>> = HashMap::new();
                for conversation in store.conversations(None)? {
                    if let Some(session) = conversation.session_id {
                        sessions.insert(conversation.id.clone(), session);
                    }
                    if !conversation.main {
                        others
                            .entry(conversation.bot_id)
                            .or_default()
                            .push(conversation.id);
                    }
                }
                // Who a message waits for comes first under a limit (7.8).
                let with_work = store.bots_with_work()?;
                Ok((bots, crews, sessions, others, with_work))
            })
        };
        let (bots, crews, sessions, others, with_work) = match read {
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
        self.watch_turns(&daemon);
        // Read before the supervisor lock, like the database.
        let sleep_after = daemon.settings.sleep_after();
        let max_running = daemon.settings.max_running();
        let now = Instant::now();
        let mut inner = self.lock();
        for (conversation, session) in sessions {
            inner.sessions.entry(conversation).or_insert(session);
        }
        let mut starts: Vec<Start> = Vec::new();
        for bot in &bots {
            let Some(crew) = crews.get(&bot.crew_id) else {
                continue;
            };
            let main = ConversationId::main_of(&bot.id);
            let mine = others.get(&bot.id).map(Vec::as_slice).unwrap_or_default();
            self.close_archived(&mut inner, &bot.id, mine);
            for conversation in std::iter::once(&main).chain(mine) {
                // Another conversation sleeps whenever it is idle (40.4).
                let nap = if conversation == &main {
                    sleep_after
                } else {
                    sleep_after.or(Some(OTHER_CONVERSATIONS_NAP))
                };
                let start =
                    self.reconcile_conversation(&mut inner, crew, bot, conversation, now, nap);
                starts.extend(start.and_then(|start| self.awake_room(&mut inner, start, &starts)));
            }
        }
        let starts = self.admit(&mut inner, starts, &with_work, max_running);
        drop(inner);
        // One after another, but nothing else waits for them.
        for start in starts {
            self.launch(&daemon, start);
        }
        self.forget_replaced(&daemon);
    }

    /// Stops what should not run; returns the start of a conversation that
    /// should.
    fn reconcile_conversation(
        &self,
        inner: &mut Inner,
        crew: &Crew,
        bot: &BotRecord,
        conversation: &ConversationId,
        now: Instant,
        sleep_after: Option<std::time::Duration>,
    ) -> Option<Start> {
        // Deleted after this pass read the database.
        if inner.gone.contains(&bot.id) {
            return None;
        }
        let archived = bot.archived_at.is_some() || crew.archived_at.is_some();
        if archived && !inner.slots.contains_key(conversation) {
            return None;
        }
        let wanted = !archived && !bot.paused && !crew.paused;
        let slot = slot_entry(&mut inner.slots, &bot.id, conversation, &self.settings);
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
            return None;
        }
        if let Rest::Asleep = self.rest(&bot.id, slot, now, sleep_after) {
            return None;
        }
        // A process on its way out settles first; its exit wakes the loop.
        let waiting = slot.restart_at.is_some_and(|at| at > now);
        if slot.running.is_some() || slot.launching || slot.stop.is_some() || waiting {
            return None;
        }
        if slot.state == BotState::AuthError {
            return None;
        }
        slot.wake_asked = false;
        self.plan_start(inner, crew, bot, conversation)
    }

    /// A conversation the owner archived stops, and its slot goes (40.3).
    fn close_archived(&self, inner: &mut Inner, bot: &BotId, active: &[ConversationId]) {
        let gone: Vec<ConversationId> = inner
            .slots
            .iter()
            .filter(|(id, slot)| slot.bot == *bot && !slot.main && !active.contains(id))
            .map(|(id, _)| id.clone())
            .collect();
        for id in gone {
            let Some(slot) = inner.slots.get_mut(&id) else {
                continue;
            };
            match &slot.running {
                Some(running) if slot.stop.is_none() => {
                    slot.stop = Some(StopIntent::Halt(BotState::Offline));
                    if let Err(err) = running.control.kill() {
                        warn!(bot = %bot, "could not stop an archived conversation: {err}");
                    }
                }
                Some(_) => {}
                None if !slot.launching => {
                    self.count_busy(slot.state, BotState::Offline);
                    inner.slots.remove(&id);
                    inner.sessions.remove(&id);
                }
                None => {}
            }
        }
    }

    /// At most `CONVERSATIONS_AWAKE` conversations of one bot run at once
    /// (spec 40.4). Another that must start puts the bot's longest idle one
    /// to sleep and waits for it, or waits for one to finish.
    fn awake_room(&self, inner: &mut Inner, start: Start, planned: &[Start]) -> Option<Start> {
        let bot = start.bot_id().clone();
        let awake = inner
            .slots
            .iter()
            .filter(|(id, slot)| {
                slot.bot == bot
                    && *id != start.conversation()
                    && (slot.running.is_some() || slot.launching)
            })
            .count()
            + planned
                .iter()
                .filter(|other| *other.bot_id() == bot)
                .count();
        if awake < CONVERSATIONS_AWAKE {
            return Some(start);
        }
        let idle = inner
            .slots
            .iter_mut()
            .filter(|(_, slot)| {
                slot.bot == bot
                    && slot.running.is_some()
                    && slot.stop.is_none()
                    && slot.state == BotState::Idle
                    && !slot.has_work()
                    && slot.tasks == 0
            })
            .min_by_key(|(_, slot)| slot.idle_since);
        if let Some((_, slot)) = idle {
            slot.stop = Some(StopIntent::Halt(BotState::Sleeping));
            if let Some(running) = &slot.running
                && let Err(err) = running.control.kill()
            {
                warn!(bot = %bot, "could not put a conversation to sleep: {err}");
            }
        }
        // It starts at a later pass, once there is room.
        if let Some(slot) = inner.slots.get_mut(start.conversation()) {
            slot.launching = false;
            slot.wake_asked = true;
            self.set_state(&bot, slot, BotState::Sleeping);
        }
        None
    }

    /// Earliest scheduled restart, or the moment an idle bot falls
    /// asleep (spec 7.7), so the run loop can wake for it.
    pub(super) fn next_due(&self, sleep_after: Option<std::time::Duration>) -> Option<Instant> {
        let inner = self.lock();
        let restarts = inner
            .slots
            .values()
            .filter(|slot| slot.running.is_none())
            .filter_map(|slot| slot.restart_at);
        let naps = inner
            .slots
            .values()
            .filter(|slot| slot.running.is_some() && slot.state == BotState::Idle)
            .filter_map(|slot| Some(slot.idle_since? + sleep_after?));
        restarts.chain(naps).min()
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
    slots: &'a mut HashMap<ConversationId, Slot>,
    bot: &BotId,
    conversation: &ConversationId,
    settings: &SupervisorSettings,
) -> &'a mut Slot {
    slots.entry(conversation.clone()).or_insert_with(|| {
        let backoff = Backoff::new(settings.backoff_initial, settings.backoff_max);
        Slot::new(bot, conversation, backoff)
    })
}
