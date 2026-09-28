//! State shared by every connection: the store, the event bus and what the
//! daemon knows about itself.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};
use std::time::Instant;

use botloft_core::protocol::{Bot, Crew};
use botloft_store::Store;
use tokio::sync::broadcast;

use crate::paths::Paths;
use crate::secrets::TokenHash;
use crate::workspace::WorkspaceEnv;

/// Something that changed and every connected app should hear about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    CrewChanged(Crew),
    BotChanged(Bot),
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
}

pub struct Daemon {
    pub paths: Paths,
    pub port: u16,
    pub bin: PathBuf,
    store: Mutex<Store>,
    owner_token: TokenHash,
    events: broadcast::Sender<Event>,
    started: Instant,
}

impl Daemon {
    pub fn new(options: DaemonOptions) -> Self {
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        Self {
            paths: options.paths,
            port: options.port,
            bin: options.bin,
            store: Mutex::new(options.store),
            owner_token: options.owner_token,
            events,
            started: Instant::now(),
        }
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
