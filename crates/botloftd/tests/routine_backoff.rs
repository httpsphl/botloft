//! A routine that keeps failing waits longer (spec 20.3): from the second
//! failure in a row its next time moves past a wait that doubles, and a run
//! that goes well brings back its usual pace.

mod common;

use botloft_core::protocol::{Missed, Overlap, RunStatus, Schedule};
use botloftd::routines::schedule::at;
use common::routines::*;

fn time(hour: i8, minute: i8) -> Option<i64> {
    Some(at("UTC", 2026, 10, 1, hour, minute))
}

#[tokio::test(start_paused = true)]
async fn failures_in_a_row_push_the_next_time_and_a_good_run_resets_it() {
    let s = routines_setup().await;
    let check = create(&s, every(5), Overlap::Skip, Missed::RunOnce);

    // 08:05 fails: one failure does not wait.
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    finish_turn(&s, &lines[0], true).await;
    let now = routine(&s, &check.id);
    assert_eq!(now.fail_streak, 1);
    assert_eq!(now.next_run_at, time(8, 10));

    // 08:10 fails again: 15 minutes from now, on the routine's own steps.
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 2).await;
    finish_turn(&s, &lines[1], true).await;
    let now = routine(&s, &check.id);
    assert_eq!(now.fail_streak, 2);
    assert_eq!(now.next_run_at, time(8, 25));

    // 08:15 and 08:20 pass with nothing, not even a skipped run.
    tick_after(&s, 5 * MINUTE);
    tick_after(&s, 5 * MINUTE);
    assert_eq!(runs(&s, &check.id).len(), 2);

    // 08:25 fails a third time: the wait doubles to 30 minutes.
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 3).await;
    finish_turn(&s, &lines[2], true).await;
    let now = routine(&s, &check.id);
    assert_eq!(now.fail_streak, 3);
    assert_eq!(now.next_run_at, time(8, 55));

    // 08:55 goes well: back to every 5 minutes.
    tick_after(&s, 30 * MINUTE);
    let lines = written(&s, 4).await;
    finish_turn(&s, &lines[3], false).await;
    let now = routine(&s, &check.id);
    assert_eq!(now.fail_streak, 0);
    assert_eq!(now.next_run_at, time(9, 0));
    let statuses: Vec<_> = runs(&s, &check.id).iter().map(|run| run.status).collect();
    assert_eq!(
        statuses,
        [
            RunStatus::Failed,
            RunStatus::Failed,
            RunStatus::Failed,
            RunStatus::Done
        ]
    );
}

#[tokio::test(start_paused = true)]
async fn a_shortcut_that_fails_keeps_running_when_clicked() {
    let s = routines_setup().await;
    let shortcut = create(&s, Schedule::Manual, Overlap::Skip, Missed::RunOnce);
    for n in 1..=2 {
        botloftd::routines::fire_now(&s.daemon, &shortcut.id).expect("run now");
        let lines = written(&s, n).await;
        finish_turn(&s, &lines[n - 1], true).await;
    }
    let now = routine(&s, &shortcut.id);
    assert_eq!(now.fail_streak, 2, "the row is counted");
    assert_eq!(now.next_run_at, None, "but a shortcut has no time to move");
}
