//! Linux and macOS (spec 14.1). Starting with the system comes later; the
//! rest gives bots what they get on Windows.

mod account;
mod env;
mod job;

use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

pub use self::account::owner_name;
pub use self::env::user_environment;
pub use self::job::ProcessJob;
use super::{TaskDefinition, TaskInfo};

/// `0700` for folders, `0600` for files.
pub fn restrict_to_current_user(path: &Path) -> io::Result<()> {
    let mode = if path.is_dir() { 0o700 } else { 0o600 };
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
}

pub async fn shutdown_signal() {
    use tokio::signal::unix::{SignalKind, signal};

    let terminate = async {
        match signal(SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        () = terminate => {}
    }
}

/// Nothing keeps a Unix machine awake yet.
#[derive(Debug)]
pub struct KeepAwake;

impl KeepAwake {
    pub fn new() -> io::Result<Self> {
        Ok(Self)
    }

    pub fn set(&mut self, _on: bool) -> io::Result<()> {
        Ok(())
    }
}

/// Nothing to leave: Unix never opens a console for the daemon.
pub fn leave_own_console() {}

fn no_tasks() -> io::Error {
    io::Error::new(
        io::ErrorKind::Unsupported,
        "starting at logon is only supported on Windows",
    )
}

pub fn register_task(_name: &str, _task: &TaskDefinition) -> io::Result<()> {
    Err(no_tasks())
}

/// Nothing can be installed here yet, so there is never a task to find.
pub fn find_task(_name: &str) -> io::Result<Option<TaskInfo>> {
    Ok(None)
}

pub fn run_task(_name: &str) -> io::Result<()> {
    Err(no_tasks())
}

pub fn stop_task(_name: &str) -> io::Result<()> {
    Err(no_tasks())
}

pub fn delete_task(_name: &str) -> io::Result<bool> {
    Ok(false)
}

/// There is no Recycle Bin off Windows: the folder stays.
pub fn recycle(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "the Recycle Bin is only supported on Windows",
    ))
}

/// Unix has no Windows sign-in to tell apart.
pub fn sign_in_id() -> Option<String> {
    None
}
