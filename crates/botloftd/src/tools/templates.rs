//! `list_bot_templates` and `get_bot_template` (spec 26.4): the chief reads
//! the bot catalog before it suggests a bot.

use botloft_core::ids::BotId;
use botloft_core::protocol::BotTemplateCategory;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::catalog;
use crate::service::{bots, lead};
use crate::state::Daemon;

pub const LIST_BOT_TEMPLATES: &str = "list_bot_templates";
pub const GET_BOT_TEMPLATE: &str = "get_bot_template";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ListArgs {
    #[serde(default)]
    category: Option<BotTemplateCategory>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct GetArgs {
    id: String,
}

pub(super) fn tools() -> [Value; 2] {
    let categories = [
        "code",
        "design",
        "content",
        "research",
        "business",
        "product",
        "marketing",
    ];
    [
        json!({
            "name": LIST_BOT_TEMPLATES,
            "title": "List bot templates",
            "description": "Only for the crew's chief. Lists the catalog of ready-made bot roles \
                (id, category, name, role and a summary of each). Look here before you write \
                instructions for a new bot: if a role fits, suggest it with suggest_bot and its \
                `template`.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "category": { "type": "string", "enum": categories },
                },
                "additionalProperties": false,
            },
            "annotations": { "readOnlyHint": true },
        }),
        json!({
            "name": GET_BOT_TEMPLATE,
            "title": "Read a bot template",
            "description": "Only for the crew's chief. Reads one role of the catalog with the \
                instructions a bot made from it would start with, and its default model and \
                effort.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "An id from list_bot_templates." },
                },
                "required": ["id"],
                "additionalProperties": false,
            },
            "annotations": { "readOnlyHint": true },
        }),
    ]
}

pub(super) fn list(daemon: &Daemon, bot: &BotId, args: ListArgs) -> Result<Value, String> {
    chief_only(daemon, bot)?;
    Ok(json!({
        "templates": catalog::list(args.category),
        "note": "Read one with get_bot_template. To use it, call suggest_bot with its id in \
            `template`; add `instructions` only for what is specific to this crew.",
    }))
}

pub(super) fn get(daemon: &Daemon, bot: &BotId, args: GetArgs) -> Result<Value, String> {
    chief_only(daemon, bot)?;
    let template = catalog::get(args.id.trim()).ok_or_else(|| {
        format!(
            "no bot template named {}; list them with list_bot_templates",
            args.id
        )
    })?;
    serde_json::to_value(template).map_err(|err| err.to_string())
}

fn chief_only(daemon: &Daemon, bot: &BotId) -> Result<(), String> {
    let (crew, _) = bots::active(&daemon.store(), bot).map_err(explain)?;
    if lead::is_lead(&crew, bot) {
        Ok(())
    } else {
        Err(
            "only the crew's chief can read the bot catalog; ask the chief with send_message"
                .to_owned(),
        )
    }
}
