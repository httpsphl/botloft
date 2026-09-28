//! Everything that talks to the operating system directly. The rest of the
//! daemon calls these functions and never Win32.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::Path;

#[cfg(unix)]
pub use unix::restrict_to_current_user;
#[cfg(windows)]
pub use windows::restrict_to_current_user;

/// Exclusive lock on `<home>\botloftd.lock`, held for the daemon's lifetime.
/// The OS releases it when the process exits, even after a crash.
#[derive(Debug)]
pub struct InstanceLock {
    _file: File,
}

#[derive(Debug, thiserror::Error)]
pub enum LockError {
    #[error("another botloftd is already running with this data folder")]
    AlreadyRunning,
    #[error("cannot open the lock file: {0}")]
    Io(#[from] io::Error),
}

impl InstanceLock {
    pub fn acquire(path: &Path) -> Result<Self, LockError> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(path)?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(LockError::AlreadyRunning),
            Err(TryLockError::Error(err)) => Err(LockError::Io(err)),
        }
    }
}

/// Resolves when the daemon should stop: Ctrl+C, and on Windows also the
/// console closing, logoff and shutdown (spec 14).
pub async fn shutdown_signal() {
    #[cfg(windows)]
    windows::shutdown_signal().await;
    #[cfg(unix)]
    unix::shutdown_signal().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_second_lock_on_the_same_file_is_refused_until_released() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("botloftd.lock");
        let first = InstanceLock::acquire(&path).expect("first lock");
        assert!(matches!(
            InstanceLock::acquire(&path),
            Err(LockError::AlreadyRunning)
        ));
        drop(first);
        InstanceLock::acquire(&path).expect("lock after release");
    }
}
