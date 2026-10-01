//! The conversation written again to the prompt cache after it expired
//! (spec 8.7): counted apart from what a turn really added.

mod common;

use botloft_core::protocol::{BotState, ChatBody, TokenUsage};
use common::context::{holds, settle};
use common::stream;
use common::supervised::{Setup, setup};
use serde_json::{Value, json};

fn request(id: &str, written: u64, read: u64) -> Value {
    json!({
        "type": "assistant", "parent_tool_use_id": null,
        "message": { "id": id, "role": "assistant", "model": "claude-sonnet-5-5",
            "content": [{ "type": "text", "text": "Done." }],
            "usage": { "input_tokens": 2, "cache_creation_input_tokens": written,
                       "cache_read_input_tokens": read, "output_tokens": 983 } },
    })
}

fn result(written: u64, read: u64) -> Value {
    json!({
        "type": "result", "subtype": "success", "is_error": false, "duration_ms": 10_772,
        "usage": { "input_tokens": 2, "cache_creation_input_tokens": written,
                   "cache_read_input_tokens": read, "output_tokens": 983 },
    })
}

fn last_turn(s: &Setup) -> TokenUsage {
    let items = s
        .daemon
        .store()
        .chat_history(&s.bot, None, 50)
        .expect("history");
    items
        .into_iter()
        .find_map(|item| match item.body {
            ChatBody::Turn(turn) => turn.tokens,
            _ => None,
        })
        .expect("a turn with tokens")
}

#[tokio::test(start_paused = true)]
async fn a_turn_after_the_cache_expired_says_how_much_it_wrote_again() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    holds(&process, 72_000, 1_000_000).await;

    s.message("what is still to do?");
    process.emit(stream::began("session-1")).await;
    // Nothing came from the cache. One request prints a line per block.
    process.emit(request("msg_1", 72_636, 0)).await;
    process.emit(request("msg_1", 72_636, 0)).await;
    process.emit(result(72_636, 0)).await;
    s.until(BotState::Idle).await;
    let cold = last_turn(&s);
    assert_eq!(cold.cache_write, 72_636);
    assert_eq!(cold.reloaded, 72_000, "the conversation from before");

    s.message("and now?");
    process.emit(stream::began("session-1")).await;
    process.emit(request("msg_2", 1_400, 72_638)).await;
    process.emit(result(1_400, 72_638)).await;
    settle().await;
    s.until(BotState::Idle).await;
    assert_eq!(last_turn(&s).reloaded, 0, "the cache was warm");
}
