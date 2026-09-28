//! State shared by every connection: the store, the supervisor, the
//! courier, the event bus and what the daemon knows about itself.

use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use botloft_core::protocol::{
    AccountUsage, Bot, BotStateChanged, ChatDelta, ChatItemChanged, Crew, Delivery, Message, Task,
};
use botloft_store::Store;
use tokio::sync::broadcast;

use crate::approvals::Approvals;
use crate::clock::Clock;
use crate::config::Config;
use crate::courier::{Courier, CourierSettings};
use crate::paths::Paths;
use crate::runtime::Runtime;
use crate::secrets::TokenHash;
use crate::service::tasks::TaskSettings;
use crate::supervisor::{Supervisor, SupervisorSettings};
use crate::workspace::WorkspaceEnv;

/// Something that changed and every connected app should hear about.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    CrewChanged(Crew),
    BotChanged(Bot),
    BotState(BotStateChanged),
    ChatItem(ChatItemChanged),
    ChatDelta(ChatDelta),
    MessageCreated(Message),
    DeliveryChanged(Delivery),
    TaskChanged(Task),
}

/// Events buffered per connection before a slow client is dropped.
const EVENT_BUFFER: usize = 1024;

/// Settings for what bots may ask of the owner (spec 6, `[bots]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BotSettings {
    /// How long a permission request waits for the owner (spec 10.1).
    pub approval_timeout: Duration,
    /// Largest attachment, per file (spec 9.5).
    pub attachment_max_bytes: u64,
}

impl BotSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            approval_timeout: Duration::from_secs(config.bots.approval_timeout_minutes * 60),
            attachment_max_bytes: config.bots.attachment_max_mb * 1024 * 1024,
        }
    }
}

pub struct DaemonOptions {
    pub paths: Paths,
    /// Port the daemon listens on, written into each bot's `mcp.json`.
    pub port: u16,
    pub store: Store,
    pub owner_token: TokenHash,
    pub runtime: Arc<dyn Runtime>,
    pub supervisor: SupervisorSettings,
    pub clock: Arc<dyn Clock>,
    pub courier: CourierSettings,
    pub tasks: TaskSettings,
    pub bots: BotSettings,
}

pub struct Daemon {
    pub paths: Paths,
    pub port: u16,
    pub supervisor: Supervisor,
    pub courier: Courier,
    pub tasks: TaskSettings,
    pub bots: BotSettings,
    pub approvals: Approvals,
    /// Time for everything stored or compared with stored times.
    pub clock: Arc<dyn Clock>,
    store: Mutex<Store>,
    owner_token: TokenHash,
    events: broadcast::Sender<Event>,
    usage: Mutex<Option<AccountUsage>>,
    started: Instant,
}

impl Daemon {
    /// The supervisor keeps a weak reference back to the daemon, so both are
    /// created together. Start them with [`crate::supervisor::run`] and
    /// [`crate::courier::run`].
    pub fn new(options: DaemonOptions) -> Arc<Self> {
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        Arc::new_cyclic(|daemon| Self {
            supervisor: Supervisor::new(
                daemon.clone(),
                options.runtime,
                options.supervisor,
                events.clone(),
            ),
            courier: Courier::new(options.courier),
            tasks: options.tasks,
            bots: options.bots,
            approvals: Approvals::default(),
            clock: options.clock,
            paths: options.paths,
            port: options.port,
            store: Mutex::new(options.store),
            owner_token: options.owner_token,
            events,
            usage: Mutex::new(None),
            started: Instant::now(),
        })
    }

    /// Locks the store. Hold the guard only for synchronous work, never
    /// across an `.await`.
    pub fn store(&self) -> MutexGuard<'_, Store> {
        // A panic while holding the lock cannot leave SQLite half-written
        // (every write is its own statement or transaction), so keep going.
        self.store
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn workspace_env(&self) -> WorkspaceEnv<'_> {
        WorkspaceEnv {
            paths: &self.paths,
            port: self.port,
            approval_timeout: self.bots.approval_timeout,
        }
    }

    pub fn is_owner_token(&self, token: &str) -> bool {
        self.owner_token.matches(token)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    pub(crate) fn emit(&self, event: Event) {
        // No receivers just means no app is connected.
        let _ = self.events.send(event);
    }

    /// The account usage Claude Code last reported (spec 8.1).
    pub fn usage(&self) -> Option<AccountUsage> {
        self.usage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub(crate) fn set_usage(&self, usage: AccountUsage) {
        *self
            .usage
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(usage);
    }

    pub fn uptime_ms(&self) -> i64 {
        i64::try_from(self.started.elapsed().as_millis()).unwrap_or(i64::MAX)
    }
}
