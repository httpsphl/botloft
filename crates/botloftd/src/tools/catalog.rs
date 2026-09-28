//! The tools `tools/list` returns (spec 10), in a fixed order so clients
//! can cache the list.

use serde_json::{Value, json};

use crate::service::tasks::MAX_DEADLINE_MINUTES;

pub const CREW_ROSTER: &str = "crew_roster";
pub const SEND_MESSAGE: &str = "send_message";
pub const COMPLETE_TASK: &str = "complete_task";
pub const MY_TASKS: &str = "my_tasks";

pub fn tools() -> Value {
    json!([
        {
            "name": CREW_ROSTER,
            "title": "Crew roster",
            "description": "Lists the other bots of your crew with their handle, name, role and \
                current state (idle, busy, offline...). Use it to find who can help and the \
                handle to send a message to.",
            "inputSchema": { "type": "object", "additionalProperties": false },
            "annotations": { "readOnlyHint": true },
        },
        {
            "name": SEND_MESSAGE,
            "title": "Send a message",
            "description": "Sends a note, or a task, to another bot of your crew. It is queued and \
                delivered when that bot can read it; do not wait for the answer, it arrives later \
                as a new message. With kind \"task\" the other bot must report back with \
                complete_task before the deadline.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "to": {
                        "type": "string",
                        "description": "Handle of the bot, as crew_roster shows it (\"revisor\" or \"@revisor\").",
                    },
                    "body": {
                        "type": "string",
                        "description": "The message. Write it so it makes sense on its own.",
                    },
                    "kind": {
                        "type": "string",
                        "enum": ["note", "task"],
                        "description": "\"note\" (default) informs; \"task\" asks for work and a result.",
                    },
                    "deadline_minutes": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": MAX_DEADLINE_MINUTES,
                        "description": "For a task: minutes until it is due. Defaults to the crew setting.",
                    },
                },
                "required": ["to", "body"],
                "additionalProperties": false,
            },
        },
        {
            "name": COMPLETE_TASK,
            "title": "Complete a task",
            "description": "Reports the result of a task assigned to you. The bot that asked \
                receives the result as a message. Also works after the deadline.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "task_id": {
                        "type": "string",
                        "description": "The id from the task message, like tsk_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0.",
                    },
                    "result": {
                        "type": "string",
                        "description": "What you did and found, or why it could not be done.",
                    },
                    "status": {
                        "type": "string",
                        "enum": ["done", "failed"],
                        "description": "\"done\" (default) or \"failed\".",
                    },
                },
                "required": ["task_id", "result"],
                "additionalProperties": false,
            },
        },
        {
            "name": MY_TASKS,
            "title": "My tasks",
            "description": "Lists the unfinished tasks assigned to you and those you asked other \
                bots for, with who, what was asked and the time left.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "role": {
                        "type": "string",
                        "enum": ["assigned", "requested"],
                        "description": "Only tasks assigned to you, or only those you requested. Both when absent.",
                    },
                },
                "additionalProperties": false,
            },
            "annotations": { "readOnlyHint": true },
        },
    ])
}
