//! How full a bot's conversation is (spec 8.6): what Claude Code answers
//! when asked, and the usage it prints while it works.

mod common;

use botloft_core::protocol::BotState;
use common::context::{asks, bot, holds, sent, settle, shown};
use common::stream;
use common::supervised::setup;
use serde_json::json;

#[tokio::test(start_paused = true)]
async fn a_new_process_tells_how_full_the_conversation_is() {
    let s = setup().await;
    let mut events = s.daemon.subscribe();
    let process = s.runtime.process(1).await;
    assert_eq!(asks(&process), 1, "asked as it starts");
    assert_eq!(bot(&s).context, None, "not known until it answers");

    holds(&process, 556_000, 1_000_000).await;
    let context = shown(&s);
    assert_eq!(
        (context.used_tokens, context.window_tokens),
        (556_000, 1_000_000)
    );
    assert_eq!(context.auto_compact_tokens, Some(967_000));
    assert!(!context.compacting);
    assert_eq!(sent(&mut events), vec![Some(context)]);
}

#[tokio::test(start_paused = true)]
async fn the_size_follows_each_model_request_and_is_asked_again_when_a_turn_ends() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    holds(&process, 22_000, 200_000).await;
    s.message("read the files");

    let request = |cached: u64| {
        json!({
            "type": "assistant", "parent_tool_use_id": null,
            "message": { "role": "assistant", "model": "claude-haiku-4-5-20251001", "content": [],
                "usage": { "input_tokens": 10, "cache_creation_input_tokens": 763,
                           "cache_read_input_tokens": cached, "output_tokens": 4 } },
        })
    };
    process.emit(request(45_245)).await;
    settle().await;
    assert_eq!(shown(&s).used_tokens, 46_018);
    // A subagent's requests fill its own conversation, not the bot's.
    let mut inside = request(90_000);
    inside["parent_tool_use_id"] = json!("toolu_1");
    process.emit(inside).await;
    // What a command printed was not a model request.
    let mut printed = request(0);
    printed["message"]["model"] = json!("<synthetic>");
    process.emit(printed).await;
    settle().await;
    assert_eq!(shown(&s).used_tokens, 46_018);

    process.emit(stream::result(false)).await;
    settle().await;
    assert_eq!(asks(&process), 2, "asked again at the end of the turn");
    holds(&process, 46_700, 200_000).await;
    assert_eq!(shown(&s).used_tokens, 46_700);
    assert_eq!(shown(&s).auto_compact_tokens, Some(167_000));
}

#[tokio::test(start_paused = true)]
async fn an_answer_without_sizes_or_with_auto_compact_off_is_read_carefully() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    process
        .answer_control("get_context_usage", json!({ "categories": [] }))
        .await;
    settle().await;
    assert_eq!(bot(&s).context, None);

    process
        .answer_control(
            "get_context_usage",
            json!({ "totalTokens": 1000, "maxTokens": 200_000,
                    "autoCompactThreshold": 167_000, "isAutoCompactEnabled": false }),
        )
        .await;
    settle().await;
    assert_eq!(shown(&s).auto_compact_tokens, None);

    // A request this Claude Code does not know fails; nothing changes.
    let id = "botloft-context-0";
    process
        .emit(json!({ "type": "control_response",
            "response": { "subtype": "error", "request_id": id, "error": "Unsupported" } }))
        .await;
    settle().await;
    assert_eq!(shown(&s).used_tokens, 1000);
}
