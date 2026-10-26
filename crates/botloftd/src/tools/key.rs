//! `ask_key` (spec 32.5): the bot asks the owner for a key, such as an API
//! token, and carries on. The owner types it in a form Botloft keeps out of
//! the chat; the bot hears later, in a message, that it is in its environment.

use botloft_core::ids::BotId;
use botloft_core::keys::KEY_NAME_MAX;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::service::key_requests::{self, WHY_MAX_CHARS};
use crate::state::Daemon;

pub const ASK_KEY: &str = "ask_key";

pub(super) fn tool() -> Value {
    json!({
        "name": ASK_KEY,
        "title": "Ask the owner for a key",
        "description": "Asks the owner for a key you need, such as an API token or a password \
            for a service, without it ever going through the chat. The owner types it in a \
            form Botloft keeps apart; once saved, you restart with it as an environment \
            variable by the name you asked for, and a message from Botloft tells you so. It \
            does not wait: carry on with what does not need the key, or end your turn. Use it \
            instead of asking for a key in your reply, and never ask the owner to paste a key \
            in the chat.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "pattern": "^[A-Z][A-Z0-9_]*$",
                    "maxLength": KEY_NAME_MAX,
                    "description": "The environment variable you will read it from, in \
                        capitals, such as GITHUB_TOKEN or STRIPE_API_KEY.",
                },
                "why": {
                    "type": "string",
                    "maxLength": WHY_MAX_CHARS,
                    "description": "What you need it for and where the owner finds it, short, \
                        in the language the owner writes to you in.",
                },
            },
            "required": ["name", "why"],
            "additionalProperties": false,
        },
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct KeyArgs {
    name: String,
    why: String,
}

pub(super) fn ask(daemon: &Daemon, bot: &BotId, args: KeyArgs) -> Result<Value, String> {
    let id = key_requests::ask(daemon, bot, &args.name, &args.why).map_err(explain)?;
    Ok(json!({
        "request_id": id,
        "note": format!(
            "The owner sees your request for {} in your chat. When they save the key you \
             restart with it and a message from Botloft tells you; if they say not now, a \
             message tells you that too. Don't wait for it: carry on with what doesn't need \
             the key, or end your turn.",
            args.name.trim()
        ),
    }))
}
