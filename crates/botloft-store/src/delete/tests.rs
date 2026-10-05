//! Tests for deleting bots and crews.

use botloft_core::ids::{ApprovalId, AttachmentId, ChatItemId, RoutineId, RoutineRunId, TaskId};
use botloft_core::protocol::{
    Approval, ApprovalItem, ApprovalStatus, Attachment, ChatBody, ChatItem, Message, MessageKind,
    Missed, Overlap, Routine, RoutineRun, RunStatus, Schedule, SenderKind, Task, TaskStatus,
};

use super::*;
use crate::tests::{Fixture, message_to};
use crate::{ApprovalRecord, MessageFilter, TaskFilter};

/// A message from bot `from` to bot `to` of the fixture.
fn from_bot(fx: &Fixture, from: usize, to: usize, body: &str) -> Message {
    let (mut message, delivery) = message_to(&fx.crew.id, &fx.bots[to].id, body);
    message.from_kind = SenderKind::Bot;
    message.from_bot_id = Some(fx.bots[from].id.clone());
    fx.store
        .insert_message(&message, &delivery, None)
        .expect("message");
    message
}

/// Bot `from` asks bot `to` for a task.
fn ask(fx: &Fixture, from: usize, to: usize, origin: Option<&Task>) -> Task {
    let task = Task {
        id: TaskId::generate(),
        crew_id: fx.crew.id.clone(),
        requester_bot_id: fx.bots[from].id.clone(),
        assignee_bot_id: fx.bots[to].id.clone(),
        status: TaskStatus::Open,
        deadline_at: 1_000,
        hops: 1,
        origin_task_id: origin.map(|task| task.id.clone()),
        result: None,
        created_at: 0,
        updated_at: 0,
    };
    let (mut message, delivery) = message_to(&fx.crew.id, &fx.bots[to].id, "do it");
    message.from_kind = SenderKind::Bot;
    message.from_bot_id = Some(fx.bots[from].id.clone());
    message.kind = MessageKind::Task;
    message.task_id = Some(task.id.clone());
    fx.store
        .insert_message(&message, &delivery, Some(&task))
        .expect("task");
    task
}

/// Everything a bot can own, for bot 0 (scout): a note from the owner with
/// a file, an approval, a browser site and a routine that ran once.
struct Owned {
    attachment: AttachmentId,
    approval: ApprovalId,
    routine: RoutineId,
}

fn fill(fx: &Fixture) -> Owned {
    let scout = &fx.bots[0].id;
    let (mut note, delivery) = message_to(&fx.crew.id, scout, "see attached");
    let attachment = AttachmentId::generate();
    note.attachments = vec![Attachment {
        id: attachment.clone(),
        name: "plan.pdf".into(),
        media_type: "application/pdf".into(),
        size: 2048,
        path: "attachments/2026-09-30/plan.pdf".into(),
    }];
    fx.store
        .insert_message(&note, &delivery, None)
        .expect("note");

    let approval = Approval {
        id: ApprovalId::generate(),
        bot_id: scout.clone(),
        tool_name: "Bash".into(),
        summary: "ls".into(),
        input: r#"{"command":"ls"}"#.into(),
        status: ApprovalStatus::Pending,
        note: None,
        created_at: 10,
        answered_at: None,
    };
    let item = ChatItem {
        id: ChatItemId::generate(),
        bot_id: scout.clone(),
        body: ChatBody::Approval(ApprovalItem {
            approval_id: approval.id.clone(),
            tool_name: approval.tool_name.clone(),
            summary: approval.summary.clone(),
            explanation: None,
            input: approval.input.clone(),
            status: ApprovalStatus::Pending,
            note: None,
            always: None,
        }),
        created_at: 10,
        updated_at: 10,
    };
    fx.store.insert_chat_item(&item).expect("item");
    let record = ApprovalRecord {
        approval,
        chat_item_id: item.id,
        tool_use_id: "toolu_1".into(),
    };
    fx.store.insert_approval(&record).expect("approval");
    fx.store
        .allow_browser_site(scout, "example.com", 10)
        .expect("site");
    let always = botloft_core::protocol::AllowScope {
        tool_name: "Bash".into(),
        kind: botloft_core::protocol::AllowKind::Command,
        value: "git status".into(),
    };
    fx.store.add_allow_rule(scout, &always, 10).expect("rule");
    fx.store
        .grant_desktop_app(
            scout,
            "C:/Windows/notepad.exe",
            "Notepad",
            botloft_core::protocol::DesktopLevel::See,
            10,
        )
        .expect("desktop");

    let routine = Routine {
        id: RoutineId::generate(),
        bot_id: scout.clone(),
        name: "Morning".into(),
        prompt: "Summarize".into(),
        schedule: Schedule::Interval { minutes: 60 },
        timezone: "UTC".into(),
        overlap: Overlap::Skip,
        missed: Missed::RunOnce,
        enabled: true,
        next_run_at: Some(100),
        last_run: None,
        created_at: 0,
        updated_at: 0,
        archived_at: None,
    };
    fx.store.insert_routine(&routine).expect("routine");
    let (mut fired, delivery) = message_to(&fx.crew.id, scout, "Summarize");
    fired.from_kind = SenderKind::System;
    fired.routine_id = Some(routine.id.clone());
    let run = RoutineRun {
        id: RoutineRunId::generate(),
        routine_id: routine.id.clone(),
        scheduled_for: 100,
        status: RunStatus::Queued,
        reason: None,
        skipped_count: 0,
        message_id: Some(fired.id.clone()),
        created_at: 100,
        finished_at: None,
        signal: None,
    };
    fx.store.fire_run(&run, &fired, &delivery).expect("run");

    // A question the owner answered.
    let question = crate::questions::tests::ask(&fx.store, fx, scout, "Which client?", 120);
    let (mut answer, delivery) = message_to(&fx.crew.id, scout, "Acme");
    answer.question_id = Some(question.id.clone());
    fx.store
        .answer_question(&question.id, &answer, &delivery)
        .expect("answer")
        .expect("open");
    Owned {
        attachment,
        approval: record.approval.id,
        routine: routine.id,
    }
}

fn count(store: &Store, table: &str) -> i64 {
    store
        .conn
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .expect("count")
}

/// Rows that point at something that is gone.
fn dangling(store: &Store) -> usize {
    let mut stmt = store
        .conn
        .prepare("PRAGMA foreign_key_check")
        .expect("check");
    stmt.query_map([], |_| Ok(())).expect("rows").count()
}

#[test]
fn a_deleted_bot_takes_what_was_its_own() {
    let fx = Fixture::new();
    let (scout, writer) = (fx.bots[0].id.clone(), fx.bots[1].id.clone());
    let owned = fill(&fx);
    let mut crew = fx.crew.clone();
    crew.lead_bot_id = Some(scout.clone());
    fx.store.update_crew(&crew).expect("chief");

    assert!(fx.store.delete_bot(&scout).expect("delete"));

    assert_eq!(fx.store.bot(&scout).expect("bot"), None);
    assert!(fx.store.bot(&writer).expect("bot").is_some());
    assert!(
        fx.store
            .chat_history(&scout, None, 50)
            .expect("chat")
            .is_empty()
    );
    assert!(
        fx.store
            .deliveries(None, Some(&scout))
            .expect("deliveries")
            .is_empty()
    );
    assert_eq!(fx.store.attachment(&owned.attachment).expect("file"), None);
    assert_eq!(fx.store.approval(&owned.approval).expect("approval"), None);
    assert_eq!(fx.store.routine(&owned.routine).expect("routine"), None);
    assert!(
        fx.store
            .runs(&owned.routine, None, 10)
            .expect("runs")
            .is_empty()
    );
    assert!(fx.store.browser_sites(&scout).expect("sites").is_empty());
    assert!(fx.store.allow_rules(&scout).expect("rules").is_empty());
    assert!(fx.store.desktop_grants(&scout).expect("desktop").is_empty());
    assert_eq!(count(&fx.store, "questions"), 0);
    let crew = fx.store.crew(&fx.crew.id).expect("crew").expect("kept");
    assert_eq!(crew.lead_bot_id, None);
    assert_eq!(dangling(&fx.store), 0);

    assert!(!fx.store.delete_bot(&scout).expect("again"), "already gone");
}

#[test]
fn what_a_deleted_bot_sent_stays_without_a_sender() {
    let fx = Fixture::new();
    let (scout, writer) = (fx.bots[0].id.clone(), fx.bots[1].id.clone());
    let note = from_bot(&fx, 0, 1, "the draft is in shared/");
    let reply = from_bot(&fx, 1, 0, "thanks");
    let asked = ask(&fx, 0, 1, None);
    let given = ask(&fx, 1, 0, None);

    assert!(fx.store.delete_bot(&scout).expect("delete"));

    let kept = fx.store.message(&note.id).expect("read").expect("kept");
    assert_eq!(kept.from_kind, SenderKind::Bot);
    assert_eq!(kept.from_bot_id, None);
    assert_eq!(fx.store.message(&reply.id).expect("read"), None);
    // The writer's chat still shows what it received.
    let chat = fx.store.chat_history(&writer, None, 50).expect("chat");
    assert_eq!(chat.len(), 2);

    // Tasks to and from the deleted bot go; the request the writer got
    // stays as a message, without its task.
    assert_eq!(fx.store.task(&asked.id).expect("task"), None);
    assert_eq!(fx.store.task(&given.id).expect("task"), None);
    let request = fx
        .store
        .messages(MessageFilter {
            bot: Some(&writer),
            limit: 10,
            ..MessageFilter::default()
        })
        .expect("messages");
    assert_eq!(request.len(), 2);
    assert!(request.iter().all(|message| message.task_id.is_none()));
    assert_eq!(dangling(&fx.store), 0);
}

#[test]
fn a_chain_of_tasks_loses_only_the_deleted_link() {
    let mut fx = Fixture::new();
    let mut editor = fx.bots[1].clone();
    editor.id = botloft_core::ids::BotId::generate();
    editor.handle = "editor".into();
    editor.slug = "editor".into();
    fx.store.insert_bot(&editor).expect("third bot");
    fx.bots.push(editor);

    // Scout asks the writer, who asks the editor.
    let first = ask(&fx, 0, 1, None);
    let second = ask(&fx, 1, 2, Some(&first));

    assert!(fx.store.delete_bot(&fx.bots[0].id).expect("delete"));

    let left = fx.store.tasks(TaskFilter::default()).expect("tasks");
    assert_eq!(left.len(), 1);
    assert_eq!(left[0].id, second.id);
    assert_eq!(left[0].origin_task_id, None);
    assert_eq!(dangling(&fx.store), 0);
}

#[test]
fn a_deleted_crew_takes_its_bots_and_leaves_other_crews_alone() {
    let fx = Fixture::new();
    fill(&fx);
    from_bot(&fx, 0, 1, "hello");
    ask(&fx, 1, 0, None);
    // An archived bot of the crew goes too.
    let mut archived = fx.bots[1].clone();
    archived.archived_at = Some(5);
    fx.store.update_bot(&archived).expect("archive");

    let other = Fixture::new().crew;
    let other = botloft_core::protocol::Crew {
        slug: "other".into(),
        ..other
    };
    fx.store.insert_crew(&other).expect("other crew");

    assert!(fx.store.delete_crew(&fx.crew.id).expect("delete"));

    assert_eq!(fx.store.crews(true).expect("crews"), vec![other]);
    for table in [
        "bots",
        "messages",
        "deliveries",
        "attachments",
        "chat_items",
        "approvals",
        "tasks",
        "routines",
        "routine_runs",
        "browser_sites",
        "allow_rules",
        "desktop_grants",
        "crew_access",
        "turn_costs",
    ] {
        assert_eq!(count(&fx.store, table), 0, "{table}");
    }
    assert_eq!(dangling(&fx.store), 0);
    assert!(!fx.store.delete_crew(&fx.crew.id).expect("again"));
}
