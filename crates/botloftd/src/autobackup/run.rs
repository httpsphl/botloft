//! The loop of the automatic backup (spec 27.10).

use std::sync::Arc;
use std::time::Duration;

use tracing::info;

use super::{Saved, period};
use crate::backup::fingerprint::light_fingerprint;
use crate::service::{ApiError, cloud};
use crate::state::{Daemon, Event};

/// After a failure that may pass by itself, such as being offline.
const SOON: Duration = Duration::from_secs(15 * 60);
/// After one that needs the owner, such as signing in again.
const LATER: Duration = Duration::from_secs(3600);

/// Runs until the daemon stops.
pub async fn run(daemon: Arc<Daemon>) {
    loop {
        let pass = Arc::clone(&daemon);
        let _ = tokio::task::spawn_blocking(move || tick(&pass)).await;
        tokio::select! {
            () = tokio::time::sleep(daemon.autobackup.tick) => {}
            () = daemon.autobackup.wake.notified() => {}
        }
    }
}

enum Outcome {
    Sent(String),
    /// Nothing the owner made changed since the last copy.
    Unchanged,
    Failed {
        reason: String,
        retry: Duration,
        /// Nothing more to do until the owner turns it on again.
        turn_off: bool,
    },
}

fn failed(reason: &str, retry: Duration) -> Outcome {
    Outcome::Failed {
        reason: reason.to_owned(),
        retry,
        turn_off: false,
    }
}

/// One pass: sends a copy if it is on, its time has come and something
/// changed. It blocks (disk, network), so it runs on the blocking pool; tests
/// call it after moving the clock.
pub fn tick(daemon: &Daemon) {
    let saved = daemon.autobackup.snapshot();
    let now = daemon.clock.now_ms();
    if !saved.enabled || saved.next_at.is_some_and(|at| now < at) {
        return;
    }
    let outcome = attempt(daemon, &saved);
    let wait = period(saved.every);
    daemon.autobackup.update(|state| match &outcome {
        Outcome::Sent(fingerprint) => {
            state.last_ok_at = Some(now);
            state.fingerprint = Some(fingerprint.clone());
            state.next_at = Some(after(now, wait));
            state.last_error = None;
        }
        Outcome::Unchanged => {
            state.next_at = Some(after(now, wait));
            state.last_error = None;
        }
        Outcome::Failed { reason, retry, .. } => {
            state.next_at = Some(after(now, *retry));
            state.last_error = Some(reason.clone());
        }
    });
    if matches!(outcome, Outcome::Failed { turn_off: true, .. }) {
        daemon.autobackup.turn_off();
    }
    daemon.emit(Event::AutoBackupChanged(daemon.autobackup.status()));
}

fn after(at: i64, by: Duration) -> i64 {
    crate::clock::after(at, by)
}

fn attempt(daemon: &Daemon, saved: &Saved) -> Outcome {
    if daemon.cloud.credentials(&daemon.paths.secrets()).is_none() {
        return failed("not_signed_in", LATER);
    }
    let passphrase = match daemon.autobackup.secrets().load() {
        Ok(Some(passphrase)) => passphrase,
        // Gone from the credential store: it cannot go on.
        Ok(None) | Err(_) => {
            return Outcome::Failed {
                reason: "no_passphrase".to_owned(),
                retry: LATER,
                turn_off: true,
            };
        }
    };
    let Ok(fingerprint) = light_fingerprint(daemon) else {
        return failed("cloud_error", LATER);
    };
    if saved.last_ok_at.is_some() && saved.fingerprint.as_deref() == Some(&fingerprint) {
        return Outcome::Unchanged;
    }
    match cloud::send_light(daemon, &passphrase) {
        Ok(copy) => {
            info!(bytes = copy.size, "an automatic copy went up to the cloud");
            Outcome::Sent(fingerprint)
        }
        Err(ApiError::Rule { reason, .. }) => {
            // A copy too big or an account too full stays so until the owner
            // frees space, so it waits for the next turn rather than trying
            // again soon.
            let retry = match reason {
                "offline" | "cloud_error" | "rate_limited" => SOON,
                "quota" | "too_big" => period(saved.every),
                _ => LATER,
            };
            failed(reason, retry)
        }
        Err(_) => failed("cloud_error", LATER),
    }
}
