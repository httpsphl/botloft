//! The automatic backup (spec 27.10): once the owner turns it on, a light
//! copy goes to the cloud every day or week, and only when something the
//! owner made has changed. It seals with the owner's passphrase, kept in the
//! system's credential store, and stops by itself when the account or the
//! passphrase is gone.

mod run;
mod secrets;
mod state;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::protocol::AutoBackupStatus;
use tokio::sync::Notify;
use tracing::warn;

pub use self::run::{run, tick};
pub use self::secrets::{MemorySecrets, SecretStore, SystemSecrets};
pub(crate) use self::state::{Saved, period};

/// How often it looks whether its time has come.
pub const DEFAULT_TICK: Duration = Duration::from_secs(60);

pub struct AutoBackup {
    file: PathBuf,
    saved: Mutex<Saved>,
    secrets: Arc<dyn SecretStore>,
    wake: Notify,
    tick: Duration,
}

impl AutoBackup {
    pub fn new(home: &Path, secrets: Arc<dyn SecretStore>, tick: Duration) -> Self {
        let file = home.join("autobackup.json");
        Self {
            saved: Mutex::new(state::load(&file)),
            file,
            secrets,
            wake: Notify::new(),
            tick,
        }
    }

    pub fn secrets(&self) -> &dyn SecretStore {
        &*self.secrets
    }

    /// Looks now instead of at the next tick: it was turned on or changed.
    pub fn wake(&self) {
        self.wake.notify_one();
    }

    pub fn snapshot(&self) -> Saved {
        self.lock().clone()
    }

    /// Changes what it remembers and saves it.
    pub(crate) fn update(&self, change: impl FnOnce(&mut Saved)) {
        let mut saved = self.lock();
        change(&mut saved);
        if let Err(err) = state::save(&self.file, &saved) {
            warn!("could not save the automatic backup's state: {err}");
        }
    }

    pub fn status(&self) -> AutoBackupStatus {
        let saved = self.snapshot();
        AutoBackupStatus {
            available: self.secrets.available(),
            enabled: saved.enabled,
            every: saved.every,
            last_ok_at: saved.last_ok_at,
            next_at: saved.enabled.then_some(saved.next_at).flatten(),
            last_error: saved.last_error,
        }
    }

    /// A manual copy that held `fingerprint` went up at `now`: the next look
    /// has nothing to send until something changes.
    pub(crate) fn covered(&self, fingerprint: String, now: i64) {
        let wait = period(self.snapshot().every);
        self.update(|saved| {
            saved.fingerprint = Some(fingerprint);
            saved.last_ok_at = Some(now);
            saved.next_at = Some(crate::clock::after(now, wait));
            saved.last_error = None;
        });
    }

    /// Off, with the passphrase forgotten.
    pub fn turn_off(&self) {
        if let Err(err) = self.secrets.delete() {
            warn!("could not remove the automatic backup's passphrase: {err}");
        }
        self.update(|saved| {
            saved.enabled = false;
            saved.fingerprint = None;
            saved.next_at = None;
        });
    }

    fn lock(&self) -> MutexGuard<'_, Saved> {
        self.saved
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}
