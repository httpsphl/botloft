//! The owner's desktop as the bots use it (spec 24): one action at a time
//! across every bot, and whether the owner is there to see it.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::sync::MutexGuard;

/// How long the owner may go without touching the computer and still count
/// as there (spec 24.8).
pub const AWAY_AFTER: Duration = Duration::from_secs(5 * 60);

/// How long ago the owner last used the mouse or keyboard; `None` where
/// that cannot be known.
pub type OwnerIdle = Arc<dyn Fn() -> Option<Duration> + Send + Sync>;

pub struct Desktop {
    /// App connections that said hello, open now.
    apps: AtomicUsize,
    /// Held for each desktop action: the cursor is one.
    turn: tokio::sync::Mutex<()>,
    idle: Mutex<OwnerIdle>,
}

/// Why the bot may not use the desktop now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Away {
    /// Botloft's app is closed.
    NoApp,
    /// Nobody touched the computer for a while.
    Idle,
}

impl Away {
    /// What the bot reads.
    pub fn why(self) -> &'static str {
        match self {
            Self::NoApp => {
                "The owner does not have Botloft open, so you may not use their desktop now. \
                 Only grants they marked for use while they are away work then."
            }
            Self::Idle => {
                "The owner has not used the computer for a while, so you may not use their \
                 desktop now. Only grants they marked for use while they are away work then."
            }
        }
    }
}

impl Desktop {
    pub fn new(idle: OwnerIdle) -> Self {
        Self {
            apps: AtomicUsize::new(0),
            turn: tokio::sync::Mutex::new(()),
            idle: Mutex::new(idle),
        }
    }

    pub fn app_opened(&self) {
        self.apps.fetch_add(1, Ordering::SeqCst);
    }

    pub fn app_closed(&self) {
        let _ = self
            .apps
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |open| {
                open.checked_sub(1)
            });
    }

    /// Changes how the owner's idle time is read: tests stand in for them.
    pub fn set_owner_idle(&self, idle: OwnerIdle) {
        *self.idle.lock().unwrap_or_else(PoisonError::into_inner) = idle;
    }

    /// Whether the owner is there to see what a bot does (spec 24.8): the
    /// app is open and they used the computer lately.
    pub fn owner_here(&self) -> Result<(), Away> {
        if self.apps.load(Ordering::SeqCst) == 0 {
            return Err(Away::NoApp);
        }
        let idle = self
            .idle
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        match idle() {
            Some(idle) if idle <= AWAY_AFTER => Ok(()),
            _ => Err(Away::Idle),
        }
    }

    /// Waits for the desktop to be free; it is the caller's until dropped.
    pub async fn turn(&self) -> MutexGuard<'_, ()> {
        self.turn.lock().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idle_for(seconds: u64) -> OwnerIdle {
        Arc::new(move || Some(Duration::from_secs(seconds)))
    }

    #[test]
    fn the_owner_is_there_with_the_app_open_and_the_computer_in_use() {
        let desktop = Desktop::new(idle_for(10));
        assert_eq!(desktop.owner_here(), Err(Away::NoApp));
        desktop.app_opened();
        assert_eq!(desktop.owner_here(), Ok(()));
        desktop.set_owner_idle(idle_for(AWAY_AFTER.as_secs() + 1));
        assert_eq!(desktop.owner_here(), Err(Away::Idle));
        desktop.set_owner_idle(Arc::new(|| None));
        assert_eq!(desktop.owner_here(), Err(Away::Idle));
        desktop.set_owner_idle(idle_for(0));
        desktop.app_closed();
        desktop.app_closed();
        assert_eq!(desktop.owner_here(), Err(Away::NoApp));
    }
}
