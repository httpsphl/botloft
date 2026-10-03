//! Tests for routines and their runs.

use botloft_core::ids::{MessageId, RoutineRunId};
use botloft_core::protocol::{Missed, Overlap, RoutineRun, RunStatus, SkipReason};

use super::*;
use crate::tests::Fixture;

fn routine(fx: &Fixture, next: i64) -> Routine {
    Routine {
        id: RoutineId::generate(),
        bot_id: fx.bots[0].id.clone(),
        name: "Morning".into(),
        prompt: "Summarize".into(),
        schedule: Schedule::Interval { minutes: 60 },
        timezone: "UTC".into(),
        overlap: Overlap::Skip,
        missed: Missed::RunOnce,
        enabled: true,
        next_run_at: Some(next),
        last_run: None,
        created_at: 0,
        updated_at: 0,
        archived_at: None,
    }
}

fn run(routine: &Routine, status: RunStatus, message: Option<MessageId>) -> RoutineRun {
    RoutineRun {
        id: RoutineRunId::generate(),
        routine_id: routine.id.clone(),
        scheduled_for: 100,
        status,
        reason: (status == RunStatus::Skipped).then_some(SkipReason::Overlap),
        skipped_count: 0,
        message_id: message,
        created_at: 100,
        finished_at: None,
        signal: None,
    }
}

#[test]
fn routines_come_due_and_keep_their_schedule() {
    let fx = Fixture::new();
    let soon = routine(&fx, 100);
    let later = routine(&fx, 500);
    fx.store.insert_routine(&soon).expect("insert");
    fx.store.insert_routine(&later).expect("insert");
    let due = fx.store.due_routines(200).expect("due");
    assert_eq!(due.len(), 1);
    assert_eq!(due[0].schedule, Schedule::Interval { minutes: 60 });
    assert_eq!(fx.store.next_routine_at().expect("next"), Some(100));

    let archived = fx
        .store
        .archive_routines_of(&fx.bots[0].id, 300)
        .expect("archive");
    assert_eq!(archived.len(), 2);
    assert!(fx.store.due_routines(1_000).expect("due").is_empty());
    assert!(fx.store.routines(None, false).expect("list").is_empty());
}

#[test]
fn a_run_is_saved_with_its_message_and_ends_once() {
    let fx = Fixture::new();
    let routine = routine(&fx, 100);
    fx.store.insert_routine(&routine).expect("insert");
    let (mut message, delivery) =
        crate::tests::message_to(&fx.crew.id, &fx.bots[0].id, "Summarize");
    message.routine_id = Some(routine.id.clone());
    let fired = run(&routine, RunStatus::Queued, Some(message.id.clone()));
    fx.store
        .fire_run(&fired, &message, &delivery)
        .expect("fire");
    assert_eq!(fx.store.open_runs(&routine.id).expect("open"), 1);
    assert_eq!(
        fx.store
            .message(&message.id)
            .expect("read")
            .map(|m| m.routine_id),
        Some(Some(routine.id.clone()))
    );
    assert_eq!(
        fx.store
            .run_of_message(&message.id)
            .expect("run")
            .map(|r| r.id),
        Some(fired.id.clone())
    );

    let done = fx
        .store
        .finish_run(&fired.id, RunStatus::Done, 150)
        .expect("finish")
        .expect("was open");
    assert_eq!(done.finished_at, Some(150));
    assert_eq!(
        fx.store
            .finish_run(&fired.id, RunStatus::Failed, 160)
            .expect("again"),
        None
    );

    let skipped = run(&routine, RunStatus::Skipped, None);
    fx.store.insert_run(&skipped).expect("skip");
    let listed = fx
        .store
        .routine(&routine.id)
        .expect("read")
        .expect("routine");
    assert_eq!(listed.last_run.map(|r| r.id), Some(skipped.id));
    assert_eq!(fx.store.runs(&routine.id, None, 10).expect("runs").len(), 2);
}
