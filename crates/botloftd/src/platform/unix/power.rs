//! Keeps the computer from sleeping while bots work, off Windows (spec
//! 14.1): `caffeinate -i` on macOS, an idle lock of `systemd-inhibit` on
//! Linux, held by a helper process for as long as it is wanted. Like the
//! power request on Windows, it only stops sleep after inactivity: closing
//! the lid or choosing Sleep still works. The helper also ends when the
//! daemon does, so a crash does not keep the computer awake.

use std::io;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

#[cfg_attr(target_os = "macos", allow(dead_code))]
const REASON: &str = "Botloft bots are working";
/// A helper that fails (no logind, say) exits at once; one still running
/// after this holds the lock.
const SETTLE: Duration = Duration::from_millis(200);

#[derive(Debug, Default)]
pub struct KeepAwake {
    helper: Option<Child>,
}

impl KeepAwake {
    pub fn new() -> io::Result<Self> {
        Ok(Self::default())
    }

    pub fn set(&mut self, on: bool) -> io::Result<()> {
        if on == self.running() {
            return Ok(());
        }
        if !on {
            self.stop();
            return Ok(());
        }
        let mut helper = helper(std::process::id())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        std::thread::sleep(SETTLE);
        if let Some(status) = helper.try_wait()? {
            return Err(io::Error::other(format!(
                "the keep-awake helper stopped at once ({status})"
            )));
        }
        self.helper = Some(helper);
        Ok(())
    }

    /// Whether the helper holds the lock now.
    fn running(&mut self) -> bool {
        self.helper
            .as_mut()
            .is_some_and(|helper| matches!(helper.try_wait(), Ok(None)))
    }

    fn stop(&mut self) {
        if let Some(mut helper) = self.helper.take() {
            let _ = helper.kill();
            let _ = helper.wait();
        }
    }
}

impl Drop for KeepAwake {
    fn drop(&mut self) {
        self.stop();
    }
}

/// `-w` ends `caffeinate` with the daemon.
#[cfg(target_os = "macos")]
fn helper(daemon: u32) -> Command {
    let mut command = Command::new("caffeinate");
    command.args(["-i", "-w", &daemon.to_string()]);
    command
}

/// The lock lasts while the shell it runs does, and the shell watches the
/// daemon.
#[cfg(not(target_os = "macos"))]
fn helper(daemon: u32) -> Command {
    let mut command = Command::new("systemd-inhibit");
    command.args([
        "--what=idle",
        "--who=Botloft",
        &format!("--why={REASON}"),
        "--mode=block",
        "sh",
        "-c",
        &format!("while kill -0 {daemon} 2>/dev/null; do sleep 5; done"),
    ]);
    command
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_helper_runs_only_while_wanted() {
        let mut awake = KeepAwake::new().expect("keep awake");
        if let Err(err) = awake.set(true) {
            // No `caffeinate` or no logind here (a container, say).
            eprintln!("cannot keep this computer awake ({err}); skipping");
            return;
        }
        assert!(awake.running());
        awake.set(true).expect("again");
        assert!(awake.running(), "asking twice keeps one helper");
        awake.set(false).expect("off");
        assert!(!awake.running());
    }
}
