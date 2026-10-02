//! Claude Code's permission requests (spec 10.1): the MCP tool
//! `permission_prompt`, named with `--permission-prompt-tool`.

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{ChatBody, ToolStatus};
use serde_json::{Value, json};
use tracing::debug;

use super::{Answer, always, ask};
use crate::state::Daemon;

/// How long to wait for the `tool_use` event that the request is about;
/// it can arrive a moment after the MCP call.
const MATCH_WINDOW: Duration = Duration::from_secs(2);
const MATCH_POLL: Duration = Duration::from_millis(50);

/// Arguments Claude Code sends to the permission tool.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PromptArgs {
    pub tool_name: String,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub tool_use_id: String,
}

/// Handles one `permission_prompt` call and returns the text of the tool
/// result: `{"behavior": "allow" | "deny", ...}`.
pub async fn prompt(daemon: &Daemon, bot: &BotId, generation: u64, args: PromptArgs) -> String {
    if !matches_running_tool(daemon, bot, &args.tool_use_id).await {
        debug!(bot = %bot, "permission request for no running tool; denied");
        return deny("There is no pending tool call with that id.");
    }
    // The owner said once that this one needs no asking.
    if always::allowed(daemon, bot, &args.tool_name, &args.input) {
        debug!(bot = %bot, tool = %args.tool_name, "allowed by a rule of the bot");
        return json!({ "behavior": "allow", "updatedInput": args.input }).to_string();
    }
    let asked = ask(
        daemon,
        bot,
        generation,
        &args.tool_name,
        &args.input,
        &args.tool_use_id,
    )
    .await;
    match asked {
        // What runs is what was asked: the owner cannot edit a tool call.
        Some(Answer::Allowed { .. }) => {
            json!({ "behavior": "allow", "updatedInput": args.input }).to_string()
        }
        Some(Answer::Denied { note: Some(note) }) => {
            deny(&format!("The owner denied this: {note}"))
        }
        Some(Answer::Denied { note: None }) => deny("The owner denied this."),
        Some(Answer::Expired) => deny("The owner did not answer in time."),
        None => deny("Botloft could not ask the owner."),
    }
}

fn deny(message: &str) -> String {
    json!({ "behavior": "deny", "message": message }).to_string()
}

/// Whether the bot's chat shows `tool_use_id` running, waiting briefly for
/// the event if it has not been read yet.
async fn matches_running_tool(daemon: &Daemon, bot: &BotId, tool_use_id: &str) -> bool {
    if tool_use_id.is_empty() {
        return false;
    }
    let deadline = tokio::time::Instant::now() + MATCH_WINDOW;
    loop {
        let found = daemon.store().tool_item(bot, tool_use_id);
        if let Ok(Some(item)) = found
            && matches!(&item.body, ChatBody::Tool(tool) if tool.status == ToolStatus::Running)
        {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(MATCH_POLL).await;
    }
}
