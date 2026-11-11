//! Waiting longer after failures (spec 20.3). From the second run in a row
//! that failed, a routine on a schedule does not run again before a wait
//! that starts at 15 minutes and doubles with each failure, up to 6 hours:
//! a routine that keeps failing (a site down, a key gone) stops spending
//! the plan every few minutes. Its next time moves to the first scheduled
//! one after the wait, so the times in between leave no skipped runs.

use botloft_core::ids::RoutineId;
use botloft_core::protocol::Schedule;
use tracing::{debug, warn};

use super::fire::next_run_at;
use crate::state::Daemon;

/// The wait after the second failure in a row.
const FIRST_WAIT_MS: i64 = 15 * 60_000;
/// The longest wait.
const MAX_WAIT_MS: i64 = 6 * 60 * 60_000;

/// How long a routine waits after `streak` failed runs in a row.
pub fn wait_after(streak: u32) -> i64 {
    if streak < 2 {
        return 0;
    }
    // 15 min, 30 min, 1 h, 2 h... capped before the shift could overflow.
    let doublings = (streak - 2).min(16);
    FIRST_WAIT_MS
        .saturating_mul(1 << doublings)
        .min(MAX_WAIT_MS)
}

/// A run of `routine` failed at `failed_at`: its next time waits if this
/// makes two or more failures in a row. The caller announces the routine.
pub(super) fn after_failure(daemon: &Daemon, routine: &RoutineId, failed_at: i64) {
    let found = daemon.store().routine(routine);
    let mut routine = match found {
        Ok(Some(routine)) => routine,
        Ok(None) => return,
        Err(err) => {
            warn!(routine = %routine, "could not read a routine after a failed run: {err}");
            return;
        }
    };
    // Only times on a schedule wait: a shortcut, a one-time run and a
    // signal run when someone asks.
    let scheduled = matches!(
        routine.schedule,
        Schedule::Weekly { .. } | Schedule::Interval { .. } | Schedule::Cron { .. }
    );
    let wait = wait_after(routine.fail_streak);
    let Some(next) = routine.next_run_at else {
        return;
    };
    let until = failed_at + wait;
    if !scheduled || !routine.enabled || wait == 0 || next >= until {
        return;
    }
    let Some(later) = next_run_at(&routine, next, until - 1) else {
        return;
    };
    routine.next_run_at = Some(later);
    debug!(
        routine = %routine.id,
        streak = routine.fail_streak,
        wait_ms = wait,
        "a routine that keeps failing waits longer"
    );
    if let Err(err) = daemon.store().update_routine(&routine) {
        warn!(routine = %routine.id, "could not move a failing routine's next time: {err}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wait_starts_at_the_second_failure_and_doubles_up_to_six_hours() {
        assert_eq!(wait_after(0), 0);
        assert_eq!(wait_after(1), 0);
        let minutes = |streak| wait_after(streak) / 60_000;
        assert_eq!(
            [2, 3, 4, 5, 6, 7, 8].map(minutes),
            [15, 30, 60, 120, 240, 360, 360]
        );
        assert_eq!(wait_after(u32::MAX), MAX_WAIT_MS);
    }
}
