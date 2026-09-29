//! `botloftd service ...`: the daemon as a per-user scheduled task that
//! starts at logon and comes back within a minute if it dies (spec 14).
//! Not a Windows service: a service runs in another session, without the
//! owner's Claude Code sign-in.

pub mod binary;
mod choice;
pub mod health;

pub use choice::{scheduled_start, set_start_with_windows, stop};

use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, bail};
use sha2::{Digest, Sha256};

use crate::config::Config;
use crate::paths;
use crate::platform::{self, TaskDefinition, TaskState, Triggers};

/// How long a daemon may take to stop or to answer after starting.
const STOP_TIMEOUT: Duration = Duration::from_secs(10);
const START_TIMEOUT: Duration = Duration::from_secs(20);
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// One task per data folder: `Botloft` for the default one, and a name
/// with a hash of the folder for a dev `BOTLOFT_HOME`, so installing a dev
/// daemon never replaces the real one.
pub fn task_name(home: &Path) -> String {
    let key = |path: &Path| path.to_string_lossy().to_lowercase();
    match paths::default_home() {
        Some(default) if key(&default) == key(home) => "Botloft".to_owned(),
        _ => {
            let digest = Sha256::digest(key(home).as_bytes());
            format!("Botloft-{}", hex::encode(&digest[..4]))
        }
    }
}

/// Quotes one argument for the Windows command line. Backslashes before
/// the closing quote are doubled, so `C:\` does not escape it.
fn quote(arg: &str) -> String {
    let trailing = arg.len() - arg.trim_end_matches('\\').len();
    format!("\"{arg}{}\"", "\\".repeat(trailing))
}

/// `--scheduled` tells the daemon the task started it (`choice`).
fn definition(home: &Path, program: &Path, triggers: Triggers) -> TaskDefinition {
    let home_text = home.to_string_lossy();
    TaskDefinition {
        description: format!(
            "Starts the Botloft daemon at logon and keeps it running. Data folder: {home_text}"
        ),
        program: program.to_owned(),
        arguments: format!("serve --home {} --scheduled", quote(&home_text)),
        working_dir: home.to_owned(),
        triggers,
    }
}

/// Registers the task with `triggers`, running the installed daemon.
fn register(home: &Path, triggers: Triggers) -> anyhow::Result<()> {
    let name = task_name(home);
    let program = bin_dir(home).join(binary::EXE_NAME);
    platform::register_task(&name, &definition(home, &program, triggers))
        .with_context(|| format!("cannot register the scheduled task {name}"))
}

fn config(home: &Path) -> anyhow::Result<Config> {
    Ok(Config::load(&home.join("config.toml"), false)?)
}

fn port(home: &Path) -> anyhow::Result<u16> {
    Ok(config(home)?.port)
}

fn bin_dir(home: &Path) -> PathBuf {
    home.join("bin")
}

/// Copies this binary to `<home>\bin`, registers the task and makes sure
/// the daemon of this version runs. A daemon that already runs this very
/// binary is left alone, so its bots are not interrupted.
pub fn install(home: &Path) -> anyhow::Result<()> {
    std::fs::create_dir_all(home).with_context(|| format!("cannot create {}", home.display()))?;
    let config = config(home)?;
    let port = config.port;
    let exe = std::env::current_exe().context("cannot find this program")?;
    let installed = binary::install(&exe, &bin_dir(home))
        .with_context(|| format!("cannot copy the daemon to {}", bin_dir(home).display()))?;
    let name = task_name(home);
    register(home, choice::running(config.start_with_windows))?;
    // The owner opened Botloft: it runs in this sign-in.
    choice::mark_sign_in(home);
    println!(
        "Scheduled task {name} starts {} at logon.",
        installed.path.display()
    );

    let running = platform::find_task(&name)?.is_some_and(|task| task.state == TaskState::Running);
    let current = health::probe(port).is_some_and(|health| health.version == VERSION);
    if running && current && !installed.changed {
        println!("The daemon {VERSION} is already running on 127.0.0.1:{port}.");
        return Ok(());
    }
    restart_task(home, port)
}

/// Stops the daemon and starts it again from the task.
pub fn restart(home: &Path) -> anyhow::Result<()> {
    let name = task_name(home);
    if platform::find_task(&name)?.is_none() {
        bail!("the scheduled task {name} is not installed; run `botloftd service install`");
    }
    restart_task(home, port(home)?)
}

fn restart_task(home: &Path, port: u16) -> anyhow::Result<()> {
    let name = &task_name(home);
    stop_task(name, port)?;
    choice::mark_sign_in(home);
    platform::run_task(name).with_context(|| format!("cannot start the scheduled task {name}"))?;
    let up = health::wait(port, START_TIMEOUT, |health| {
        health.is_some_and(|health| health.version == VERSION)
    });
    if !up {
        bail!(
            "the daemon did not answer on 127.0.0.1:{port} within {} s. Another program may \
             use the port, or another botloftd may run with this data folder; its log is in \
             the logs folder",
            START_TIMEOUT.as_secs()
        );
    }
    println!("The daemon {VERSION} is running on 127.0.0.1:{port}.");
    Ok(())
}

/// Ends the task's daemon and waits until it no longer answers. Killing
/// it is safe: every message is stored before it is delivered, and the
/// bots die with it (Job Objects).
fn stop_task(name: &str, port: u16) -> anyhow::Result<()> {
    let running = platform::find_task(name)?.is_some_and(|task| task.state == TaskState::Running);
    if !running {
        return Ok(());
    }
    platform::stop_task(name).with_context(|| format!("cannot stop the scheduled task {name}"))?;
    let stopped = health::wait(port, STOP_TIMEOUT, |health| health.is_none())
        && wait_until(STOP_TIMEOUT, || {
            platform::find_task(name)
                .is_ok_and(|task| task.is_none_or(|task| task.state != TaskState::Running))
        });
    if !stopped {
        bail!("the daemon of the scheduled task {name} did not stop");
    }
    Ok(())
}

/// Stops the daemon, deletes the task and the installed binary. The data
/// folder and the bots' workspaces stay.
pub fn uninstall(home: &Path) -> anyhow::Result<()> {
    let name = task_name(home);
    let port = port(home).unwrap_or(Config::default().port);
    stop_task(&name, port)?;
    if platform::delete_task(&name)? {
        println!("Deleted the scheduled task {name}.");
    } else {
        println!("The scheduled task {name} was not installed.");
    }
    binary::remove(&bin_dir(home));
    println!("Your bots and data stay in {}.", home.display());
    Ok(())
}

/// Prints whether the task is installed and whether the daemon answers.
pub fn status(home: &Path) -> anyhow::Result<()> {
    let name = task_name(home);
    match platform::find_task(&name)? {
        Some(task) => {
            let state = match task.state {
                TaskState::Running => "running",
                TaskState::Ready => "waiting for the next logon",
                TaskState::Disabled => "disabled",
                TaskState::Unknown => "in an unknown state",
            };
            println!("Scheduled task {name}: installed, {state}.");
            if let Some(command) = task.command {
                println!("  {command}");
            }
        }
        None => println!("Scheduled task {name}: not installed."),
    }
    let port = port(home)?;
    match health::probe(port) {
        Some(health) => println!(
            "Daemon: {} (protocol {}) answers on 127.0.0.1:{port}.",
            health.version, health.protocol
        ),
        None => println!("Daemon: nothing answers on 127.0.0.1:{port}."),
    }
    println!("Data folder: {}", home.display());
    Ok(())
}

fn wait_until(limit: Duration, done: impl Fn() -> bool) -> bool {
    let started = std::time::Instant::now();
    while !done() {
        if started.elapsed() > limit {
            return false;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arguments_are_quoted_for_the_command_line() {
        assert_eq!(
            quote(r"C:\Users\Ana Lima\Botloft"),
            r#""C:\Users\Ana Lima\Botloft""#
        );
        assert_eq!(quote(r"D:\"), r#""D:\\""#);
        let task = definition(
            Path::new(r"C:\data"),
            Path::new(r"C:\data\bin\botloftd.exe"),
            choice::running(true),
        );
        assert_eq!(task.arguments, r#"serve --home "C:\data" --scheduled"#);
        assert!(task.description.ends_with(r"Data folder: C:\data"));
    }

    #[test]
    fn a_dev_home_gets_its_own_task() {
        let default = paths::default_home().expect("default home");
        assert_eq!(task_name(&default), "Botloft");
        let upper = PathBuf::from(default.to_string_lossy().to_uppercase());
        assert_eq!(task_name(&upper), "Botloft", "Windows paths ignore case");
        let dev = task_name(Path::new(r"C:\src\botloft\.dev\home"));
        assert!(dev.starts_with("Botloft-") && dev.len() == "Botloft-".len() + 8);
        assert_ne!(dev, task_name(Path::new(r"C:\other\home")));
    }
}
