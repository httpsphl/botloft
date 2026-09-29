//! Routines (spec 20) firing on a FakeRuntime with a manual clock: on
//! time, overlap, missed times, a paused bot, run now and turning off.

mod common;

use botloft_core::protocol::{
    BotsSetPausedParams, Missed, Overlap, RoutineIdParams, RoutinesSetEnabledParams, RunStatus,
    Schedule, SkipReason,
};
use botloftd::routines::schedule::at;
use botloftd::service::{bots, routines as service};
use common::bots::text_of;
use common::routines::*;

#[tokio::test(start_paused = true)]
async fn a_weekly_routine_fires_at_its_time_and_the_bot_knows_it() {
    let s = routines_setup().await;
    let weekdays = Schedule::Weekly {
        days: vec![1, 2, 3, 4, 5],
        time: "09:00".into(),
    };
    let morning = create(&s, weekdays, Overlap::Skip, Missed::RunOnce);
    assert_eq!(morning.next_run_at, Some(at("UTC", 2026, 10, 1, 9, 0)));

    tick_after(&s, 30 * MINUTE);
    assert!(runs(&s, &morning.id).is_empty(), "not yet");
    tick_after(&s, 30 * MINUTE);
    let fired = runs(&s, &morning.id);
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].status, RunStatus::Queued);

    let lines = written(&s, 1).await;
    assert_eq!(
        text_of(&lines[0]),
        "[botloft] routine \"Morning\" · scheduled 2026-10-01 09:00 (UTC)\n\
         Nobody is watching live: do the work, then report it in your reply.\n\n\
         Summarize what arrived in shared/inbox."
    );
    // Thursday 09:00 fired; Friday 09:00 is next.
    assert_eq!(
        routine(&s, &morning.id).next_run_at,
        Some(at("UTC", 2026, 10, 2, 9, 0))
    );

    finish_turn(&s, &lines[0], false).await;
    let done = runs(&s, &morning.id);
    assert_eq!(done[0].status, RunStatus::Done);
    assert!(done[0].finished_at.is_some());
    assert_eq!(
        routine(&s, &morning.id).last_run.map(|run| run.status),
        Some(RunStatus::Done)
    );
}

#[tokio::test(start_paused = true)]
async fn a_run_still_open_skips_the_next_time() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    tick_after(&s, 5 * MINUTE);
    let seen = runs(&s, &routine.id);
    assert_eq!(seen.len(), 2);
    assert_eq!(seen[1].status, RunStatus::Skipped);
    assert_eq!(seen[1].reason, Some(SkipReason::Overlap));

    finish_turn(&s, &lines[0], false).await;
    tick_after(&s, 5 * MINUTE);
    let seen = runs(&s, &routine.id);
    assert_eq!(seen[2].status, RunStatus::Queued, "free again");
}

#[tokio::test(start_paused = true)]
async fn queue_lets_one_run_wait_behind_the_open_one() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Queue, Missed::RunOnce);
    for _ in 0..3 {
        tick_after(&s, 5 * MINUTE);
    }
    let statuses: Vec<_> = runs(&s, &routine.id).iter().map(|run| run.status).collect();
    assert_eq!(
        statuses,
        [RunStatus::Queued, RunStatus::Queued, RunStatus::Skipped]
    );
}

#[tokio::test(start_paused = true)]
async fn missed_times_run_once_or_not_at_all() {
    let s = routines_setup().await;
    let once = create(&s, every(60), Overlap::Skip, Missed::RunOnce);
    let skip = create(&s, every(60), Overlap::Skip, Missed::Skip);
    // The computer was off for 5 h 10 min.
    tick_after(&s, 310 * MINUTE);

    let caught = runs(&s, &once.id);
    assert_eq!(caught.len(), 2);
    assert_eq!(caught[0].reason, Some(SkipReason::Missed));
    assert_eq!(caught[0].skipped_count, 4);
    assert_eq!(caught[1].status, RunStatus::Queued);
    assert_eq!(
        caught[1].scheduled_for,
        at("UTC", 2026, 10, 1, 13, 0),
        "the latest"
    );

    let skipped = runs(&s, &skip.id);
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0].status, RunStatus::Skipped);
    assert_eq!(skipped[0].skipped_count, 5);
    // Both keep their hourly grid.
    assert_eq!(
        routine(&s, &once.id).next_run_at,
        Some(at("UTC", 2026, 10, 1, 14, 0))
    );
    assert_eq!(
        routine(&s, &skip.id).next_run_at,
        Some(at("UTC", 2026, 10, 1, 14, 0))
    );
}

#[tokio::test(start_paused = true)]
async fn a_paused_bot_skips_its_times_and_run_now_still_queues() {
    let s = routines_setup().await;
    let routine = create(&s, every(30), Overlap::Skip, Missed::RunOnce);
    bots::set_paused(
        &s.daemon,
        BotsSetPausedParams {
            bot_id: s.bot.clone(),
            paused: true,
        },
    )
    .expect("pause");
    tick_after(&s, 30 * MINUTE);
    let seen = runs(&s, &routine.id);
    assert_eq!(seen[0].reason, Some(SkipReason::BotPaused));

    let next = self::routine(&s, &routine.id).next_run_at;
    let now = service::run_now(
        &s.daemon,
        RoutineIdParams {
            routine_id: routine.id.clone(),
        },
    )
    .expect("run now");
    assert_eq!(
        now.status,
        RunStatus::Queued,
        "the owner asked: it waits for the bot"
    );
    assert_eq!(
        self::routine(&s, &routine.id).next_run_at,
        next,
        "the schedule stays"
    );
}

#[tokio::test(start_paused = true)]
async fn off_routines_do_not_run_and_come_back_from_now() {
    let s = routines_setup().await;
    let routine = create(&s, every(60), Overlap::Skip, Missed::RunOnce);
    service::set_enabled(
        &s.daemon,
        RoutinesSetEnabledParams {
            routine_id: routine.id.clone(),
            enabled: false,
        },
    )
    .expect("off");
    tick_after(&s, 300 * MINUTE);
    assert!(runs(&s, &routine.id).is_empty());
    let back = service::set_enabled(
        &s.daemon,
        RoutinesSetEnabledParams {
            routine_id: routine.id.clone(),
            enabled: true,
        },
    )
    .expect("on");
    assert_eq!(
        back.next_run_at,
        Some(at("UTC", 2026, 10, 1, 14, 0)),
        "nothing missed"
    );
}
