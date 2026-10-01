//! `botloftd service` against the real systemd or launchd (spec 14.1):
//! install, come back after a crash, stay stopped after a stop, uninstall.
//! It changes the owner's service manager, so it runs only when
//! `BOTLOFT_SERVICE_TEST=1` (CI sets it on its Linux and macOS runners).
#![cfg(unix)]

use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;

use botloftd::autostart::health;

const BACK_WITHIN: Duration = Duration::from_secs(60);
const STOP_WITHIN: Duration = Duration::from_secs(20);

struct Installed {
    home: PathBuf,
}

impl Installed {
    fn service(&self, command: &str) -> Output {
        eprintln!("service {command}");
        Command::new(env!("CARGO_BIN_EXE_botloftd"))
            .arg("--home")
            .arg(&self.home)
            .args(["service", command])
            .output()
            .expect("run botloftd")
    }

    fn ok(&self, command: &str) -> String {
        let output = self.service(command);
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        self.check(
            output.status.success(),
            &format!("service {command}: {said}"),
        );
        said
    }

    /// Fails with the daemon's own log, which says why it did not run.
    fn check(&self, holds: bool, what: &str) {
        if holds {
            return;
        }
        let mut logs = String::new();
        for entry in std::fs::read_dir(self.home.join("logs"))
            .into_iter()
            .flatten()
            .flatten()
        {
            logs += &format!("--- {}\n", entry.path().display());
            logs += &std::fs::read_to_string(entry.path()).unwrap_or_default();
        }
        panic!("{what}\n{logs}");
    }
}

/// Uninstalls even when the test fails, so the runner keeps no agent.
impl Drop for Installed {
    fn drop(&mut self) {
        let _ = self.service("uninstall");
    }
}

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.local_addr().expect("address").port()
}

fn daemon_pid(home: &Path) -> Option<String> {
    let output = Command::new("pgrep")
        .args(["-f", &format!("serve --home {}", home.display())])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .map(str::to_owned)
}

#[test]
fn the_service_brings_the_daemon_back_until_it_is_stopped() {
    if std::env::var_os("BOTLOFT_SERVICE_TEST").is_none() {
        eprintln!("BOTLOFT_SERVICE_TEST is not set; skipping");
        return;
    }
    // Under the home folder, like a real data folder: the CI runner keeps a
    // `/tmp` of its own that the user's service manager does not see.
    let dir = tempfile::tempdir_in(dirs::home_dir().expect("home folder")).expect("tempdir");
    let home = dir.path().join("home");
    std::fs::create_dir_all(&home).expect("home");
    let port = free_port();
    std::fs::write(
        home.join("config.toml"),
        format!(
            "port = {port}\nworkspaces_root = '{}'\n",
            dir.path().join("bots").display()
        ),
    )
    .expect("config");
    let installed = Installed { home: home.clone() };

    let up = |installed: &Installed, what: &str| {
        installed.check(health::probe(port).is_some(), what);
    };
    installed.ok("install");
    up(&installed, "install waits for the daemon");
    let status = installed.ok("status");
    installed.check(status.contains("is installed, running"), &status);

    eprintln!("a crash");
    let pid = daemon_pid(&home);
    installed.check(pid.is_some(), "the daemon's pid");
    Command::new("kill")
        .args(["-9", &pid.unwrap_or_default()])
        .status()
        .expect("kill");
    installed.check(
        health::wait(port, STOP_WITHIN, |health| health.is_none()),
        "the daemon died",
    );
    installed.check(
        health::wait(port, BACK_WITHIN, |health| health.is_some()),
        "the daemon came back after the crash",
    );

    // A stop asked for: it stays stopped.
    installed.ok("stop");
    installed.check(health::probe(port).is_none(), "stopped");
    std::thread::sleep(Duration::from_secs(15));
    installed.check(health::probe(port).is_none(), "nothing brought it back");

    // Starting again works from the stopped state.
    installed.ok("restart");
    up(&installed, "restarted");

    let said = installed.ok("uninstall");
    installed.check(said.contains("Deleted the"), &said);
    installed.check(health::probe(port).is_none(), "gone after uninstall");
    let status = installed.ok("status");
    installed.check(status.contains("is not installed"), &status);
}
