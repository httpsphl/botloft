//! The loop of the automatic backup (spec 27.10).

use std::sync::Arc;
use std::time::Duration;

use botloft_core::protocol::AutoBackupEvery;
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
    daemon
        .autobackup
        .update(|state| record(state, &outcome, saved.every, now));
    if matches!(outcome, Outcome::Failed { turn_off: true, .. }) {
        daemon.autobackup.turn_off();
    }
    daemon.emit(Event::AutoBackupChanged(daemon.autobackup.status()));
}

fn after(at: i64, by: Duration) -> i64 {
    crate::clock::after(at, by)
}

/// What the pass leaves behind. `sent_every` is how often it was set when the
/// pass began: the owner may have changed it while a copy went up, and that
/// wins, so the next look comes soon and counts the new period.
fn record(state: &mut Saved, outcome: &Outcome, sent_every: AutoBackupEvery, now: i64) {
    let unchanged = state.every == sent_every;
    let next = |by: Duration| unchanged.then(|| after(now, by));
    match outcome {
        Outcome::Sent(fingerprint) => {
            state.last_ok_at = Some(now);
            state.fingerprint = Some(fingerprint.clone());
            state.next_at = next(period(state.every)).or(state.next_at);
            state.last_error = None;
        }
        Outcome::Unchanged => {
            state.next_at = next(period(state.every)).or(state.next_at);
            state.last_error = None;
        }
        Outcome::Failed { reason, retry, .. } => {
            state.next_at = next(*retry).or(state.next_at);
            state.last_error = Some(reason.clone());
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_000_000;

    fn on(every: AutoBackupEvery) -> Saved {
        Saved {
            enabled: true,
            every,
            ..Saved::default()
        }
    }

    #[test]
    fn a_sent_copy_waits_one_period() {
        let mut state = on(AutoBackupEvery::Daily);
        record(
            &mut state,
            &Outcome::Sent("ab".to_owned()),
            AutoBackupEvery::Daily,
            NOW,
        );
        assert_eq!(state.last_ok_at, Some(NOW));
        assert_eq!(state.fingerprint.as_deref(), Some("ab"));
        assert_eq!(state.next_at, Some(NOW + 86_400_000));
    }

    #[test]
    fn a_change_of_period_made_during_the_pass_is_not_undone() {
        // The pass began as daily; the owner chose weekly and asked it to look again.
        let mut state = on(AutoBackupEvery::Weekly);
        state.next_at = None;
        record(
            &mut state,
            &Outcome::Sent("ab".to_owned()),
            AutoBackupEvery::Daily,
            NOW,
        );
        assert_eq!(state.last_ok_at, Some(NOW));
        assert_eq!(state.next_at, None);
        record(&mut state, &Outcome::Unchanged, AutoBackupEvery::Daily, NOW);
        assert_eq!(state.next_at, None);
        // The next pass, begun as weekly, counts the new period.
        record(
            &mut state,
            &Outcome::Unchanged,
            AutoBackupEvery::Weekly,
            NOW,
        );
        assert_eq!(state.next_at, Some(NOW + 7 * 86_400_000));
    }

    #[test]
    fn a_failure_is_kept_with_when_to_try_again() {
        let mut state = on(AutoBackupEvery::Daily);
        let failed = Outcome::Failed {
            reason: "offline".to_owned(),
            retry: SOON,
            turn_off: false,
        };
        record(&mut state, &failed, AutoBackupEvery::Daily, NOW);
        assert_eq!(state.last_error.as_deref(), Some("offline"));
        assert_eq!(state.next_at, Some(NOW + 900_000));
        record(&mut state, &Outcome::Unchanged, AutoBackupEvery::Daily, NOW);
        assert_eq!(state.last_error, None);
    }
}
