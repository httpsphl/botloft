//! Keeps every active bot running (spec 7): starts processes, follows their
//! state through hooks, restarts them with backoff and stops them when they
//! are paused or archived.
//!
//! Lock order: the store lock may be held while taking the supervisor lock
//! (service calls read states), never the other way around.

mod reconcile;
mod settings;
mod slot;
mod spawn;

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, MutexGuard, Weak};
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::now_ms;
use botloft_core::protocol::{BotState, BotStateChanged};
use bytes::Bytes;
use tokio::sync::{Notify, broadcast};
use tokio::time::Instant;
use tracing::{debug, warn};

use self::reconcile::slot_entry;
pub use self::settings::{ClaudeSource, SupervisorSettings};
pub use self::slot::{Hook, Inbox};
use self::slot::{Slot, StopIntent, state_after_hook};
use crate::runtime::claude::Claude;
use crate::runtime::{Runtime, TermSize};
use crate::secrets::TokenHash;
use crate::state::{Daemon, Event};
use crate::terminal::Terminal;

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
}

#[derive(Default)]
struct Inner {
    slots: HashMap<BotId, Slot>,
    /// Token hash of each running generation -> its bot.
    tokens: HashMap<String, (BotId, u64)>,
    claude: ClaudeStatus,
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

    pub fn terminal(&self, bot: &BotId) -> Arc<Terminal> {
        let mut inner = self.lock();
        Arc::clone(&self.slot(&mut inner, bot).terminal)
    }

    fn slot<'a>(&self, inner: &'a mut Inner, bot: &BotId) -> &'a mut Slot {
        slot_entry(&mut inner.slots, bot, &self.settings)
    }

    /// Sends keystrokes to the bot. Answering a permission prompt resumes
    /// the turn (spec 7.2).
    pub fn write(&self, bot: &BotId, data: Bytes) -> Result<(), NotRunning> {
        let mut inner = self.lock();
        let slot = self.slot(&mut inner, bot);
        let running = slot.running.as_ref().ok_or(NotRunning)?;
        running.control.write(data).map_err(|_| NotRunning)?;
        if slot.state == BotState::NeedsApproval {
            self.set_state(bot, slot, BotState::Busy);
        }
        Ok(())
    }

    /// Answers the process of `generation` on behalf of the terminal.
    pub(crate) fn reply(&self, bot: &BotId, generation: u64, data: &'static [u8]) {
        let inner = self.lock();
        let running = inner
            .slots
            .get(bot)
            .filter(|slot| slot.generation == Some(generation))
            .and_then(|slot| slot.running.as_ref());
        if let Some(running) = running {
            let _ = running.control.write(Bytes::from_static(data));
        }
    }

    /// Remembers the size for the next start and applies it now if running.
    pub fn resize(&self, bot: &BotId, size: TermSize) {
        let mut inner = self.lock();
        let slot = self.slot(&mut inner, bot);
        slot.size = size;
        if let Some(running) = &slot.running
            && let Err(err) = running.control.resize(size)
        {
            debug!(bot = %bot, "resize failed: {err}");
        }
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

    /// The bot and generation a hook token belongs to.
    pub fn hook_owner(&self, token: &str) -> Option<(BotId, u64)> {
        self.lock()
            .tokens
            .get(&TokenHash::of(token).to_hex())
            .cloned()
    }

    pub fn on_hook(&self, bot: &BotId, generation: u64, hook: Hook) {
        let mut inner = self.lock();
        let slot = self.slot(&mut inner, bot);
        if slot.generation != Some(generation) || slot.running.is_none() {
            return;
        }
        debug!(bot = %bot, generation, ?hook, "hook");
        if let Hook::SessionStart { inbox } = &hook {
            slot.inbox.clone_from(inbox);
            if let Some(running) = &slot.running {
                spawn::mark_started(&running.workspace);
            }
        }
        if let Some(next) = state_after_hook(slot.state, &hook) {
            self.set_state(bot, slot, next);
        }
    }

    /// Where to deliver messages to the bot's current generation.
    pub fn inbox(&self, bot: &BotId) -> Option<Inbox> {
        self.lock()
            .slots
            .get(bot)
            .and_then(|slot| slot.inbox.clone())
    }

    fn set_state(&self, bot: &BotId, slot: &mut Slot, state: BotState) {
        if slot.state != state {
            slot.state = state;
            self.announce(bot, slot);
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
