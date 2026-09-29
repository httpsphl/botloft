//! Everything that talks to the operating system directly. The rest of the
//! daemon calls these functions and never Win32.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::path::{Path, PathBuf};

#[cfg(unix)]
pub use unix::{
    KeepAwake, ProcessJob, delete_task, find_task, leave_own_console, owner_name, register_task,
    restrict_to_current_user, run_task, sign_in_id, stop_task, user_environment,
};
#[cfg(windows)]
pub use windows::{
    KeepAwake, ProcessJob, delete_task, find_task, leave_own_console, owner_name, register_task,
    restrict_to_current_user, run_task, sign_in_id, stop_task, user_environment,
};

/// A program the system starts for the owner (spec 14).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskDefinition {
    pub description: String,
    pub program: PathBuf,
    /// Already quoted for the command line.
    pub arguments: String,
    pub working_dir: PathBuf,
    pub triggers: Triggers,
}

/// When the system starts the task (spec 14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Triggers {
    /// When the owner signs in to Windows.
    pub logon: bool,
    /// Every minute, which brings the program back soon after it stops.
    pub watchdog: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Running,
    /// Installed and waiting for its trigger.
    Ready,
    Disabled,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInfo {
    pub state: TaskState,
    pub command: Option<String>,
    pub triggers: Triggers,
}

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
    fn user_environment_has_the_basics_and_no_claude_session() {
        let vars = user_environment().expect("environment");
        let has = |name: &str| vars.iter().any(|(k, _)| k.eq_ignore_ascii_case(name));
        assert!(has("PATH"));
        assert!(!has("CLAUDECODE"));
        assert!(!has("CLAUDE_CODE_MESSAGING_SOCKET"));
    }

    #[test]
    fn the_owner_has_a_name() {
        assert!(!owner_name().trim().is_empty());
    }

    #[test]
    fn a_job_can_be_created_and_terminated_empty() {
        let job = ProcessJob::new().expect("job");
        job.terminate().expect("terminate");
    }

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
