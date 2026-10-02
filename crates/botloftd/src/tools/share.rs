//! `share_file` (spec 10): the bot shows the owner files it made, as cards
//! in the chat they can open and save. The card is the tool's own chat
//! item; this only checks the files and describes them.

use botloft_core::ids::BotId;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::service::files;
use crate::state::Daemon;

pub const SHARE_FILE: &str = "share_file";
/// Files in one card group, as many as the owner can attach to a message.
pub const SHARE_MAX: usize = 10;

pub(super) fn tool() -> Value {
    json!({
        "name": SHARE_FILE,
        "title": "Share a file",
        "description": "Shows the owner files you made, or that they ask you for, as cards in \
            your chat: they can preview each one, open it and save a copy anywhere. Use it \
            whenever you finish a file for the owner, and when they ask for one again, instead of \
            writing its path in your reply. Only files in the crew's work folder or your own \
            folder, or files you wrote, can be shared.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "files": {
                    "type": "array",
                    "items": { "type": "string" },
                    "minItems": 1,
                    "maxItems": SHARE_MAX,
                    "description": "Paths of the files: absolute, or relative to your own \
                        folder. Write them with forward slashes (C:/folder/file.md).",
                },
            },
            "required": ["files"],
            "additionalProperties": false,
        },
        "annotations": { "readOnlyHint": true },
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ShareArgs {
    files: Vec<String>,
}

/// Every file must be one the owner can read; otherwise none is shown and
/// the bot reads which one and why.
pub(super) fn share(daemon: &Daemon, bot: &BotId, args: ShareArgs) -> Result<Value, String> {
    if args.files.is_empty() || args.files.len() > SHARE_MAX {
        return Err(format!("files takes 1 to {SHARE_MAX} paths"));
    }
    let shown = args
        .files
        .iter()
        .map(|path| files::shared(daemon, bot, path).map_err(explain))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "shown": shown,
        "note": "The owner sees each file as a card in your chat, with buttons to open and save \
            it. You don't need to repeat the paths in your reply.",
    }))
}
