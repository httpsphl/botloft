//! Keeps every active bot running (spec 7): starts processes, follows their
//! state through the events they print, restarts them with backoff and
//! stops them when they are paused or archived.
//!
//! Lock order: the store lock may be held while taking the supervisor lock
//! (service calls read states), never the other way around.

mod reconcile;
mod relaunch;
mod settings;
mod sign_in;
mod slot;
mod spawn;
mod turns;

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::now_ms;
use botloft_core::protocol::{BotState, BotStateChanged};
use bytes::Bytes;
use tokio::sync::{Notify, broadcast, watch};
use tokio::time::Instant;
use tracing::warn;

use self::reconcile::slot_entry;
pub use self::settings::{ClaudeSource, SupervisorSettings};
use self::slot::{Slot, StopIntent};
use crate::runtime::Runtime;
use crate::runtime::claude::Claude;
use crate::secrets::TokenHash;
use crate::state::{Daemon, Event};

/// How often the supervisor compares running processes with the database.
const RECONCILE_EVERY: Duration = Duration::from_secs(5);
/// A session this long resets the restart backoff (spec 7.3).
const STABLE_AFTER: Duration = Duration::from_secs(10 * 60);
/// How long a failed Claude Code probe is trusted before trying again.
const REPROBE_AFTER: Duration = Duration::from_secs(30);

pub struct Supervisor {
    daemon: Weak<Daemon>,
    runtime: Arc<dyn Runtime>,
    settings: SupervisorSettings,
    events: broadcast::Sender<Event>,
    inner: Mutex<Inner>,
    /// Generations are unique across bots and daemon restarts, so a client
    /// never mistakes a new process for one it saw before.
    next_generation: AtomicU64,
    reconcile_lock: tokio::sync::Mutex<()>,
    wake: Notify,
    /// How many bots are `busy`, for keeping the computer awake (spec 14).
    busy: watch::Sender<usize>,
}

#[derive(Default)]
struct Inner {
    slots: HashMap<BotId, Slot>,
    /// Token hash of each running generation -> its bot.
    tokens: HashMap<String, (BotId, u64)>,
    /// The conversation each bot resumes, as far as this run knows; the
    /// database has it too (spec 7.3).
    sessions: HashMap<BotId, String>,
    claude: ClaudeStatus,
    sign_in: sign_in::SignIn,
}

#[derive(Default)]
enum ClaudeStatus {
    #[default]
    Unknown,
    Ready(Claude),
    Failed {
        error: String,
        at: Instant,
    },
}

impl Supervisor {
    pub(crate) fn new(
        daemon: Weak<Daemon>,
        runtime: Arc<dyn Runtime>,
        settings: SupervisorSettings,
        events: broadcast::Sender<Event>,
    ) -> Self {
        Self {
            daemon,
            runtime,
            settings,
            events,
            inner: Mutex::default(),
            next_generation: AtomicU64::new(u64::try_from(now_ms()).unwrap_or(1)),
            reconcile_lock: tokio::sync::Mutex::new(()),
            wake: Notify::new(),
            busy: watch::Sender::new(0),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Asks the run loop to reconcile now instead of at the next tick.
    pub fn wake(&self) {
        self.wake.notify_one();
    }

    /// State and generation of a bot the supervisor has seen.
    pub fn status(&self, bot: &BotId) -> Option<(BotState, Option<u64>)> {
        let inner = self.lock();
        inner
            .slots
            .get(bot)
            .map(|slot| (slot.state, slot.generation))
    }

    /// Follows how many bots are `busy`.
    pub fn busy_bots(&self) -> watch::Receiver<usize> {
        self.busy.subscribe()
    }

    /// The Claude Code executable the bots run, once found.
    pub fn claude_path(&self) -> Option<std::path::PathBuf> {
        match &self.lock().claude {
            ClaudeStatus::Ready(claude) => Some(claude.path.clone()),
            _ => None,
        }
    }

    pub fn claude_version(&self) -> Option<String> {
        match &self.lock().claude {
            ClaudeStatus::Ready(claude) => Some(claude.version.clone()),
            _ => None,
        }
    }

    /// Why bots cannot start, if Claude Code is missing or too old.
    pub fn runtime_error(&self) -> Option<String> {
        match &self.lock().claude {
            ClaudeStatus::Failed { error, .. } => Some(error.clone()),
            _ => None,
        }
    }

    fn slot<'a>(&self, inner: &'a mut Inner, bot: &BotId) -> &'a mut Slot {
        slot_entry(&mut inner.slots, bot, &self.settings)
    }

    /// Writes one stream-json line to the bot's stdin (spec 9.2). A message
    /// is one turn more for the bot. Returns the generation it went to.
    pub fn write_message(&self, bot: &BotId, line: Bytes) -> Result<u64, NotRunning> {
        let mut inner = self.lock();
        let slot = self.slot(&mut inner, bot);
        let (Some(running), Some(generation)) = (&slot.running, slot.generation) else {
            return Err(NotRunning);
        };
        if slot.stop.is_some() {
            return Err(NotRunning);
        }
        running.control.write(line).map_err(|_| NotRunning)?;
        slot.turns += 1;
        if slot.is_working() {
            self.set_state(bot, slot, slot.working_state());
        }
        Ok(generation)
    }

    /// Whether `generation` is the bot's running process.
    pub fn is_current(&self, bot: &BotId, generation: u64) -> bool {
        self.lock()
            .slots
            .get(bot)
            .is_some_and(|slot| slot.generation == Some(generation) && slot.running.is_some())
    }

    /// Restarts the bot, also out of `auth_error`. The caller checked that
    /// the bot should run.
    pub fn restart(&self, bot: &BotId, fresh: bool) {
        let mut inner = self.lock();
        let slot = self.slot(&mut inner, bot);
        slot.backoff.reset();
        match &slot.running {
            Some(running) => {
                slot.stop = Some(StopIntent::Restart { fresh });
                if let Err(err) = running.control.kill() {
                    warn!(bot = %bot, "could not stop the bot for a restart: {err}");
                }
            }
            None => {
                slot.fresh_next = fresh;
                slot.restart_at = None;
                if slot.state == BotState::AuthError {
                    self.set_state(bot, slot, BotState::Offline);
                }
            }
        }
        drop(inner);
        self.wake();
    }

    /// The bot and generation an MCP token belongs to.
    pub fn token_owner(&self, token: &str) -> Option<(BotId, u64)> {
        self.lock()
            .tokens
            .get(&TokenHash::of(token).to_hex())
            .cloned()
    }

    fn set_state(&self, bot: &BotId, slot: &mut Slot, state: BotState) {
        if slot.state != state {
            self.count_busy(slot.state, state);
            slot.state = state;
            self.announce(bot, slot);
        }
    }

    /// Keeps the busy count in step with a state change. Every change of
    /// `slot.state` goes through here.
    fn count_busy(&self, from: BotState, to: BotState) {
        let (was, is) = (from == BotState::Busy, to == BotState::Busy);
        if was != is {
            self.busy.send_modify(|count| {
                *count = if is {
                    *count + 1
                } else {
                    count.saturating_sub(1)
                };
            });
        }
    }

    /// Emits `bot.state`. A new process always announces itself, even when
    /// the state name did not change, because its generation did.
    fn announce(&self, bot: &BotId, slot: &Slot) {
        let _ = self.events.send(Event::BotState(BotStateChanged {
            bot_id: bot.clone(),
            state: slot.state,
            generation: slot.generation,
        }));
    }

    /// Kills every bot, for daemon shutdown.
    pub fn shutdown(&self) {
        let inner = self.lock();
        for running in inner
            .slots
            .values()
            .filter_map(|slot| slot.running.as_ref())
        {
            let _ = running.control.kill();
        }
    }
}

/// The bot has no running process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the bot is not running")]
pub struct NotRunning;

/// Reconciles on start, every few seconds and whenever something wakes the
/// loop. Runs until the daemon is dropped.
pub async fn run(daemon: Arc<Daemon>) {
    let supervisor = &daemon.supervisor;
    loop {
        supervisor.reconcile().await;
        let next = supervisor.next_restart().map_or(RECONCILE_EVERY, |at| {
            at.saturating_duration_since(Instant::now())
                .min(RECONCILE_EVERY)
        });
        tokio::select! {
            () = tokio::time::sleep(next) => {}
            () = supervisor.wake.notified() => {}
        }
    }
}
