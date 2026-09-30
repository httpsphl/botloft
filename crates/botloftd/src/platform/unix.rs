//! Unix fallbacks, so the daemon builds and its tests run off Windows.

use std::ffi::OsString;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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

/// No job objects here; the runtime kills the child directly.
#[derive(Debug)]
pub struct ProcessJob;

impl ProcessJob {
    pub fn new() -> io::Result<Self> {
        Ok(Self)
    }

    pub fn terminate(&self) -> io::Result<()> {
        Ok(())
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

/// The daemon's environment without Claude Code session variables.
pub fn user_environment() -> io::Result<Vec<(OsString, OsString)>> {
    Ok(std::env::vars_os()
        .filter(|(name, _)| {
            let name = name.to_string_lossy();
            name != "CLAUDECODE" && !name.starts_with("CLAUDE_CODE_")
        })
        .collect())
}

/// The login name; Unix has no display name to ask for.
pub fn owner_name() -> String {
    std::env::var("USER").unwrap_or_default()
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

pub fn find_task(_name: &str) -> io::Result<Option<TaskInfo>> {
    Err(no_tasks())
}

pub fn run_task(_name: &str) -> io::Result<()> {
    Err(no_tasks())
}

pub fn stop_task(_name: &str) -> io::Result<()> {
    Err(no_tasks())
}

pub fn delete_task(_name: &str) -> io::Result<bool> {
    Err(no_tasks())
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
