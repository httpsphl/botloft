//! The tools `tools/list` returns (spec 10), in a fixed order so clients
//! can cache the list.

use serde_json::{Value, json};

use super::routine::PROMPT_MAX;
use super::suggest::{INSTRUCTIONS_MAX, REASON_MAX};
use crate::service::tasks::MAX_DEADLINE_MINUTES;
use botloft_core::validate::{NAME_MAX_CHARS, ROLE_MAX_CHARS};

pub const CREW_ROSTER: &str = "crew_roster";
pub const SEND_MESSAGE: &str = "send_message";
pub const COMPLETE_TASK: &str = "complete_task";
pub const MY_TASKS: &str = "my_tasks";
/// The chief's tool (spec 10.2); Claude Code names it `mcp__botloft__suggest_bot`.
pub const SUGGEST_BOT: &str = "suggest_bot";
/// Any bot's tool to ask for a routine (spec 20.12).
pub const SCHEDULE_ROUTINE: &str = "schedule_routine";
/// Claude Code's `--permission-prompt-tool` (spec 10.1).
pub const PERMISSION_PROMPT: &str = "permission_prompt";

pub fn tools() -> Value {
    let mut tools = crew_tools();
    if let Value::Array(list) = &mut tools {
        // Before permission_prompt, which stays last.
        let at = list.len() - 1;
        let more = std::iter::once(super::share::tool()).chain(super::browser_catalog::tools());
        list.splice(at..at, more);
    }
    tools
}

fn crew_tools() -> Value {
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
        {
            "name": SUGGEST_BOT,
            "title": "Suggest a new bot",
            "description": "Only for the crew's chief (crew_roster marks it). Suggests a new bot \
                for work that needs a specialist the crew lacks. The owner sees the suggestion in \
                your chat, may change it, and approves or declines it; the call waits for that. \
                Once approved the bot starts at once: give it work with send_message. If the owner \
                declines, do the work yourself or with the bots you have.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "maxLength": NAME_MAX_CHARS,
                        "description": "A short name, like \"Designer\". Its handle comes from it.",
                    },
                    "role": {
                        "type": "string",
                        "maxLength": ROLE_MAX_CHARS,
                        "description": "One line: what the bot does in the crew.",
                    },
                    "instructions": {
                        "type": "string",
                        "maxLength": INSTRUCTIONS_MAX,
                        "description": "Written to the new bot: what it works on, how, what it \
                            hands back and when to ask.",
                    },
                    "model": {
                        "type": "string",
                        "enum": ["default", "fable", "opus", "sonnet", "haiku"],
                        "description": "\"haiku\" for simple, repetitive work; \"sonnet\" for most \
                            work; \"opus\" or \"fable\" for the hardest reasoning. The owner's \
                            plan default when absent.",
                    },
                    "reason": {
                        "type": "string",
                        "maxLength": REASON_MAX,
                        "description": "For the owner: why the crew needs this bot now.",
                    },
                },
                "required": ["name", "role", "instructions", "reason"],
                "additionalProperties": false,
            },
        },
        {
            "name": SCHEDULE_ROUTINE,
            "title": "Schedule a routine",
            "description": "Asks the owner for a routine: work done at set times, such as every \
                weekday at 08:00 or every 2 hours. Each time, the bot gets your prompt as a \
                message, even after restarts. The owner sees the routine in your chat, may change \
                it, and approves or declines it; the call waits for that. Approved, it shows among \
                the bot's routines in the Botloft app. This is the only way to schedule work.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "maxLength": 80,
                        "description": "A short name the owner recognizes, like \"Morning inbox\".",
                    },
                    "prompt": {
                        "type": "string",
                        "maxLength": PROMPT_MAX,
                        "description": "What to do each time, written to the bot that runs it. \
                            Make it complete on its own: it arrives with no other context.",
                    },
                    "schedule": {
                        "description": "When it runs, in the owner's time zone. Runs must be at \
                            least 5 minutes apart.",
                        "oneOf": [
                            {
                                "type": "object",
                                "properties": {
                                    "kind": { "const": "weekly" },
                                    "days": {
                                        "type": "array",
                                        "items": { "type": "integer", "minimum": 1, "maximum": 7 },
                                        "minItems": 1,
                                        "description": "1 = Monday … 7 = Sunday.",
                                    },
                                    "time": { "type": "string", "description": "HH:MM, 24-hour." },
                                },
                                "required": ["kind", "days", "time"],
                                "additionalProperties": false,
                            },
                            {
                                "type": "object",
                                "properties": {
                                    "kind": { "const": "interval" },
                                    "minutes": { "type": "integer", "minimum": 5, "maximum": 10_080 },
                                },
                                "required": ["kind", "minutes"],
                                "additionalProperties": false,
                            },
                            {
                                "type": "object",
                                "properties": {
                                    "kind": { "const": "cron" },
                                    "expr": {
                                        "type": "string",
                                        "description": "5 fields: minute hour day month weekday.",
                                    },
                                },
                                "required": ["kind", "expr"],
                                "additionalProperties": false,
                            },
                        ],
                    },
                    "bot": {
                        "type": "string",
                        "description": "Handle of another bot of your crew the routine is for. \
                            Leave it out for a routine of your own.",
                    },
                },
                "required": ["name", "prompt", "schedule"],
                "additionalProperties": false,
            },
        },
        {
            "name": PERMISSION_PROMPT,
            "title": "Permission prompt",
            "description": "Used by Claude Code to ask the owner before a tool runs. Never call \
                it yourself: it decides nothing about what you do.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "tool_name": { "type": "string" },
                    "input": { "type": "object" },
                    "tool_use_id": { "type": "string" },
                },
                "required": ["tool_name", "input"],
            },
        },
    ])
}
