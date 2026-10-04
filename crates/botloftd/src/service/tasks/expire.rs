//! Deadlines (spec 9.4): an open task past its deadline becomes `expired`
//! and its requester is told. The courier runs this every cycle.

use botloft_core::ids::MessageId;
use botloft_core::protocol::{Message, MessageKind, SenderKind, Task, TaskStatus};
use botloft_store::Store;
use tracing::warn;

use crate::service::ApiResult;
use crate::service::bots;
use crate::service::messages::{announce, pending_delivery};
use crate::state::Daemon;

pub(crate) fn expire_overdue(daemon: &Daemon, store: &Store, now: i64) {
    let overdue = match store.overdue_tasks(now) {
        Ok(overdue) => overdue,
        Err(err) => {
            warn!("could not look for overdue tasks: {err}");
            return;
        }
    };
    for task in overdue {
        if let Err(err) = expire(daemon, store, &task, now) {
            warn!(task = %task.id, "could not expire a task: {err}");
        }
    }
}

fn expire(daemon: &Daemon, store: &Store, task: &Task, now: i64) -> ApiResult<()> {
    let assignee = bots::find(store, &task.assignee_bot_id)?.handle;
    let notice = Message {
        id: MessageId::generate(),
        crew_id: task.crew_id.clone(),
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: task.requester_bot_id.clone(),
        kind: MessageKind::System,
        body: format!(
            "Task {} for @{assignee} passed its deadline without a result. It stays open for a \
             late result; decide whether to wait, ask @{assignee} with send_message, or get it \
             done another way.",
            task.id
        ),
        task_id: Some(task.id.clone()),
        routine_id: None,
        question_id: None,
        reply_to: None,
        attachments: Vec::new(),
        created_at: now,
    };
    let delivery = pending_delivery(&notice);
    let open = [TaskStatus::Open];
    let expired = store.settle_task(
        &task.id,
        &open,
        TaskStatus::Expired,
        None,
        now,
        (&notice, &delivery),
    )?;
    if let Some((expired, item)) = expired {
        announce(daemon, Some(expired), &notice, delivery, item);
    }
    Ok(())
}
