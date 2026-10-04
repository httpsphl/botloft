//! The tools that let a bot see and change its own routines (spec 20.12),
//! as `tools/list` describes them.

use serde_json::{Value, json};

use super::routine::PROMPT_MAX;

pub const MY_ROUTINES: &str = "my_routines";
pub const CHANGE_ROUTINE: &str = "change_routine";
pub const DELETE_ROUTINE: &str = "delete_routine";

/// Longest reason a bot may give for deleting a routine, in characters.
pub(super) const REASON_MAX: usize = 500;

pub(super) fn tools() -> [Value; 3] {
    let id = json!({
        "type": "string",
        "description": "The routine's id, as my_routines gives it.",
    });
    [
        json!({
            "name": MY_ROUTINES,
            "title": "My routines",
            "description": "Lists your routines (work done at set times or on a signal): id, \
                name, prompt, schedule, time zone, whether it is on and when it runs next.",
            "inputSchema": { "type": "object", "properties": {}, "additionalProperties": false },
        }),
        json!({
            "name": CHANGE_ROUTINE,
            "title": "Change a routine",
            "description": "Asks the owner to change one of your routines: give its id and only \
                what changes (name, prompt, schedule, time zone, or enabled to turn it off or \
                on). The owner sees the change in your chat, may adjust it, and approves or \
                declines it; the call waits for that. Say it changed only after the tool says it \
                did.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "routine_id": id,
                    "name": { "type": "string", "maxLength": 80 },
                    "prompt": { "type": "string", "maxLength": PROMPT_MAX },
                    "schedule": super::catalog::schedule_schema(),
                    "timezone": {
                        "type": "string",
                        "description": "IANA name, like America/Sao_Paulo.",
                    },
                    "overlap": { "type": "string", "enum": ["skip", "queue"] },
                    "missed": { "type": "string", "enum": ["run_once", "skip"] },
                    "enabled": { "type": "boolean", "description": "false turns it off, true on." },
                },
                "required": ["routine_id"],
                "additionalProperties": false,
            },
        }),
        json!({
            "name": DELETE_ROUTINE,
            "title": "Delete a routine",
            "description": "Asks the owner to delete one of your routines for good. The owner \
                approves or declines it in your chat; the call waits for that. To pause one \
                instead, use change_routine with enabled false.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "routine_id": id,
                    "reason": {
                        "type": "string",
                        "maxLength": REASON_MAX,
                        "description": "For the owner: why it is no longer needed.",
                    },
                },
                "required": ["routine_id"],
                "additionalProperties": false,
            },
        }),
    ]
}
