//! `build`'s steps on crews that already exist (spec 36.8), for
//! `tools/list`.

use botloft_core::validate::{NAME_MAX_CHARS, ROLE_MAX_CHARS};
use serde_json::{Value, json};

use super::schema::{EFFORTS, MODELS, why};
use crate::tools::catalog::schedule_schema;
use crate::tools::routine::PROMPT_MAX;
use crate::tools::suggest::INSTRUCTIONS_MAX;

fn crew() -> Value {
    json!({ "type": "string", "description": "The name of an existing crew, as crews_overview gives it." })
}

fn agent() -> Value {
    json!({ "type": "string", "description": "The bot's handle, as crews_overview gives it, or \"chief\"." })
}

fn routine() -> Value {
    json!({ "type": "string", "description": "The routine's name, as crews_overview gives it." })
}

pub(super) fn change_agent() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": { "const": "change_agent" },
            "crew": crew(),
            "agent": agent(),
            "name": { "type": "string", "maxLength": NAME_MAX_CHARS, "description": "A new name; the handle changes with it." },
            "role": { "type": "string", "maxLength": ROLE_MAX_CHARS },
            "instructions": {
                "type": "string",
                "maxLength": INSTRUCTIONS_MAX,
                "description": "The bot's whole new instructions, written to it: they replace \
                    the old ones, which crews_overview shows with `crew`. Keep what still \
                    holds.",
            },
            "add_instructions": {
                "type": "string",
                "maxLength": INSTRUCTIONS_MAX,
                "description": "Instead of `instructions`: text added after the bot's \
                    instructions, under \"From the Maker:\".",
            },
            "model": { "type": "string", "enum": MODELS },
            "effort": { "type": "string", "enum": EFFORTS },
            "why": why(),
        },
        "required": ["op", "crew", "agent"],
        "additionalProperties": false,
    })
}

pub(super) fn archive_agent() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": { "const": "archive_agent" },
            "crew": crew(),
            "agent": agent(),
            "why": why(),
        },
        "required": ["op", "crew", "agent"],
        "additionalProperties": false,
    })
}

pub(super) fn change_routine() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": { "const": "change_routine" },
            "crew": crew(),
            "routine": routine(),
            "agent": { "type": "string", "description": "The handle of the routine's bot, when two routines have that name." },
            "name": { "type": "string", "maxLength": 80 },
            "prompt": { "type": "string", "maxLength": PROMPT_MAX, "description": "What the bot does each time; replaces the old text." },
            "schedule": schedule_schema(),
            "timezone": { "type": "string" },
            "enabled": { "type": "boolean", "description": "false turns the routine off, true back on." },
            "why": why(),
        },
        "required": ["op", "crew", "routine"],
        "additionalProperties": false,
    })
}

pub(super) fn delete_routine() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": { "const": "delete_routine" },
            "crew": crew(),
            "routine": routine(),
            "agent": { "type": "string", "description": "The handle of the routine's bot, when two routines have that name." },
            "why": why(),
        },
        "required": ["op", "crew", "routine"],
        "additionalProperties": false,
    })
}
