//! Compacting a bot's conversation (spec 8.6): `/compact` asked for by the
//! owner, and Claude Code compacting by itself.

mod common;

use botloft_core::protocol::{
    Bot, BotIdParams, BotState, BotsRestartParams, BotsSetPausedParams, NoticeCode,
};
use botloftd::context;
use botloftd::runtime::fake::FakeProcess;
use botloftd::service::{ApiError, bots};
use common::context::{asks, bot, holds, kinds, notices, sent, settle, shown};
use common::stream;
use common::supervised::{Setup, setup};
use serde_json::{Value, json};

fn compact(s: &Setup) -> Result<Bot, ApiError> {
    let params = BotIdParams {
        bot_id: s.bot.clone(),
    };
    context::compact(&s.daemon, params)
}

fn boundary(trigger: &str) -> Value {
    json!({
        "type": "system", "subtype": "compact_boundary",
        "compact_metadata": { "trigger": trigger, "pre_tokens": 85_688, "post_tokens": 20_792 },
    })
}

fn status(status: Value, result: Value) -> Value {
    json!({ "type": "system", "subtype": "status", "status": status, "compact_result": result })
}

/// What Claude Code prints when `/compact` has run.
async fn compacted(process: &FakeProcess) {
    process.emit(status(json!("compacting"), Value::Null)).await;
    process.emit(status(Value::Null, json!("success"))).await;
    process.emit(stream::init("session-1")).await;
    process.emit(boundary("manual")).await;
    process
        .emit(json!({
            "type": "result", "subtype": "success", "is_error": false, "num_turns": 0,
            "duration_ms": 12_375, "total_cost_usd": 0.03, "local_command": "compact",
        }))
        .await;
}

#[tokio::test(start_paused = true)]
async fn the_owner_compacts_the_conversation() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    holds(&process, 556_000, 1_000_000).await;
    let mut events = s.daemon.subscribe();

    assert!(
        compact(&s)
            .expect("compact")
            .context
            .expect("context")
            .compacting
    );
    let lines = process.input_lines();
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0]["type"], "user");
    assert_eq!(lines[0]["message"]["content"][0]["text"], "/compact");
    assert_eq!(s.state(), BotState::Busy);
    // Asking again while it is on its way writes nothing more.
    compact(&s).expect("compact again");
    assert_eq!(process.input_lines().len(), 1);

    compacted(&process).await;
    s.until(BotState::Idle).await;
    assert!(!shown(&s).compacting);
    assert_eq!(asks(&process), 3, "after the summary and at the end");
    holds(&process, 24_034, 1_000_000).await;
    assert_eq!(shown(&s).used_tokens, 24_034);

    let found = notices(&s);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].0, NoticeCode::Compacted);
    assert_eq!(
        kinds(&s),
        ["notice"],
        "a compaction is not a turn of the bot"
    );
    let sent = sent(&mut events);
    assert!(
        sent.first()
            .expect("sent")
            .as_ref()
            .expect("context")
            .compacting
    );
    assert!(
        !sent
            .last()
            .expect("sent")
            .as_ref()
            .expect("context")
            .compacting
    );
}

#[tokio::test(start_paused = true)]
async fn a_compaction_waits_behind_the_turn_in_progress() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    holds(&process, 300_000, 1_000_000).await;
    s.message("work");
    compact(&s).expect("compact");
    assert_eq!(process.input_lines().len(), 2);

    process.emit(stream::result(false)).await;
    settle().await;
    assert_eq!(
        s.state(),
        BotState::Busy,
        "the compaction still has its turn"
    );
    assert!(shown(&s).compacting);
    compacted(&process).await;
    s.until(BotState::Idle).await;
    assert_eq!(kinds(&s), ["turn", "notice"]);
}

#[tokio::test(start_paused = true)]
async fn claude_code_compacting_by_itself_is_told_in_the_chat() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    holds(&process, 160_000, 200_000).await;
    s.message("read more");

    process.emit(status(json!("requesting"), Value::Null)).await;
    settle().await;
    assert!(!shown(&s).compacting);
    process.emit(status(json!("compacting"), Value::Null)).await;
    settle().await;
    assert!(shown(&s).compacting);
    process.emit(status(Value::Null, json!("success"))).await;
    process.emit(boundary("auto")).await;
    // The summary comes back as a message of its own; it is not a reply.
    process
        .emit(json!({ "type": "user", "isSynthetic": true, "parent_tool_use_id": null,
            "message": { "role": "user", "content": [{ "type": "text", "text": "Summary: ..." }] } }))
        .await;
    settle().await;
    assert!(!shown(&s).compacting);
    assert_eq!(notices(&s)[0].0, NoticeCode::AutoCompacted);
    assert_eq!(s.state(), BotState::Busy, "the turn goes on");

    process.emit(stream::result(false)).await;
    s.until(BotState::Idle).await;
    assert_eq!(kinds(&s), ["notice", "turn"]);
}

#[tokio::test(start_paused = true)]
async fn nothing_to_compact_is_a_notice_not_a_reply() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    holds(&process, 22_000, 200_000).await;
    compact(&s).expect("compact");

    process.emit(stream::init("session-1")).await;
    process
        .emit(json!({
            "type": "assistant", "parent_tool_use_id": null,
            "message": { "role": "assistant", "model": "<synthetic>",
                "content": [{ "type": "text", "text": "Error: No messages to compact" }] },
            "local_command_run": { "command": "compact", "args": "" },
            "local_command_outcome": { "kind": "failed" },
        }))
        .await;
    process
        .emit(
            json!({ "type": "result", "subtype": "success", "is_error": false,
            "duration_ms": 129, "total_cost_usd": 0, "local_command": "compact" }),
        )
        .await;
    s.until(BotState::Idle).await;

    assert_eq!(
        notices(&s),
        [(
            NoticeCode::CompactFailed,
            "No messages to compact".to_owned()
        )]
    );
    assert_eq!(kinds(&s), ["notice"]);
    assert!(!shown(&s).compacting);
}

#[tokio::test(start_paused = true)]
async fn a_bot_that_is_not_running_cannot_compact() {
    let s = setup().await;
    s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let pause = BotsSetPausedParams {
        bot_id: s.bot.clone(),
        paused: true,
    };
    bots::set_paused(&s.daemon, pause).expect("pause");
    s.until(BotState::Offline).await;
    assert!(matches!(compact(&s), Err(ApiError::Conflict(_))));
}

#[tokio::test(start_paused = true)]
async fn a_process_that_ends_takes_its_compaction_and_a_new_conversation_its_size() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    holds(&first, 556_000, 1_000_000).await;
    compact(&s).expect("compact");
    first.exit(1).await;
    let second = s.runtime.process(2).await;
    settle().await;
    let context = shown(&s);
    assert!(!context.compacting);
    assert_eq!(
        context.used_tokens, 556_000,
        "the same conversation, resumed"
    );
    assert_eq!(asks(&second), 1);

    s.until(BotState::Idle).await;
    let restart = BotsRestartParams {
        bot_id: s.bot.clone(),
        fresh: Some(true),
    };
    bots::restart(&s.daemon, restart).expect("restart");
    let third = s.runtime.process(3).await;
    settle().await;
    assert_eq!(bot(&s).context, None, "a new conversation");
    holds(&third, 22_205, 1_000_000).await;
    assert_eq!(shown(&s).used_tokens, 22_205);
}
