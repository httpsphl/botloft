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
