//! Running a tool for a bot. Problems the bot can fix (an unknown handle, a
//! bad argument, the hop limit) come back as tool results with `isError`,
//! so the model reads them and corrects itself; only an unknown tool or a
//! malformed call is a JSON-RPC error.

use std::collections::HashMap;

use axum::http::StatusCode;
use botloft_core::envelope::due;
use botloft_core::ids::{BotId, TaskId};
use botloft_core::protocol::{BotsListParams, Task, TaskStatus, error_code};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use tracing::warn;

use super::Failure;
use super::catalog::{COMPLETE_TASK, CREW_ROSTER, MY_TASKS, SEND_MESSAGE};
use super::question::{self, ASK_OWNER};
use super::share::{self, SHARE_FILE};
use super::signal::{self, SEND_SIGNAL};
use crate::service::tasks::{self, BotMessage};
use crate::service::{ApiError, bots, lead};
use crate::state::Daemon;

/// What the bot reads when a tool fails.
type Outcome = Result<Value, String>;

pub(super) fn call(daemon: &Daemon, bot: &BotId, params: &Value) -> Result<Value, Failure> {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return Err(invalid("tools/call needs the name of a tool"));
    };
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let outcome = match name {
        CREW_ROSTER => roster(daemon, bot),
        SEND_MESSAGE => parse(arguments).and_then(|args| send(daemon, bot, args)),
        COMPLETE_TASK => parse(arguments).and_then(|args| complete(daemon, bot, args)),
        MY_TASKS => parse(arguments).and_then(|args| my_tasks(daemon, bot, args)),
        SHARE_FILE => parse(arguments).and_then(|args| share::share(daemon, bot, args)),
        ASK_OWNER => parse(arguments).and_then(|args| question::ask(daemon, bot, args)),
        SEND_SIGNAL => parse(arguments).and_then(|args| signal::send(daemon, bot, args)),
        other => return Err(invalid(&format!("Unknown tool: {other}"))),
    };
    Ok(tool_result(outcome))
}

/// A tool result: the value as JSON text, or the error the bot reads.
pub(super) fn tool_result(outcome: Outcome) -> Value {
    let (text, is_error) = match outcome {
        Ok(value) => (
            serde_json::to_string_pretty(&value).unwrap_or_default(),
            false,
        ),
        Err(message) => (message, true),
    };
    json!({
        "content": [{ "type": "text", "text": text }],
        "isError": is_error,
    })
}

fn invalid(message: &str) -> Failure {
    Failure::new(StatusCode::OK, error_code::INVALID_PARAMS, message)
}

pub(super) fn parse<T: DeserializeOwned>(arguments: Value) -> Result<T, String> {
    serde_json::from_value(arguments).map_err(|err| format!("invalid arguments: {err}"))
}

/// The message a bot may see. Internal errors can name paths; they go to
/// the log instead.
pub(super) fn explain(err: ApiError) -> String {
    match err {
        ApiError::Internal(_) | ApiError::Workspace(_) => {
            warn!(error = %err, "a tool call failed");
            "Botloft had an internal error; the owner can find it in the Botloft log".to_owned()
        }
        other => other.to_string(),
    }
}

fn roster(daemon: &Daemon, bot: &BotId) -> Outcome {
    let (crew, me) = bots::active(&daemon.store(), bot).map_err(explain)?;
    let list = BotsListParams {
        crew_id: Some(crew.id.clone()),
    };
    let crew_bots = bots::list(daemon, list).map_err(explain)?;
    let others: Vec<_> = crew_bots
        .iter()
        .filter(|other| other.id != me.id)
        .map(|other| {
            json!({
                "handle": other.handle,
                "name": other.name,
                "role": other.role,
                "state": other.state,
                "chief": lead::is_lead(&crew, &other.id),
            })
        })
        .collect();
    Ok(json!({
        "crew": crew.name,
        "you": me.handle,
        "you_lead": lead::is_lead(&crew, &me.id),
        "bots": others,
        "signals": signal::waited_for(daemon, bot)?,
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SendArgs {
    to: String,
    body: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    deadline_minutes: Option<u32>,
}

fn send(daemon: &Daemon, bot: &BotId, args: SendArgs) -> Outcome {
    let task = match args.kind.as_deref() {
        None | Some("note") => false,
        Some("task") => true,
        Some(_) => return Err("kind must be \"note\" or \"task\"".to_owned()),
    };
    if !task && args.deadline_minutes.is_some() {
        return Err("deadline_minutes only applies to kind \"task\"".to_owned());
    }
    let request = BotMessage {
        to: args.to,
        body: args.body,
        task,
        deadline_minutes: args.deadline_minutes,
    };
    let (message, task) = tasks::send(daemon, bot, request).map_err(explain)?;
    let to = bots::find(&daemon.store(), &message.to_bot_id).map_err(explain)?;
    let mut out = json!({
        "message_id": message.id,
        "note": format!(
            "Queued for @{}. Any answer arrives later as a new message; you can end your turn.",
            to.handle
        ),
    });
    if let Some(task) = task {
        out["task_id"] = json!(task.id);
        out["due"] = json!(due(task.deadline_at, daemon.clock.now_ms()));
    }
    Ok(out)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CompleteArgs {
    task_id: String,
    result: String,
    #[serde(default)]
    status: Option<String>,
}

fn complete(daemon: &Daemon, bot: &BotId, args: CompleteArgs) -> Outcome {
    let status = match args.status.as_deref() {
        None | Some("done") => TaskStatus::Done,
        Some("failed") => TaskStatus::Failed,
        Some(_) => return Err("status must be \"done\" or \"failed\"".to_owned()),
    };
    let task_id: TaskId = args.task_id.trim().parse().map_err(|_| {
        format!(
            "{} is not a task id; they look like tsk_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0",
            args.task_id.trim()
        )
    })?;
    let task = tasks::complete(daemon, bot, &task_id, &args.result, status).map_err(explain)?;
    let requester = bots::find(&daemon.store(), &task.requester_bot_id).map_err(explain)?;
    Ok(json!({
        "task_id": task.id,
        "status": task.status,
        "note": format!("@{} receives the result as a message.", requester.handle),
    }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MyTasksArgs {
    #[serde(default)]
    role: Option<String>,
}

fn my_tasks(daemon: &Daemon, bot: &BotId, args: MyTasksArgs) -> Outcome {
    let (want_assigned, want_requested) = match args.role.as_deref() {
        None => (true, true),
        Some("assigned") => (true, false),
        Some("requested") => (false, true),
        Some(_) => return Err("role must be \"assigned\" or \"requested\"".to_owned()),
    };
    let (assigned, requested) = tasks::mine(daemon, bot).map_err(explain)?;
    let store = daemon.store();
    let (crew, _) = bots::active(&store, bot).map_err(explain)?;
    let handles: HashMap<BotId, String> = store
        .bots(Some(&crew.id), true)
        .map_err(|err| explain(err.into()))?
        .into_iter()
        .map(|record| (record.id, record.handle))
        .collect();
    let now = daemon.clock.now_ms();
    let describe = |task: &Task| -> Result<Value, String> {
        let handle = |id: &BotId| handles.get(id).cloned().unwrap_or_default();
        let request = store
            .task_request(&task.id)
            .map_err(|err| explain(err.into()))?
            .map(|message| message.body);
        Ok(json!({
            "task_id": task.id,
            "from": handle(&task.requester_bot_id),
            "to": handle(&task.assignee_bot_id),
            "status": task.status,
            "due": due(task.deadline_at, now),
            "hops": task.hops,
            "request": request,
        }))
    };
    let mut out = json!({});
    if want_assigned {
        out["assigned"] = Value::Array(assigned.iter().map(&describe).collect::<Result<_, _>>()?);
    }
    if want_requested {
        out["requested"] = Value::Array(requested.iter().map(&describe).collect::<Result<_, _>>()?);
    }
    Ok(out)
}
