//! What each stream-json event means for the chat and the bot's state
//! (spec 8.1). Fields are read defensively: the format is not documented
//! and grows between Claude Code versions.

use botloft_core::chat::{TOOL_INPUT_MAX, TOOL_OUTPUT_MAX, clip, tool_summary};
use botloft_core::ids::BotId;
use botloft_core::protocol::{
    AccountUsage, ChatBody, ChatDelta, NoticeCode, NoticeItem, NoticeLevel, ReplyItem, ToolItem,
    ToolStatus, TurnItem, UsageWindow,
};
use serde_json::Value;
use tracing::{debug, warn};

use super::items;
use crate::state::{Daemon, Event};

/// Errors that restarting cannot fix: Claude Code needs the owner.
const SIGN_IN_ERRORS: &[&str] = &[
    "authentication_failed",
    "oauth_org_not_allowed",
    "billing_error",
    "account_on_hold",
];
/// When a rate limit gives no reset time.
const DEFAULT_LIMIT_MS: i64 = 5 * 60 * 1000;

pub(super) fn apply(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
    // Lines from a process that was replaced say nothing about the new one.
    if !daemon.supervisor.is_current(bot, generation) {
        return;
    }
    let kind = event["type"].as_str().unwrap_or_default();
    let subtype = event["subtype"].as_str();
    // Subagents work inside a tool call; their traffic stays out of the chat.
    let from_subagent = !event["parent_tool_use_id"].is_null();
    match kind {
        "system" if subtype == Some("init") => session(daemon, bot, event),
        "stream_event" if !from_subagent => delta(daemon, bot, event),
        "assistant" if !from_subagent => assistant(daemon, bot, generation, event),
        "user" if event["isReplay"].as_bool() == Some(true) => replay(daemon, event),
        "user" if !from_subagent => tool_results(daemon, bot, event),
        "rate_limit_event" => rate_limit(daemon, bot, generation, event),
        "result" => result(daemon, bot, generation, event),
        _ => debug!(bot = %bot, kind, subtype, "stream event not used"),
    }
}

/// Every turn starts with the session id; the next start resumes it.
fn session(daemon: &Daemon, bot: &BotId, event: &Value) {
    let Some(session) = event["session_id"].as_str() else {
        return;
    };
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

fn delta(daemon: &Daemon, bot: &BotId, event: &Value) {
    let inner = &event["event"];
    if inner["type"] != "content_block_delta" || inner["delta"]["type"] != "text_delta" {
        return;
    }
    if let Some(text) = inner["delta"]["text"].as_str()
        && !text.is_empty()
    {
        daemon.emit(Event::ChatDelta(ChatDelta {
            bot_id: bot.clone(),
            text: text.to_owned(),
        }));
    }
}

fn blocks(event: &Value) -> &[Value] {
    event["message"]["content"]
        .as_array()
        .map_or(&[], Vec::as_slice)
}

fn assistant(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
    if let Some(error) = event["error"].as_str() {
        failed_turn(daemon, bot, generation, error, event);
        return;
    }
    for block in blocks(event) {
        match block["type"].as_str() {
            Some("text") => {
                let text = block["text"].as_str().unwrap_or_default().trim();
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
            Some("tool_use") => {
                let name = block["name"].as_str().unwrap_or("tool");
                let input = &block["input"];
                items::add(
                    daemon,
                    bot,
                    ChatBody::Tool(ToolItem {
                        tool_use_id: block["id"].as_str().unwrap_or_default().to_owned(),
                        name: name.to_owned(),
                        summary: tool_summary(name, input),
                        input: clip(&input.to_string(), TOOL_INPUT_MAX),
                        status: ToolStatus::Running,
                        output: None,
                    }),
                );
            }
            _ => {}
        }
    }
}

/// An API error ended the turn: say so, and stop or pause the bot when
/// only the owner or time can fix it (spec 7.2).
fn failed_turn(daemon: &Daemon, bot: &BotId, generation: u64, error: &str, event: &Value) {
    let detail = blocks(event)
        .iter()
        .filter_map(|block| block["text"].as_str())
        .collect::<Vec<_>>()
        .join(" ");
    // The code lets the app say it in the owner's language; the text stays
    // for older apps and for the conversation list.
    let (code, text) = if SIGN_IN_ERRORS.contains(&error) {
        (
            NoticeCode::SignedOut,
            "Claude Code is not signed in or the account cannot be used. Sign in, then restart \
             the bot."
                .to_owned(),
        )
    } else if error == "rate_limit" {
        (
            NoticeCode::UsageLimit,
            "The account reached its usage limit. Messages wait until it resets.".to_owned(),
        )
    } else if detail.is_empty() {
        (NoticeCode::TurnFailed, error.to_owned())
    } else {
        (NoticeCode::TurnFailed, detail)
    };
    items::add(
        daemon,
        bot,
        ChatBody::Notice(NoticeItem {
            level: NoticeLevel::Error,
            code: Some(code),
            text,
        }),
    );
    if SIGN_IN_ERRORS.contains(&error) {
        daemon.supervisor.signed_out(bot, generation);
    } else if error == "rate_limit" {
        let until = daemon
            .usage()
            .and_then(|usage| usage.resets_at)
            .unwrap_or_else(|| daemon.clock.now_ms() + DEFAULT_LIMIT_MS);
        daemon.supervisor.rate_limited(bot, generation, until);
    }
}

/// The bot began the turn for a message the courier wrote (spec 9.1).
fn replay(daemon: &Daemon, event: &Value) {
    let Some(uuid) = event["uuid"].as_str() else {
        return;
    };
    let now = daemon.clock.now_ms();
    match daemon.store().mark_read(uuid, now) {
        Ok(Some(delivery)) => daemon.emit(Event::DeliveryChanged(delivery)),
        Ok(None) => {}
        Err(err) => warn!("could not mark a delivery read: {err}"),
    }
}

fn tool_results(daemon: &Daemon, bot: &BotId, event: &Value) {
    for block in blocks(event) {
        if block["type"] != "tool_result" {
            continue;
        }
        let Some(id) = block["tool_use_id"].as_str() else {
            continue;
        };
        let found = daemon.store().tool_item(bot, id);
        let Ok(Some(item)) = found else {
            continue;
        };
        let ChatBody::Tool(tool) = item.body else {
            continue;
        };
        let failed = block["is_error"].as_bool() == Some(true);
        let output = result_text(&block["content"]);
        let body = ChatBody::Tool(ToolItem {
            status: if failed {
                ToolStatus::Failed
            } else {
                ToolStatus::Done
            },
            output: (!output.is_empty()).then(|| clip(&output, TOOL_OUTPUT_MAX)),
            ..tool
        });
        items::update(daemon, &item.id, body);
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

fn rate_limit(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
    let info = &event["rate_limit_info"];
    let seconds = |value: &Value| value.as_i64().map(|s| s * 1000);
    let mut windows: Vec<UsageWindow> = info["unifiedWindows"]
        .as_object()
        .map(|windows| {
            windows
                .iter()
                .map(|(name, window)| UsageWindow {
                    name: name.clone(),
                    utilization: window["utilization"].as_f64().unwrap_or_default(),
                    resets_at: seconds(&window["resetsAt"]),
                })
                .collect()
        })
        .unwrap_or_default();
    windows.sort_by(|a, b| a.name.cmp(&b.name));
    let status = info["status"].as_str().unwrap_or("unknown").to_owned();
    let resets_at = seconds(&info["resetsAt"]);
    let limited = !status.starts_with("allowed");
    daemon.set_usage(AccountUsage {
        status,
        resets_at,
        windows,
        observed_at: daemon.clock.now_ms(),
    });
    if limited {
        let until = resets_at.unwrap_or_else(|| daemon.clock.now_ms() + DEFAULT_LIMIT_MS);
        daemon.supervisor.rate_limited(bot, generation, until);
    }
}

fn result(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
    let failed = event["is_error"].as_bool() == Some(true);
    let error = failed.then(|| {
        event["terminal_reason"]
            .as_str()
            .or_else(|| event["subtype"].as_str())
            .unwrap_or("error")
            .to_owned()
    });
    items::add(
        daemon,
        bot,
        ChatBody::Turn(TurnItem {
            duration_ms: event["duration_ms"].as_u64().unwrap_or_default(),
            cost_usd: event["total_cost_usd"].as_f64(),
            error,
        }),
    );
    daemon.supervisor.turn_ended(bot, generation);
}
