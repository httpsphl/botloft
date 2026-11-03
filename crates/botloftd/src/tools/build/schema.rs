//! `build`'s entry in `tools/list` (spec 36.4).

use botloft_core::validate::{NAME_MAX_CHARS, ROLE_MAX_CHARS};
use serde_json::{Value, json};

use super::BUILD;
use super::plan::{MEMORY_MAX, STEP_WHY_MAX, STEPS_MAX, TITLE_MAX, WHY_MAX};
use crate::tools::catalog::{schedule_schema, template_ids};
use crate::tools::routine::PROMPT_MAX;
use crate::tools::suggest::INSTRUCTIONS_MAX;

const MODELS: [&str; 5] = ["default", "fable", "opus", "sonnet", "haiku"];
const EFFORTS: [&str; 6] = ["default", "low", "medium", "high", "xhigh", "max"];

pub(crate) fn tool() -> Value {
    json!({
        "name": BUILD,
        "title": "Build crews for the owner",
        "description": "Only for the Maker. Builds a plan in one go: new crews with their \
            chief and memory, their bots and their routines. The owner sees the plan in your \
            chat, may change or leave out any part, and builds it; the call waits for that. \
            What was built can be undone for a week. Steps name a crew of the plan by its \
            `ref`.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "maxLength": TITLE_MAX,
                    "description": "One line for the owner, in their language: what this plan \
                        builds.",
                },
                "why": {
                    "type": "string",
                    "maxLength": WHY_MAX,
                    "description": "For the owner: what they told or sent you that led here.",
                },
                "steps": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": STEPS_MAX,
                    "items": { "oneOf": [create_crew(), add_agent(), add_routine()] },
                },
            },
            "required": ["title", "why", "steps"],
            "additionalProperties": false,
        },
    })
}

fn why() -> Value {
    json!({
        "type": "string",
        "maxLength": STEP_WHY_MAX,
        "description": "For the owner: why this step.",
    })
}

fn create_crew() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": { "const": "create_crew" },
            "ref": { "type": "string", "description": "A short name the other steps use for this crew." },
            "name": { "type": "string", "maxLength": NAME_MAX_CHARS, "description": "The crew's name, in the owner's language." },
            "goal": {
                "type": "string",
                "maxLength": INSTRUCTIONS_MAX,
                "description": "The crew's goal, written to its chief: it becomes the chief's \
                    instructions.",
            },
            "memory": {
                "type": "string",
                "maxLength": MEMORY_MAX,
                "description": "The start of the crew's memory, read by every bot of it: what \
                    is true for all of them (the business, the clients, the tone, the tools the \
                    owner uses, what never to do).",
            },
            "lead_name": { "type": "string", "maxLength": NAME_MAX_CHARS, "description": "The chief's name; \"Chief\" in the owner's language when absent." },
            "lead_role": { "type": "string", "maxLength": ROLE_MAX_CHARS },
            "lead_model": { "type": "string", "enum": MODELS },
            "lead_effort": { "type": "string", "enum": EFFORTS },
            "why": why(),
        },
        "required": ["op", "ref", "name", "goal"],
        "additionalProperties": false,
    })
}

fn add_agent() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": { "const": "add_agent" },
            "crew": { "type": "string", "description": "The `ref` of a crew of this plan." },
            "name": { "type": "string", "maxLength": NAME_MAX_CHARS, "description": "A short name in the owner's language; its handle comes from it." },
            "template": {
                "type": "string",
                "enum": template_ids(),
                "description": "A role of the bot catalog. Its role, instructions, model and \
                    effort fill what you leave out; your `instructions` go after them under \
                    \"For this crew\". Without it, `role` and `instructions` are required.",
            },
            "role": { "type": "string", "maxLength": ROLE_MAX_CHARS },
            "instructions": { "type": "string", "maxLength": INSTRUCTIONS_MAX },
            "model": { "type": "string", "enum": MODELS },
            "effort": { "type": "string", "enum": EFFORTS },
            "why": why(),
        },
        "required": ["op", "crew", "name"],
        "additionalProperties": false,
    })
}

fn add_routine() -> Value {
    json!({
        "type": "object",
        "properties": {
            "op": { "const": "add_routine" },
            "crew": { "type": "string", "description": "The `ref` of a crew of this plan." },
            "agent": { "type": "string", "description": "The name of a bot of that crew in this plan, or \"chief\"." },
            "name": { "type": "string", "maxLength": 80 },
            "prompt": {
                "type": "string",
                "maxLength": PROMPT_MAX,
                "description": "What the bot does each time, written to it.",
            },
            "schedule": schedule_schema(),
            "timezone": { "type": "string", "description": "An IANA zone; the computer's when absent." },
            "why": why(),
        },
        "required": ["op", "crew", "agent", "name", "prompt", "schedule"],
        "additionalProperties": false,
    })
}
