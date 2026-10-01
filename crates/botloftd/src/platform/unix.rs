//! Linux and macOS (spec 14.1): what bots get on Windows, and the daemon
//! started for the owner by systemd or launchd.

mod account;
mod env;
mod job;
// Both build everywhere, so each one's tests run on either system.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
mod launchd;
mod orphans;
#[cfg_attr(target_os = "macos", allow(dead_code))]
mod systemd;

use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

pub use self::account::owner_name;
pub use self::env::user_environment;
pub use self::job::ProcessJob;
#[cfg(target_os = "macos")]
pub use self::launchd::{TASK_KIND, delete_task, find_task, register_task, run_task, stop_task};
pub use self::orphans::track_groups;
#[cfg(not(target_os = "macos"))]
pub use self::systemd::{TASK_KIND, delete_task, find_task, register_task, run_task, stop_task};

/// Longest a `systemctl` or `launchctl` call may take.
const MANAGER_TIMEOUT: Duration = Duration::from_secs(30);

/// Runs a service manager command and collects what it said, giving up
/// after `MANAGER_TIMEOUT`: a stuck `launchctl` must not hang the app.
fn manager(program: &str, args: &[&str]) -> io::Result<Output> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let deadline = Instant::now() + MANAGER_TIMEOUT;
    while child.try_wait()?.is_none() {
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!(
                    "`{program} {}` did not finish in {} s",
                    args.join(" "),
                    MANAGER_TIMEOUT.as_secs()
                ),
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    child.wait_with_output()
}

/// Writes `path` whole or not at all, creating its folder.
fn write_atomically(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut staged = path.as_os_str().to_owned();
    staged.push(".new");
    std::fs::write(&staged, contents)?;
    std::fs::rename(&staged, path)
}

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

/// There is no Recycle Bin off Windows: the folder stays.
pub fn recycle(_path: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "the Recycle Bin is only supported on Windows",
    ))
}

/// No sign-in to tell apart: systemd and launchd start the daemon at
/// sign-in only when the owner wants it, so a new sign-in needs no check.
pub fn sign_in_id() -> Option<String> {
    None
}
