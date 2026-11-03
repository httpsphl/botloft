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
    process.emit(stream::began("session-1")).await;

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

#[tokio::test(start_paused = true)]
async fn the_answer_tells_what_fills_the_conversation_part_by_part() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    // As Claude Code 2.1.309 answers after a short turn (spec 33.3).
    process
        .answer_control(
            "get_context_usage",
            json!({
                "totalTokens": 24_200, "maxTokens": 1_000_000, "rawMaxTokens": 1_000_000,
                "autoCompactThreshold": 967_000, "isAutoCompactEnabled": true,
                "categories": [
                    { "id": "system_prompt", "kind": "used", "tokens": 3341 },
                    { "id": "system_tools", "kind": "used", "tokens": 13_908 },
                    { "id": "system_tools_deferred", "kind": "deferred",
                      "isDeferred": true, "tokens": 13_427 },
                    { "id": "memory_files", "kind": "used", "tokens": 299 },
                    { "id": "skills", "kind": "used", "tokens": 2210 },
                    { "id": "messages", "kind": "used", "tokens": 4442 },
                    { "id": "autocompact_buffer", "kind": "buffer", "tokens": 33_000 },
                    { "id": "free_space", "kind": "free", "tokens": 942_800 }
                ],
                "memoryFiles": [{ "path": "C:/bots/scout/CLAUDE.md", "type": "Project",
                                  "tokens": 299 }]
            }),
        )
        .await;
    settle().await;
    let context = shown(&s);
    let parts: Vec<_> = context
        .parts
        .iter()
        .map(|part| (part.kind.as_str(), part.tokens))
        .collect();
    assert_eq!(
        parts,
        [
            ("messages", 4442),
            ("instructions", 3341),
            ("tools", 13_908),
            ("memory", 299),
            ("skills", 2210),
            ("buffer", 33_000),
        ]
    );
    assert_eq!(context.memory_files.len(), 1);

    // The size moves with each model request; the parts wait for the next
    // answer, and the app makes up the difference in the conversation.
    s.message("go on");
    process.emit(stream::began("session-1")).await;
    process
        .emit(json!({
            "type": "assistant", "parent_tool_use_id": null,
            "message": { "role": "assistant", "model": "claude-haiku-4-5-20251001", "content": [],
                "usage": { "input_tokens": 10, "cache_creation_input_tokens": 0,
                           "cache_read_input_tokens": 30_000, "output_tokens": 4 } },
        }))
        .await;
    settle().await;
    let context = shown(&s);
    assert_eq!(context.used_tokens, 30_010);
    assert_eq!(context.parts.len(), 6, "the parts stay as last told");
}
