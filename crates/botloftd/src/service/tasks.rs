//! Messages and tasks between bots (spec 9.4): what the MCP tools do, the
//! `tasks.list` method and, in [`expire`], deadlines.

mod expire;

use std::time::Duration;

use botloft_core::ids::{BotId, MessageId, TaskId};
use botloft_core::protocol::{
    Crew, Message, MessageKind, SenderKind, Task, TaskStatus, TasksListParams,
};
use botloft_core::{slug, validate};
use botloft_store::{BotRecord, Store, TaskFilter};

pub(crate) use self::expire::expire_overdue;
use super::messages::{announce, pending_delivery, post};
use super::{ApiError, ApiResult, bots, crews};
use crate::clock;
use crate::config::Config;
use crate::state::Daemon;

/// Longest deadline a bot may set: a week.
pub const MAX_DEADLINE_MINUTES: u32 = 7 * 24 * 60;
/// Tasks that still expect a result.
const UNFINISHED: [TaskStatus; 2] = [TaskStatus::Open, TaskStatus::Expired];

#[derive(Debug, Clone)]
pub struct TaskSettings {
    /// Longest chain of tasks, counting the first one.
    pub max_hops: u32,
    pub default_deadline: Duration,
}

impl TaskSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            max_hops: config.tasks.max_hops.max(1),
            default_deadline: Duration::from_secs(
                u64::from(config.tasks.default_deadline_minutes.max(1)) * 60,
            ),
        }
    }
}

/// `send_message` as a bot calls it.
#[derive(Debug, Clone)]
pub struct BotMessage {
    /// Handle of a bot in the sender's crew, with or without `@`.
    pub to: String,
    pub body: String,
    pub task: bool,
    pub deadline_minutes: Option<u32>,
}

pub fn list(daemon: &Daemon, params: TasksListParams) -> ApiResult<Vec<Task>> {
    let store = daemon.store();
    if let Some(crew) = &params.crew_id {
        crews::find(&store, crew)?;
    }
    let statuses: Vec<_> = params.status.into_iter().collect();
    Ok(store.tasks(TaskFilter {
        crew: params.crew_id.as_ref(),
        statuses: &statuses,
        ..TaskFilter::default()
    })?)
}

/// A note or a task from one bot to another of its crew. A task started
/// while the sender works on another one continues that chain (spec 9.4).
pub fn send(
    daemon: &Daemon,
    sender: &BotId,
    request: BotMessage,
) -> ApiResult<(Message, Option<Task>)> {
    let body = validate::message("body", &request.body)?;
    let store = daemon.store();
    let (crew, from) = bots::active(&store, sender)?;
    let to = recipient(&store, &crew, &from, &request.to)?;
    let now = daemon.clock.now_ms();
    let task = if request.task {
        Some(new_task(
            daemon,
            &store,
            &crew,
            &from,
            &to,
            request.deadline_minutes,
            now,
        )?)
    } else {
        None
    };
    let message = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::Bot,
        from_bot_id: Some(from.id),
        to_bot_id: to.id,
        kind: if task.is_some() {
            MessageKind::Task
        } else {
            MessageKind::Note
        },
        body,
        task_id: task.as_ref().map(|task| task.id.clone()),
        routine_id: None,
        attachments: Vec::new(),
        created_at: now,
    };
    let message = post(daemon, &store, message, task.clone())?;
    Ok((message, task))
}

fn recipient(store: &Store, crew: &Crew, from: &BotRecord, to: &str) -> ApiResult<BotRecord> {
    let handle = slug::slugify(to.trim().trim_start_matches('@'), "");
    let found = match handle.as_str() {
        "" => None,
        handle => store.active_bot_by_handle(&crew.id, handle)?,
    };
    let Some(id) = found else {
        return Err(ApiError::NotFound(format!(
            "no bot in your crew answers to @{handle}; call crew_roster to see who does"
        )));
    };
    if id == from.id {
        return Err(ApiError::validation(
            "you cannot send a message to yourself",
        ));
    }
    bots::find(store, &id)
}

fn new_task(
    daemon: &Daemon,
    store: &Store,
    crew: &Crew,
    from: &BotRecord,
    to: &BotRecord,
    deadline_minutes: Option<u32>,
    now: i64,
) -> ApiResult<Task> {
    let settings = &daemon.tasks;
    let deadline = match deadline_minutes {
        None => settings.default_deadline,
        Some(minutes @ 1..=MAX_DEADLINE_MINUTES) => Duration::from_secs(u64::from(minutes) * 60),
        Some(_) => {
            return Err(ApiError::validation(format!(
                "deadline_minutes must be between 1 and {MAX_DEADLINE_MINUTES}"
            )));
        }
    };
    // The sender's deepest unfinished chain; the newest one on a tie.
    let working_on = store.tasks(TaskFilter {
        assignee: Some(&from.id),
        statuses: &[TaskStatus::Open],
        ..TaskFilter::default()
    })?;
    let parent = working_on.iter().rev().max_by_key(|task| task.hops);
    let hops = parent.map_or(1, |task| task.hops + 1);
    let origin = parent.map(|task| {
        task.origin_task_id
            .clone()
            .unwrap_or_else(|| task.id.clone())
    });
    if hops > settings.max_hops {
        let origin = origin
            .as_ref()
            .map_or_else(String::new, |id| format!(" that started with {id}"));
        return Err(ApiError::validation(format!(
            "this task would be step {hops} of a chain of tasks{origin}, and the limit is {}. \
             Do the work yourself, or finish your own task with complete_task and explain what is missing.",
            settings.max_hops
        )));
    }
    Ok(Task {
        id: TaskId::generate(),
        crew_id: crew.id.clone(),
        requester_bot_id: from.id.clone(),
        assignee_bot_id: to.id.clone(),
        status: TaskStatus::Open,
        deadline_at: clock::after(now, deadline),
        hops,
        origin_task_id: origin,
        result: None,
        created_at: now,
        updated_at: now,
    })
}

/// The assignee reports the outcome; the requester gets it as a message.
/// A task past its deadline can still be completed.
pub fn complete(
    daemon: &Daemon,
    bot: &BotId,
    task_id: &TaskId,
    result: &str,
    status: TaskStatus,
) -> ApiResult<Task> {
    if !matches!(status, TaskStatus::Done | TaskStatus::Failed) {
        return Err(ApiError::validation("status must be done or failed"));
    }
    let result = validate::message("result", result)?;
    let store = daemon.store();
    let (crew, me) = bots::active(&store, bot)?;
    let task = store
        .task(task_id)?
        .filter(|task| task.crew_id == crew.id)
        .ok_or_else(|| ApiError::NotFound(format!("task {task_id} does not exist")))?;
    if task.assignee_bot_id != me.id {
        return Err(ApiError::Conflict(format!(
            "task {task_id} is not assigned to you; only its assignee can complete it"
        )));
    }
    if !UNFINISHED.contains(&task.status) {
        return Err(ApiError::Conflict(format!(
            "task {task_id} is already {}",
            task.status
        )));
    }
    let now = daemon.clock.now_ms();
    let report = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::Bot,
        from_bot_id: Some(me.id),
        to_bot_id: task.requester_bot_id.clone(),
        kind: MessageKind::Result,
        body: result.clone(),
        task_id: Some(task.id.clone()),
        routine_id: None,
        attachments: Vec::new(),
        created_at: now,
    };
    let delivery = pending_delivery(&report);
    let (settled, item) = store
        .settle_task(
            &task.id,
            &UNFINISHED,
            status,
            Some(&result),
            now,
            (&report, &delivery),
        )?
        .ok_or_else(|| {
            ApiError::Conflict(format!("task {task_id} changed meanwhile; check my_tasks"))
        })?;
    announce(daemon, Some(settled.clone()), &report, delivery, item);
    Ok(settled)
}

/// Unfinished tasks assigned to the bot and requested by it.
pub fn mine(daemon: &Daemon, bot: &BotId) -> ApiResult<(Vec<Task>, Vec<Task>)> {
    let store = daemon.store();
    bots::active(&store, bot)?;
    let assigned = store.tasks(TaskFilter {
        assignee: Some(bot),
        statuses: &UNFINISHED,
        ..TaskFilter::default()
    })?;
    let requested = store.tasks(TaskFilter {
        requester: Some(bot),
        statuses: &UNFINISHED,
        ..TaskFilter::default()
    })?;
    Ok((assigned, requested))
}
