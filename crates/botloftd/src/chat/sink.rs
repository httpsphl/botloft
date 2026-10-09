//! What any agent's output turns into in the chat (spec 8, 30): a reply, a
//! tool that starts and finishes, the end of a turn. An agent decodes its own
//! stream and calls these; saving the items and telling the app is the same
//! for all of them.

use botloft_core::chat::{TOOL_OUTPUT_MAX, clip, tool_file, tool_input_max, tool_summary};
use botloft_core::command::tool_explanation;
use botloft_core::ids::BotId;
use botloft_core::protocol::{ChatBody, ReplyItem, TokenUsage, ToolItem, ToolStatus, TurnItem};
use serde_json::Value;

use super::items;
use crate::state::Daemon;

/// The bot's reply text; nothing for an empty one.
pub(crate) fn reply(daemon: &Daemon, bot: &BotId, text: &str) {
    let text = text.trim();
    if !text.is_empty() {
        items::add(
            daemon,
            bot,
            ChatBody::Reply(ReplyItem {
                text: text.to_owned(),
            }),
        );
    }
}

/// A tool the bot began to use, shown as running.
pub(crate) fn tool_started(daemon: &Daemon, bot: &BotId, id: &str, name: &str, input: &Value) {
    items::add(
        daemon,
        bot,
        ChatBody::Tool(ToolItem {
            tool_use_id: id.to_owned(),
            name: name.to_owned(),
            summary: tool_summary(name, input),
            explanation: tool_explanation(name, input),
            input: clip(&input.to_string(), tool_input_max(name)),
            status: ToolStatus::Running,
            output: None,
            file: tool_file(name, input),
        }),
    );
}

/// The tool `id` finished, with what it printed. Does nothing for a tool the
/// chat does not know.
pub(crate) fn tool_finished(daemon: &Daemon, bot: &BotId, id: &str, failed: bool, output: &str) {
    let Ok(Some(item)) = daemon.store().tool_item(bot, id) else {
        return;
    };
    let ChatBody::Tool(tool) = item.body else {
        return;
    };
    let body = ChatBody::Tool(ToolItem {
        status: if failed {
            ToolStatus::Failed
        } else {
            ToolStatus::Done
        },
        output: (!output.is_empty()).then(|| clip(output, TOOL_OUTPUT_MAX)),
        ..tool
    });
    items::update(daemon, &item.id, body);
}

/// The line that closes a turn: how long it took, its tokens when the agent
/// says, and the error it ended with, if any.
pub(crate) fn turn_finished(
    daemon: &Daemon,
    bot: &BotId,
    duration_ms: u64,
    tokens: Option<TokenUsage>,
    error: Option<String>,
) {
    items::add(
        daemon,
        bot,
        ChatBody::Turn(TurnItem {
            duration_ms,
            tokens,
            error,
        }),
    );
}
