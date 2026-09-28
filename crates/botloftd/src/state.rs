//! State shared by every connection: the store, the supervisor, the
//! courier, the event bus and what the daemon knows about itself.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Instant;

use botloft_core::protocol::{Bot, BotStateChanged, Crew, Delivery, Message, Task};
use botloft_store::Store;
use tokio::sync::broadcast;

use crate::clock::Clock;
use crate::courier::{Courier, CourierSettings, InboxWriter};
use crate::paths::Paths;
use crate::runtime::Runtime;
use crate::secrets::TokenHash;
use crate::supervisor::{Supervisor, SupervisorSettings};
use crate::workspace::WorkspaceEnv;

/// Something that changed and every connected app should hear about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    CrewChanged(Crew),
    BotChanged(Bot),
    BotState(BotStateChanged),
    MessageCreated(Message),
    DeliveryChanged(Delivery),
    TaskChanged(Task),
}

/// Events buffered per connection before a slow client is dropped.
const EVENT_BUFFER: usize = 1024;

pub struct DaemonOptions {
    pub paths: Paths,
    /// Port the daemon listens on, written into each bot's `mcp.json`.
    pub port: u16,
    /// Absolute path of `botloftd.exe`, run by the bots' hooks.
    pub bin: PathBuf,
    pub store: Store,
    pub owner_token: TokenHash,
    pub runtime: Arc<dyn Runtime>,
    pub supervisor: SupervisorSettings,
    pub clock: Arc<dyn Clock>,
    pub inbox: Arc<dyn InboxWriter>,
    pub courier: CourierSettings,
}

pub struct Daemon {
    pub paths: Paths,
    pub port: u16,
    pub bin: PathBuf,
    pub supervisor: Supervisor,
    pub courier: Courier,
    /// Time for everything stored or compared with stored times.
    pub clock: Arc<dyn Clock>,
    store: Mutex<Store>,
    owner_token: TokenHash,
    events: broadcast::Sender<Event>,
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
            courier: Courier::new(options.courier, options.inbox),
            clock: options.clock,
            paths: options.paths,
            port: options.port,
            bin: options.bin,
            store: Mutex::new(options.store),
            owner_token: options.owner_token,
            events,
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
            bin: &self.bin,
            port: self.port,
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

    pub fn uptime_ms(&self) -> i64 {
        i64::try_from(self.started.elapsed().as_millis()).unwrap_or(i64::MAX)
    }
}
