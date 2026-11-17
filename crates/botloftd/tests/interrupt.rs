//! Stopping what a bot does (`bots.interrupt`, spec 9.2): Claude Code is
//! asked on stdin, the turn it cuts ends as stopped in the chat, and a tool
//! that waited for the owner is refused.

mod common;

use botloft_core::protocol::BotState;
use common::bots::{Crew, two_bots};
use common::mcp::Mcp;
use common::stream;
use serde_json::{Value, json};

/// The next `chat.item` whose item matches, skipping others.
async fn item_where(app: &mut common::Client, wanted: impl Fn(&Value) -> bool) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        if wanted(&changed["item"]) {
            return changed;
        }
    }
}

/// The owner writes to @lead; its process takes the message up.
async fn lead_working(c: &mut Crew, body: &str) -> Value {
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": body }),
        )
        .await
        .expect("send");
    let line = c.lead_process.wait_lines(1).await.last().cloned();
    c.lead_process.emit(stream::init("session-1")).await;
    c.lead_process
        .emit(stream::replay(&line.expect("line")))
        .await;
    c.t.until_state(&lead, BotState::Busy).await;
    lead
}

fn interrupts(c: &Crew) -> usize {
    c.lead_process
        .control_lines()
        .iter()
        .filter(|line| line["request"]["subtype"] == "interrupt")
        .count()
}

/// What Claude Code 2.1.309 prints once an interrupt cut the turn (spec 19).
/// A request cut while it streamed counts nothing in `usage`.
async fn cut(c: &Crew, reason: &str, output_tokens: u64) {
    c.lead_process
        .emit(json!({
            "type": "user",
            "message": { "role": "user", "content": [
                { "type": "text", "text": "[Request interrupted by user]" }
            ] },
            "parent_tool_use_id": null,
        }))
        .await;
    c.lead_process
        .emit(json!({
            "type": "result",
            "subtype": "error_during_execution",
            "is_error": true,
            "duration_ms": 4200,
            "total_cost_usd": 0.002,
            "usage": if output_tokens == 0 {
                json!({ "input_tokens": 0, "cache_creation_input_tokens": 0,
                        "cache_read_input_tokens": 0, "output_tokens": 0 })
            } else {
                json!({ "input_tokens": 3, "cache_creation_input_tokens": 900,
                        "cache_read_input_tokens": 0, "output_tokens": output_tokens })
            },
            "errors": ["[ede_diagnostic] result_type=user last_content_type=n/a stop_reason=null"],
            "terminal_reason": reason,
        }))
        .await;
}

#[tokio::test]
async fn stopping_asks_claude_code_and_the_turn_ends_as_stopped() {
    let mut c = two_bots().await;
    let lead = lead_working(&mut c, "Write a long essay").await;
    c.lead_process
        .emit(stream::text("Rivers carve the land"))
        .await;

    let bot = c
        .app
        .call("bots.interrupt", json!({ "botId": lead["id"] }))
        .await
        .expect("interrupt");
    assert_eq!(bot["id"], lead["id"]);
    assert_eq!(interrupts(&c), 1);
    let asked = c.lead_process.control_lines();
    let id = asked.last().expect("asked")["request_id"]
        .as_str()
        .expect("id")
        .to_owned();
    assert!(id.starts_with("botloft-interrupt-"), "{id}");

    // Claude Code answers, prints what it had written, and closes the turn.
    c.lead_process
        .emit(json!({ "type": "control_response", "response": {
            "subtype": "success", "request_id": id, "response": { "still_queued": [] } } }))
        .await;
    cut(&c, "aborted_streaming", 0).await;
    let turn = item_where(&mut c.app, |item| item["body"]["kind"] == "turn").await;
    let body = &turn["item"]["body"];
    assert_eq!(body["stopped"], true);
    assert_eq!(body["error"], Value::Null, "stopping is not a failure");
    assert_eq!(body["tokens"], Value::Null, "a cut request is not counted");
    c.t.until_state(&lead, BotState::Idle).await;
}

#[tokio::test]
async fn a_turn_that_fails_by_itself_is_not_stopped() {
    let mut c = two_bots().await;
    let lead = lead_working(&mut c, "Go").await;
    c.lead_process.emit(stream::result(true)).await;
    let turn = item_where(&mut c.app, |item| item["body"]["kind"] == "turn").await;
    let body = &turn["item"]["body"];
    assert_eq!(body["stopped"], false);
    assert_eq!(body["error"], "api_error");
    c.t.until_state(&lead, BotState::Idle).await;
}

#[tokio::test]
async fn stopping_refuses_the_tool_that_waits_for_the_owner() {
    let mut c = two_bots().await;
    let lead = lead_working(&mut c, "Clean the build").await;
    c.lead_process
        .emit(stream::tool_use(
            "toolu_1",
            "Bash",
            json!({ "command": "rm -rf build" }),
        ))
        .await;
    let mut mcp = Mcp::new(c.lead.addr, c.lead.token.clone());
    let args = json!({ "tool_name": "Bash", "input": { "command": "rm -rf build" },
                       "tool_use_id": "toolu_1" });
    let decision =
        tokio::spawn(async move { mcp.tool("permission_prompt", args).await.expect("decision") });
    item_where(&mut c.app, |item| item["body"]["status"] == "pending").await;
    c.t.until_state(&lead, BotState::NeedsApproval).await;

    c.app
        .call("bots.interrupt", json!({ "botId": lead["id"] }))
        .await
        .expect("interrupt");
    assert_eq!(interrupts(&c), 1);
    let denied = item_where(&mut c.app, |item| item["body"]["status"] == "denied").await;
    assert_eq!(denied["item"]["body"]["kind"], "approval");
    let outcome = decision.await.expect("task");
    assert_eq!(outcome["behavior"], "deny");

    c.lead_process
        .emit(stream::tool_result(
            "toolu_1",
            "The user doesn't want to proceed with this tool use.",
            true,
        ))
        .await;
    cut(&c, "aborted_tools", 90).await;
    let turn = item_where(&mut c.app, |item| item["body"]["kind"] == "turn").await;
    assert_eq!(turn["item"]["body"]["stopped"], true);
    assert_eq!(
        turn["item"]["body"]["tokens"]["output"], 90,
        "what the requests before spent still shows"
    );
    c.t.until_state(&lead, BotState::Idle).await;
}

#[tokio::test]
async fn a_bot_with_nothing_to_stop_is_left_alone() {
    let mut c = two_bots().await;
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.t.until_state(&lead, BotState::Idle).await;
    let bot = c
        .app
        .call("bots.interrupt", json!({ "botId": lead["id"] }))
        .await
        .expect("nothing to stop is fine");
    assert_eq!(bot["state"], "idle");
    assert_eq!(interrupts(&c), 0);

    let missing = c
        .app
        .call(
            "bots.interrupt",
            json!({ "botId": "bot_01JZZZZZZZZZZZZZZZZZZZZZZZ" }),
        )
        .await
        .expect_err("no such bot");
    assert_eq!(missing.code, -32002);
}
