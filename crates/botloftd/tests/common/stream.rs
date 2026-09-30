//! Stream-json events as Claude Code prints them (spec 8.1), for fake bots.

use botloftd::runtime::fake::FakeProcess;
use serde_json::{Value, json};

pub fn init(session: &str) -> Value {
    json!({ "type": "system", "subtype": "init", "session_id": session, "tools": [] })
}

/// Claude Code giving back a message it began, with the daemon's uuid.
pub fn replay(line: &Value) -> Value {
    json!({
        "type": "user",
        "isReplay": true,
        "uuid": line["uuid"],
        "message": line["message"],
        "parent_tool_use_id": null,
    })
}

/// The replay that opens a turn in `session`. From here on Claude Code has
/// the conversation on disk and it can be resumed (spec 7.3).
pub fn began(session: &str) -> Value {
    json!({
        "type": "user",
        "isReplay": true,
        "uuid": "u",
        "session_id": session,
        "message": { "role": "user", "content": "hi" },
        "parent_tool_use_id": null,
    })
}

/// What Claude Code 2.1.284 prints, before it exits, when the conversation
/// it was told to resume is not on disk (spec 19).
pub fn no_conversation(session: &str) -> Value {
    json!({
        "type": "result",
        "subtype": "error_during_execution",
        "is_error": true,
        "duration_ms": 0,
        "num_turns": 0,
        "session_id": session,
        "total_cost_usd": 0,
        "errors": [format!("No conversation found with session ID: {session}")],
    })
}

pub fn text(text: &str) -> Value {
    json!({
        "type": "assistant",
        "message": { "role": "assistant", "content": [{ "type": "text", "text": text }] },
        "parent_tool_use_id": null,
    })
}

pub fn tool_use(id: &str, name: &str, input: Value) -> Value {
    json!({
        "type": "assistant",
        "message": { "role": "assistant", "content": [
            { "type": "tool_use", "id": id, "name": name, "input": input }
        ] },
        "parent_tool_use_id": null,
    })
}

pub fn tool_result(id: &str, output: &str, is_error: bool) -> Value {
    json!({
        "type": "user",
        "message": { "role": "user", "content": [
            { "type": "tool_result", "tool_use_id": id, "content": output, "is_error": is_error }
        ] },
        "parent_tool_use_id": null,
    })
}

pub fn delta(text: &str) -> Value {
    json!({
        "type": "stream_event",
        "event": { "type": "content_block_delta", "index": 0,
                   "delta": { "type": "text_delta", "text": text } },
        "parent_tool_use_id": null,
    })
}

pub fn api_error(error: &str, text: &str) -> Value {
    json!({
        "type": "assistant",
        "error": error,
        "message": { "role": "assistant", "content": [{ "type": "text", "text": text }] },
        "parent_tool_use_id": null,
    })
}

pub fn result(is_error: bool) -> Value {
    json!({
        "type": "result",
        "subtype": "success",
        "is_error": is_error,
        "duration_ms": 1234,
        "total_cost_usd": 0.0125,
        "usage": {
            "input_tokens": 12,
            "cache_creation_input_tokens": 300,
            "cache_read_input_tokens": 4000,
            "output_tokens": 45,
        },
        "terminal_reason": if is_error { "api_error" } else { "completed" },
    })
}

/// The bot begins the message `line`, answers `reply` and ends the turn.
pub async fn answer(process: &FakeProcess, line: &Value, reply: &str) {
    process.emit(init("session-1")).await;
    process.emit(replay(line)).await;
    process.emit(text(reply)).await;
    process.emit(result(false)).await;
}
