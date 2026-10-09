//! What any agent's output turns into in the chat (spec 8, 30): a reply, a
//! tool that starts and finishes, the end of a turn. An agent decodes its own
//! stream and calls these; saving the items and telling the app is the same
//! for all of them.

use botloft_core::chat::{TOOL_OUTPUT_MAX, clip, tool_file, tool_input_max, tool_summary};
use botloft_core::command::tool_explanation;
use botloft_core::ids::BotId;
use botloft_core::protocol::{ChatBody, ReplyItem, TokenUsage, ToolItem, ToolStatus, TurnItem};
use serde_json::Value;

use tracing::warn;

use super::items;
use crate::state::{Daemon, Event};
use crate::{routines, screens};

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

/// The message written with `uuid` was taken up by the bot: its delivery is
/// read, and a routine's run starts (spec 9.1).
pub(crate) fn message_read(daemon: &Daemon, bot: &BotId, uuid: &str) {
    let now = daemon.clock.now_ms();
    let read = daemon.store().mark_read(uuid, now);
    match read {
        Ok(Some(delivery)) => {
            routines::turn_began(daemon, bot, &delivery.message_id);
            daemon.emit(Event::DeliveryChanged(delivery));
        }
        Ok(None) => {}
        Err(err) => warn!("could not mark a delivery read: {err}"),
    }
}

/// The agent has a conversation the next start resumes (spec 7.3).
pub(crate) fn session_started(daemon: &Daemon, bot: &BotId, session: &str) {
    let store = daemon.store();
    let known = store.session_id(bot).ok().flatten();
    if known.as_deref() != Some(session)
        && let Err(err) = store.set_session_id(bot, Some(session))
    {
        warn!(bot = %bot, "could not save the session id: {err}");
    }
    drop(store);
    daemon.supervisor.remember_session(bot, session);
}

/// Everything that follows the end of a turn, whatever the agent: the
/// routine's run, the screens, the access to other crews, the bot's state.
pub(crate) fn finish_turn(daemon: &Daemon, bot: &BotId, generation: u64, failed: bool) {
    routines::turn_ended(daemon, bot, failed);
    screens::turn_ended(daemon, bot);
    daemon.crew_access.end_turn(bot);
    daemon.supervisor.turn_ended(bot, generation);
}
