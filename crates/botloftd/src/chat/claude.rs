//! Claude Code's stream-json, decoded into the chat (spec 8.1, 30): what each
//! event means for the chat and the bot's state
//! (spec 8.1). Fields are read defensively: the format is not documented
//! and grows between Claude Code versions.

use botloft_core::ids::BotId;
use botloft_core::protocol::TokenUsage;
use serde_json::Value;
use tracing::debug;

use super::{account, control, sink};
use crate::state::Daemon;
use crate::{context, service};

pub(crate) fn apply(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
    // Lines from a process that was replaced say nothing about the new one.
    if !daemon.supervisor.is_current(bot, generation) {
        return;
    }
    let kind = event["type"].as_str().unwrap_or_default();
    let subtype = event["subtype"].as_str();
    // Subagents work inside a tool call; their traffic stays out of the chat.
    let from_subagent = !event["parent_tool_use_id"].is_null();
    match kind {
        "system" if subtype == Some("init") => {
            init(daemon, bot, event);
            daemon.supervisor.turn_began(bot, generation);
        }
        "system" if subtype == Some("background_tasks_changed") => {
            daemon
                .supervisor
                .agents_running(bot, generation, agent_count(event));
        }
        "system" if subtype == Some("status") => context::status(daemon, bot, event),
        "system" if subtype == Some("compact_boundary") => context::compacted(daemon, bot, event),
        "control_response" => control::answered(daemon, bot, event),
        "stream_event" if !from_subagent => crate::screens::stream(daemon, bot, event),
        "assistant" if !from_subagent => assistant(daemon, bot, generation, event),
        "user" if event["isReplay"].as_bool() == Some(true) => {
            let uuid = event["uuid"].as_str();
            daemon.supervisor.message_began(bot, generation, uuid);
            replay(daemon, bot, event);
        }
        "user" if !from_subagent => tool_results(daemon, bot, event),
        "rate_limit_event" => account::rate_limit(daemon, bot, generation, event),
        "result" => result(daemon, bot, generation, event),
        _ => debug!(bot = %bot, kind, subtype, "stream event not used"),
    }
}

/// How many of the background tasks Claude Code lists are subagents. Shell
/// commands and monitors can run for as long as the bot lives, so they do
/// not count as work.
fn agent_count(event: &Value) -> u32 {
    let agents = event["tasks"].as_array().map_or(0, |tasks| {
        tasks
            .iter()
            .filter(|task| task["task_type"] == "local_agent")
            .count()
    });
    u32::try_from(agents).unwrap_or(u32::MAX)
}

/// Every turn starts with the mode and the model in use (spec 7.4).
fn init(daemon: &Daemon, bot: &BotId, event: &Value) {
    if let Some(mode) = event["permissionMode"].as_str() {
        service::modes::reported(daemon, bot, mode);
    }
    if let Some(model) = event["model"].as_str() {
        service::models::reported(daemon, bot, model);
    }
}

/// The piece of reply text a `stream_event` carries, if that is what it is
/// (spec 8.3). A subagent's text stays out of the chat.
pub(crate) fn live_text(event: &Value) -> Option<&str> {
    if event["type"] != "stream_event" || !event["parent_tool_use_id"].is_null() {
        return None;
    }
    let inner = &event["event"];
    if inner["type"] != "content_block_delta" || inner["delta"]["type"] != "text_delta" {
        return None;
    }
    inner["delta"]["text"]
        .as_str()
        .filter(|text| !text.is_empty())
}

pub(super) fn blocks(event: &Value) -> &[Value] {
    event["message"]["content"]
        .as_array()
        .map_or(&[], Vec::as_slice)
}

fn assistant(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
    if let Some(error) = event["error"].as_str() {
        account::failed_turn(daemon, bot, generation, error, event);
        return;
    }
    // What `/compact` printed is the daemon's to tell, not a reply.
    if context::command_output(daemon, bot, event) {
        return;
    }
    context::used(daemon, bot, &event["message"]);
    for block in blocks(event) {
        match block["type"].as_str() {
            Some("text") => sink::reply(daemon, bot, block["text"].as_str().unwrap_or_default()),
            Some("tool_use") => sink::tool_started(
                daemon,
                bot,
                block["id"].as_str().unwrap_or_default(),
                block["name"].as_str().unwrap_or("tool"),
                &block["input"],
            ),
            _ => {}
        }
    }
}

/// The bot began the turn for a message the courier wrote (spec 9.1).
fn replay(daemon: &Daemon, bot: &BotId, event: &Value) {
    super::session::began(daemon, bot, event);
    let Some(uuid) = event["uuid"].as_str() else {
        return;
    };
    sink::message_read(daemon, bot, uuid);
}

fn tool_results(daemon: &Daemon, bot: &BotId, event: &Value) {
    for block in blocks(event) {
        if block["type"] != "tool_result" {
            continue;
        }
        let Some(id) = block["tool_use_id"].as_str() else {
            continue;
        };
        crate::screens::tool_done(daemon, bot, id);
        let failed = block["is_error"].as_bool() == Some(true);
        sink::tool_finished(daemon, bot, id, failed, &result_text(&block["content"]));
    }
}

/// The text of a tool result: a string, or text blocks among others.
fn result_text(content: &Value) -> String {
    match content {
        Value::String(text) => text.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|part| part["text"].as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

fn result(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
    // No turn began, so none ended: nothing of this is for the chat.
    if super::session::missing(event) {
        debug!(bot = %bot, "Claude Code does not have the conversation to resume");
        daemon.supervisor.session_missing(bot, generation);
        return;
    }
    crate::service::plan::turn_ended(daemon, bot, generation, event);
    let reloaded = context::reloaded(daemon, bot);
    let failed = event["is_error"].as_bool() == Some(true);
    let error = failed.then(|| {
        event["terminal_reason"]
            .as_str()
            .or_else(|| event["subtype"].as_str())
            .unwrap_or("error")
            .to_owned()
    });
    // A compaction ends like a turn; its notice already says what happened.
    if !context::turn_ended(daemon, bot, event) {
        sink::turn_finished(
            daemon,
            bot,
            event["duration_ms"].as_u64().unwrap_or_default(),
            tokens(&event["usage"], reloaded),
            error,
        );
    }
    sink::finish_turn(daemon, bot, generation, failed);
}

/// The turn's tokens from the `usage` of a `result`: the sum of the turn's
/// requests, not of the whole session like `modelUsage` (spec 8.7, 19).
/// `reloaded` is the daemon's own count, part of the cache write.
fn tokens(usage: &Value, reloaded: u64) -> Option<TokenUsage> {
    if !usage.is_object() {
        return None;
    }
    let count = |key: &str| usage[key].as_u64().unwrap_or_default();
    Some(TokenUsage {
        input: count("input_tokens"),
        cache_write: count("cache_creation_input_tokens"),
        reloaded: reloaded.min(count("cache_creation_input_tokens")),
        cache_read: count("cache_read_input_tokens"),
        output: count("output_tokens"),
    })
}
