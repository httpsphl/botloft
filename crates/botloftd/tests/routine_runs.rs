//! How a routine's run ends (spec 20.3): a failed turn, a dead message, a
//! process or a daemon gone mid-turn; and routines that go with their bot
//! or never get saved.

mod common;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::protocol::{
    BotIdParams, MessagesSendParams, Missed, Overlap, RoutinesCreateParams, RunStatus, Schedule,
};
use botloft_store::DeliveryOutcome;
use botloftd::routines;
use botloftd::service::{ApiError, bots, messages, routines as service};
use common::routines::*;
use common::stream;

#[tokio::test(start_paused = true)]
async fn a_turn_that_fails_or_a_dead_message_fails_the_run() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Queue, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    finish_turn(&s, &lines[0], true).await;
    assert_eq!(runs(&s, &routine.id)[0].status, RunStatus::Failed);

    // The next message dies before the bot reads it.
    tick_after(&s, 5 * MINUTE);
    let open = runs(&s, &routine.id)[1].clone();
    {
        let store = s.daemon.store();
        let message = open.message_id.clone().expect("message");
        let delivery = store
            .deliveries(None, Some(&s.bot))
            .expect("deliveries")
            .into_iter()
            .find(|delivery| delivery.message_id == message)
            .expect("delivery");
        store
            .finish_delivery(&delivery.id, DeliveryOutcome::Dead { error: "test" }, 0)
            .expect("dead");
    }
    routines::tick(&s.daemon);
    assert_eq!(runs(&s, &routine.id)[1].status, RunStatus::Failed);
}

#[tokio::test(start_paused = true)]
async fn a_process_that_ends_mid_turn_fails_the_run() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    let process = s.runtime.process(1).await;
    process.emit(stream::replay(&lines[0])).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    process.exit(1).await;
    s.runtime.process(2).await;
    assert_eq!(runs(&s, &routine.id)[0].status, RunStatus::Failed);
}

/// A message that joins the run's turn ends with it (spec 9.1): the run
/// is still the turn's, and is done at its `result`.
#[tokio::test(start_paused = true)]
async fn a_message_that_joins_the_runs_turn_leaves_the_run_to_it() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    let process = s.runtime.process(1).await;
    process.emit(stream::replay(&lines[0])).await;

    messages::send(
        &s.daemon,
        MessagesSendParams {
            bot_id: s.bot.clone(),
            body: "one more thing".into(),
            attachments: None,
            reply_to: None,
        },
    )
    .expect("send");
    s.clock.advance(Duration::from_secs(5));
    let lines = written(&s, 2).await;
    process.emit(stream::replay(&lines[1])).await;
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(runs(&s, &routine.id)[0].status, RunStatus::Done);
}

#[tokio::test(start_paused = true)]
async fn archiving_the_bot_archives_its_routines() {
    let s = routines_setup().await;
    let routine = create(&s, every(60), Overlap::Skip, Missed::RunOnce);
    bots::archive(
        &s.daemon,
        BotIdParams {
            bot_id: s.bot.clone(),
        },
    )
    .expect("archive");
    assert!(self::routine(&s, &routine.id).archived_at.is_some());
    tick_after(&s, 60 * MINUTE);
    assert!(runs(&s, &routine.id).is_empty());
}

#[tokio::test(start_paused = true)]
async fn schedules_too_close_or_unknown_zones_are_refused() {
    let s = routines_setup().await;
    let mut params = RoutinesCreateParams {
        bot_id: s.bot.clone(),
        name: "Too often".into(),
        prompt: "Check".into(),
        schedule: Schedule::Cron {
            expr: "* * * * *".into(),
        },
        timezone: "UTC".into(),
        overlap: None,
        missed: None,
    };
    let err = service::create(&s.daemon, params.clone()).expect_err("too often");
    assert!(err.to_string().contains("5 minutes"), "{err}");
    assert!(matches!(
        err,
        ApiError::Rule {
            reason: "too_often",
            ..
        }
    ));
    params.schedule = every(60);
    params.timezone = "Mars/Olympus".into();
    let err = service::create(&s.daemon, params).expect_err("zone");
    assert!(err.to_string().contains("time zone"), "{err}");
    assert!(matches!(
        err,
        ApiError::Rule {
            reason: "timezone_unknown",
            ..
        }
    ));
}

#[tokio::test(start_paused = true)]
async fn a_run_left_mid_turn_by_the_last_daemon_fails_at_start() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    // The bot began it; then the daemon went away before the turn ended.
    s.runtime
        .process(1)
        .await
        .emit(stream::replay(&lines[0]))
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    tokio::spawn(routines::run(Arc::clone(&s.daemon)));
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(runs(&s, &routine.id)[0].status, RunStatus::Failed);
}
