//! The chat (spec 8) and approvals (spec 10.1), end to end: a fake bot
//! prints stream-json, the app sees items and live text, and permission
//! requests wait for the owner's answer.

mod common;

use std::time::Duration;

use botloft_core::protocol::BotState;
use common::bots::{Crew, two_bots};
use common::mcp::Mcp;
use common::stream;
use serde_json::{Value, json};

const CONFLICT: i64 = -32003;

/// The next `chat.item` whose item matches, skipping others.
async fn item_where(app: &mut common::Client, wanted: impl Fn(&Value) -> bool) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        if wanted(&changed["item"]) {
            return changed;
        }
    }
}

async fn kind(app: &mut common::Client, kind: &str) -> Value {
    item_where(app, |item| item["body"]["kind"] == kind).await
}

/// Sends the owner's message to @lead and waits until its process has it.
async fn talk_to_lead(c: &mut Crew, body: &str) -> Value {
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": body }),
        )
        .await
        .expect("send");
    let lines = c.lead_process.wait_lines(1).await;
    lines.last().cloned().expect("line")
}

async fn lead(c: &mut Crew) -> Value {
    c.app.call("bots.list", json!({})).await.expect("bots")[0].clone()
}

#[tokio::test]
async fn what_the_bot_prints_becomes_the_chat() {
    let mut c = two_bots().await;
    let line = talk_to_lead(&mut c, "Run the tests").await;
    let lead = lead(&mut c).await;
    c.t.until_state(&lead, BotState::Busy).await;
    let p = &c.lead_process;
    p.emit(stream::init("s-1")).await;
    p.emit(stream::replay(&line)).await;
    p.emit(stream::delta("Run")).await;
    p.emit(stream::delta("ning.")).await;
    p.emit(stream::text("Running.")).await;
    p.emit(stream::tool_use(
        "toolu_1",
        "Bash",
        json!({ "command": "npm test", "description": "Run the tests" }),
    ))
    .await;
    // A subagent's traffic stays out of the chat.
    let mut sub = stream::text("inner thoughts");
    sub["parent_tool_use_id"] = json!("toolu_1");
    p.emit(sub).await;
    p.emit(stream::tool_result("toolu_1", "12 passed", false))
        .await;
    p.emit(stream::text("All 12 tests pass.")).await;
    p.emit(stream::result(false)).await;

    // Pieces written close together may arrive as one (spec 8.3).
    let mut live = String::new();
    while live != "Running." {
        let delta = c.app.notification("chat.delta").await;
        live.push_str(delta["text"].as_str().expect("text"));
    }
    let reply = kind(&mut c.app, "reply").await;
    assert_eq!(reply["item"]["body"]["text"], "Running.");
    assert_eq!(reply["activity"]["text"], "Running.");
    let running = kind(&mut c.app, "tool").await;
    assert_eq!(running["item"]["body"]["status"], "running");
    assert_eq!(running["item"]["body"]["summary"], "npm test");
    assert_eq!(running["item"]["body"]["explanation"], "Run the tests");
    // The app names the tool in the owner's language.
    assert_eq!(running["activity"]["text"], "Run the tests");
    assert_eq!(running["activity"]["tool"], "Bash");
    let done = kind(&mut c.app, "tool").await;
    assert_eq!(done["item"]["id"], running["item"]["id"]);
    assert_eq!(done["item"]["body"]["status"], "done");
    assert_eq!(done["item"]["body"]["output"], "12 passed");
    assert_eq!(
        done["activity"],
        Value::Null,
        "an update keeps the list line"
    );
    kind(&mut c.app, "reply").await;
    let turn = kind(&mut c.app, "turn").await;
    assert_eq!(turn["item"]["body"]["durationMs"], 1234);
    c.t.until_state(&lead, BotState::Idle).await;

    let history = c
        .app
        .call("chat.history", json!({ "botId": lead["id"] }))
        .await
        .expect("history");
    let kinds: Vec<_> = history
        .as_array()
        .expect("items")
        .iter()
        .map(|item| item["body"]["kind"].as_str().expect("kind").to_owned())
        .collect();
    assert_eq!(kinds, ["turn", "reply", "tool", "reply", "inbound"]);
    assert!(!history.to_string().contains("inner thoughts"));
    let older = c
        .app
        .call(
            "chat.history",
            json!({ "botId": lead["id"], "before": history[1]["id"], "limit": 2 }),
        )
        .await
        .expect("older");
    assert_eq!(older.as_array().map(Vec::len), Some(2));
    let listed = c.app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(listed[0]["lastActivity"]["text"], "All 12 tests pass.");
}

/// Starts a permission request for `tool_use_id` the way Claude Code does,
/// in the background; the answer comes out of the handle.
fn ask(mcp: &Mcp, tool_use_id: &str) -> tokio::task::JoinHandle<Value> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    let args = json!({ "tool_name": "Bash", "input": { "command": "rm -rf build" }, "tool_use_id": tool_use_id });
    tokio::spawn(async move { mcp.tool("permission_prompt", args).await.expect("decision") })
}

#[tokio::test]
async fn approvals_wait_for_the_owner() {
    let mut c = two_bots().await;
    let line = talk_to_lead(&mut c, "Clean the build").await;
    let lead = lead(&mut c).await;
    c.lead_process.emit(stream::replay(&line)).await;
    c.lead_process
        .emit(stream::tool_use(
            "toolu_9",
            "Bash",
            json!({ "command": "rm -rf build" }),
        ))
        .await;

    let decision = ask(&c.lead, "toolu_9");
    let pending = kind(&mut c.app, "approval").await;
    let body = &pending["item"]["body"];
    assert_eq!(
        (&body["status"], &body["toolName"]),
        (&json!("pending"), &json!("Bash"))
    );
    assert_eq!(body["summary"], "rm -rf build");
    // The app words the line ("Waiting for approval: run a command") in the
    // owner's language.
    assert_eq!(pending["activity"]["kind"], "approval");
    assert_eq!(pending["activity"]["tool"], "Bash");
    assert_eq!(pending["activity"]["text"], "rm -rf build");
    c.t.until_state(&lead, BotState::NeedsApproval).await;

    let answered = c
        .app
        .call(
            "approvals.answer",
            json!({ "approvalId": body["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    assert_eq!(answered["status"], "allowed");
    let outcome = decision.await.expect("task");
    assert_eq!(
        outcome,
        json!({ "behavior": "allow", "updatedInput": { "command": "rm -rf build" } })
    );
    let shown = kind(&mut c.app, "approval").await;
    assert_eq!(shown["item"]["id"], pending["item"]["id"]);
    assert_eq!(shown["item"]["body"]["status"], "allowed");
    c.t.until_state(&lead, BotState::Busy).await;
    let twice = c
        .app
        .call(
            "approvals.answer",
            json!({ "approvalId": body["approvalId"], "allow": false }),
        )
        .await
        .expect_err("already answered");
    assert_eq!(twice.code, CONFLICT);

    // A denial carries the owner's note.
    c.lead_process
        .emit(stream::tool_use(
            "toolu_10",
            "Bash",
            json!({ "command": "rm -rf build" }),
        ))
        .await;
    let decision = ask(&c.lead, "toolu_10");
    let pending = kind(&mut c.app, "approval").await;
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": pending["item"]["body"]["approvalId"], "allow": false, "note": "keep the cache" }),
        )
        .await
        .expect("deny");
    let outcome = decision.await.expect("task");
    assert_eq!(outcome["behavior"], "deny");
    assert_eq!(outcome["message"], "The owner denied this: keep the cache");
}

#[tokio::test]
async fn requests_that_match_no_tool_or_nobody_answers_are_denied() {
    let mut c = two_bots().await;
    let line = talk_to_lead(&mut c, "Go").await;
    c.lead_process.emit(stream::replay(&line)).await;

    // No tool call with that id: denied without bothering the owner.
    let invented = ask(&c.lead, "toolu_made_up").await.expect("task");
    assert_eq!(invented["behavior"], "deny");

    c.lead_process
        .emit(stream::tool_use(
            "toolu_2",
            "Bash",
            json!({ "command": "make" }),
        ))
        .await;
    let started = std::time::Instant::now();
    let outcome = ask(&c.lead, "toolu_2").await.expect("task");
    assert_eq!(outcome["behavior"], "deny");
    assert_eq!(outcome["message"], "The owner did not answer in time.");
    assert!(
        started.elapsed() >= Duration::from_secs(3),
        "the test timeout is 3 s"
    );
    let expired = item_where(&mut c.app, |item| item["body"]["status"] == "expired").await;
    assert_eq!(expired["item"]["body"]["kind"], "approval");
}

#[tokio::test]
async fn a_process_that_ends_expires_its_open_requests() {
    let mut c = two_bots().await;
    let line = talk_to_lead(&mut c, "Go").await;
    c.lead_process.emit(stream::replay(&line)).await;
    c.lead_process
        .emit(stream::tool_use(
            "toolu_3",
            "Bash",
            json!({ "command": "make" }),
        ))
        .await;
    let decision = ask(&c.lead, "toolu_3");
    kind(&mut c.app, "approval").await;
    c.lead_process.exit(1).await;
    let expired = item_where(&mut c.app, |item| item["body"]["status"] == "expired").await;
    assert_eq!(expired["item"]["body"]["kind"], "approval");
    let outcome = decision.await.expect("task");
    assert_eq!(outcome["behavior"], "deny");
}

#[tokio::test]
async fn api_errors_are_told_in_the_chat() {
    let mut c = two_bots().await;
    let line = talk_to_lead(&mut c, "Go").await;
    let lead = lead(&mut c).await;
    c.lead_process.emit(stream::replay(&line)).await;
    c.lead_process
        .emit(stream::api_error(
            "server_error",
            "API Error: 500 Internal Server Error",
        ))
        .await;
    c.lead_process.emit(stream::result(true)).await;
    let notice = kind(&mut c.app, "notice").await;
    assert_eq!(notice["item"]["body"]["level"], "error");
    assert_eq!(
        notice["item"]["body"]["text"],
        "API Error: 500 Internal Server Error"
    );
    let turn = kind(&mut c.app, "turn").await;
    assert_eq!(turn["item"]["body"]["error"], "api_error");
    c.t.until_state(&lead, BotState::Idle).await;
}
