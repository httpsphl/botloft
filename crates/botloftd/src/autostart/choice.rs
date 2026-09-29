//! The owner's choices about when Botloft runs (spec 14): starting with
//! Windows, and stopping it when they close the app.
//!
//! The task's triggers follow them:
//! - running: the minute trigger that brings a stopped daemon back, plus
//!   the sign-in trigger when Botloft starts with Windows;
//! - stopped by the owner: only the sign-in trigger, when Botloft starts
//!   with Windows, so nothing brings it back before then.
//!
//! The minute trigger also fires after a new sign-in. When Botloft does
//! not start with Windows, the daemon it starts sees that it is a new
//! sign-in (`<home>\signin` names the last one Botloft ran in), takes the
//! triggers off and exits. The app starts it again when the owner opens it.

use std::path::Path;

use anyhow::{Context, bail};
use tracing::{info, warn};

use super::{config, register, stop_task, task_name};
use crate::platform::{self, Triggers};

const SIGN_IN_FILE: &str = "signin";

/// The triggers while the daemon should run.
pub(super) fn running(start_with_windows: bool) -> Triggers {
    Triggers {
        logon: start_with_windows,
        watchdog: true,
    }
}

/// The triggers after the owner stopped it.
fn stopped(start_with_windows: bool) -> Triggers {
    Triggers {
        logon: start_with_windows,
        watchdog: false,
    }
}

/// Records that Botloft runs in the owner's current sign-in.
pub(super) fn mark_sign_in(home: &Path) {
    let Some(id) = platform::sign_in_id() else {
        return;
    };
    if let Err(err) = std::fs::write(home.join(SIGN_IN_FILE), id) {
        warn!("cannot record the sign-in Botloft runs in: {err}");
    }
}

/// Whether Botloft has not run in this sign-in yet. Unknown sign-ins
/// count as the same one, so the daemon keeps running.
fn is_new_sign_in(home: &Path) -> bool {
    let Some(id) = platform::sign_in_id() else {
        return false;
    };
    std::fs::read_to_string(home.join(SIGN_IN_FILE)).map_or(true, |last| last.trim() != id)
}

/// What the daemon started by its task does first. Returns whether it
/// should run: not after a new sign-in when Botloft does not start with
/// Windows.
pub fn scheduled_start(home: &Path, start_with_windows: bool) -> bool {
    if !start_with_windows && is_new_sign_in(home) {
        if let Err(err) = register(home, stopped(false)) {
            warn!("cannot take the triggers off the scheduled task: {err:#}");
        }
        return false;
    }
    mark_sign_in(home);
    // Started at sign-in after the owner stopped it: the minute trigger
    // comes back.
    let wanted = running(start_with_windows);
    let task = platform::find_task(&task_name(home));
    if task.is_ok_and(|task| task.is_some_and(|task| task.triggers != wanted))
        && let Err(err) = register(home, wanted)
    {
        warn!("cannot update the scheduled task: {err:#}");
    }
    true
}

/// `service stop`: stops the daemon until the owner opens Botloft again,
/// or until the next sign-in when it starts with Windows.
pub fn stop(home: &Path) -> anyhow::Result<()> {
    let name = task_name(home);
    if platform::find_task(&name)?.is_none() {
        bail!("the scheduled task {name} is not installed; run `botloftd service install`");
    }
    let config = config(home)?;
    register(home, stopped(config.start_with_windows))?;
    stop_task(&name, config.port)?;
    println!("Stopped the daemon of the scheduled task {name}.");
    Ok(())
}

/// Follows the owner turning starting with Windows on or off while the
/// daemon runs. A task that is not installed has nothing to follow.
pub fn set_start_with_windows(home: &Path, start: bool) -> anyhow::Result<()> {
    let name = task_name(home);
    let task = platform::find_task(&name)
        .with_context(|| format!("cannot read the scheduled task {name}"))?;
    if let Some(task) = task {
        register(
            home,
            Triggers {
                logon: start,
                watchdog: task.triggers.watchdog,
            },
        )?;
    }
    // The daemon runs in this sign-in, whatever started it.
    mark_sign_in(home);
    info!(start, "start with Windows");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sign_in_is_new_until_botloft_runs_in_it() {
        let home = tempfile::tempdir().expect("home");
        let Some(id) = platform::sign_in_id() else {
            // Without a sign-in to name, the daemon always runs.
            assert!(!is_new_sign_in(home.path()));
            return;
        };
        assert!(is_new_sign_in(home.path()), "nothing recorded yet");
        std::fs::write(home.path().join(SIGN_IN_FILE), "1-1").expect("write");
        assert!(is_new_sign_in(home.path()), "another sign-in");
        mark_sign_in(home.path());
        assert_eq!(
            std::fs::read_to_string(home.path().join(SIGN_IN_FILE)).expect("read"),
            id
        );
        assert!(!is_new_sign_in(home.path()));
    }

    #[test]
    fn running_keeps_the_minute_trigger_and_stopping_takes_it_off() {
        for start in [true, false] {
            assert_eq!(running(start).logon, start);
            assert_eq!(stopped(start).logon, start);
            assert!(running(start).watchdog);
            assert!(!stopped(start).watchdog);
        }
    }
}
