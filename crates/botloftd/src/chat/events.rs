//! What each stream-json event means for the chat and the bot's state
//! (spec 8.1). Fields are read defensively: the format is not documented
//! and grows between Claude Code versions.

use botloft_core::chat::{TOOL_OUTPUT_MAX, clip, tool_file, tool_input_max, tool_summary};
use botloft_core::ids::BotId;
use botloft_core::protocol::{ChatBody, ChatDelta, ReplyItem, ToolItem, ToolStatus, TurnItem};
use serde_json::Value;
use tracing::{debug, warn};

use super::{account, control, items};
use crate::state::{Daemon, Event};
use crate::{context, routines, service};

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
        "system" if subtype == Some("init") => {
            session(daemon, bot, event);
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
        "stream_event" if !from_subagent => {
            delta(daemon, bot, event);
            crate::screens::stream(daemon, bot, event);
        }
        "assistant" if !from_subagent => assistant(daemon, bot, generation, event),
        "user" if event["isReplay"].as_bool() == Some(true) => replay(daemon, bot, event),
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
    if let Some(mode) = event["permissionMode"].as_str() {
        service::modes::reported(daemon, bot, mode);
    }
    if let Some(model) = event["model"].as_str() {
        service::models::reported(daemon, bot, model);
    }
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
                        input: clip(&input.to_string(), tool_input_max(name)),
                        status: ToolStatus::Running,
                        output: None,
                        file: tool_file(name, input),
                    }),
                );
            }
            _ => {}
        }
    }
}

/// The bot began the turn for a message the courier wrote (spec 9.1).
fn replay(daemon: &Daemon, bot: &BotId, event: &Value) {
    let Some(uuid) = event["uuid"].as_str() else {
        return;
    };
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

fn tool_results(daemon: &Daemon, bot: &BotId, event: &Value) {
    for block in blocks(event) {
        if block["type"] != "tool_result" {
            continue;
        }
        let Some(id) = block["tool_use_id"].as_str() else {
            continue;
        };
        crate::screens::tool_done(daemon, bot, id);
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

fn result(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
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
        items::add(
            daemon,
            bot,
            ChatBody::Turn(TurnItem {
                duration_ms: event["duration_ms"].as_u64().unwrap_or_default(),
                cost_usd: event["total_cost_usd"].as_f64(),
                error,
            }),
        );
    }
    routines::turn_ended(daemon, bot, failed);
    crate::screens::turn_ended(daemon, bot);
    daemon.supervisor.turn_ended(bot, generation);
}
