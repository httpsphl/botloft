use botloft_core::protocol::MessageKind;

use super::*;
use crate::tests::{Fixture, message_to};

/// Bot 0 asks bot 1, due at `deadline_at`.
fn ask(fx: &Fixture, deadline_at: i64) -> Task {
    let task = Task {
        id: TaskId::generate(),
        crew_id: fx.crew.id.clone(),
        requester_bot_id: fx.bots[0].id.clone(),
        assignee_bot_id: fx.bots[1].id.clone(),
        status: TaskStatus::Open,
        deadline_at,
        hops: 1,
        origin_task_id: None,
        result: None,
        created_at: 0,
        updated_at: 0,
    };
    let (mut message, delivery) = message_to(&fx.crew.id, &fx.bots[1].id, "do it");
    message.kind = MessageKind::Task;
    message.task_id = Some(task.id.clone());
    fx.store
        .insert_message(&message, &delivery, Some(&task))
        .expect("insert");
    task
}

#[test]
fn a_task_is_saved_with_the_message_that_asks_for_it() {
    let fx = Fixture::new();
    let task = ask(&fx, 7_200_000);
    assert_eq!(fx.store.task(&task.id).expect("read"), Some(task.clone()));
    let request = fx.store.task_request(&task.id).expect("read");
    assert_eq!(request.map(|m| m.body), Some("do it".to_owned()));
}

#[test]
fn tasks_filter_by_bot_and_status() {
    let fx = Fixture::new();
    let first = ask(&fx, 100);
    let second = ask(&fx, 200);
    let open = [TaskStatus::Open];
    let assigned = fx
        .store
        .tasks(TaskFilter {
            assignee: Some(&fx.bots[1].id),
            statuses: &open,
            ..TaskFilter::default()
        })
        .expect("list");
    let ids: Vec<_> = assigned.iter().map(|t| t.id.clone()).collect();
    assert_eq!(ids, vec![second.id.clone(), first.id.clone()]);
    let requested_by_the_assignee = fx
        .store
        .tasks(TaskFilter {
            requester: Some(&fx.bots[1].id),
            ..TaskFilter::default()
        })
        .expect("list");
    assert!(requested_by_the_assignee.is_empty());
    let overdue = fx.store.overdue_tasks(150).expect("overdue");
    let overdue: Vec<_> = overdue.into_iter().map(|t| t.id).collect();
    assert_eq!(overdue, vec![first.id]);
}

#[test]
fn settling_updates_the_task_and_saves_the_report_together() {
    let fx = Fixture::new();
    let task = ask(&fx, 100);
    let from = [TaskStatus::Open, TaskStatus::Expired];
    let (report, delivery) = message_to(&fx.crew.id, &fx.bots[0].id, "done: 42");
    let done = fx
        .store
        .settle_task(
            &task.id,
            &from,
            TaskStatus::Done,
            Some("42"),
            500,
            (&report, &delivery),
        )
        .expect("settle")
        .expect("was open");
    assert_eq!(done.status, TaskStatus::Done);
    assert_eq!(done.result.as_deref(), Some("42"));
    assert_eq!(done.updated_at, 500);
    assert!(fx.store.message(&report.id).expect("read").is_some());

    let (again, again_delivery) = message_to(&fx.crew.id, &fx.bots[0].id, "twice");
    let twice = fx
        .store
        .settle_task(
            &task.id,
            &from,
            TaskStatus::Failed,
            None,
            600,
            (&again, &again_delivery),
        )
        .expect("settle");
    assert_eq!(twice, None, "a done task stays done");
    assert!(
        fx.store.message(&again.id).expect("read").is_none(),
        "and nothing is reported"
    );
}
