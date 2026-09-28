//! Unix fallbacks, so the daemon builds and its tests run off Windows.

use std::ffi::OsString;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

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

/// The daemon's environment without Claude Code session variables.
pub fn user_environment() -> io::Result<Vec<(OsString, OsString)>> {
    Ok(std::env::vars_os()
        .filter(|(name, _)| {
            let name = name.to_string_lossy();
            name != "CLAUDECODE" && !name.starts_with("CLAUDE_CODE_")
        })
        .collect())
}
