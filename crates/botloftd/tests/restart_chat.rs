//! What the chat shows across restarts. A request left open when the daemon
//! stopped (spec 10.1): its card stops offering an answer when the daemon
//! starts, and a card that still offered one shows how it ended once the
//! owner answers it. A new conversation (spec 7.3) leaves a line. A message
//! the bot never began goes out again (spec 9.1).

mod common;

use common::bots::{Crew, two_bots};
use common::mcp::Mcp;
use common::stream;
use serde_json::{Value, json};

async fn item_where(app: &mut common::Client, wanted: impl Fn(&Value) -> bool) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        if wanted(&changed["item"]) {
            return changed;
        }
    }
}

/// The lead asks to run a command; returns its card in the chat.
async fn open_request(c: &mut Crew) -> Value {
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": "Clean the build" }),
        )
        .await
        .expect("send");
    let line = c
        .lead_process
        .wait_lines(1)
        .await
        .last()
        .cloned()
        .expect("line");
    c.lead_process.emit(stream::replay(&line)).await;
    c.lead_process
        .emit(stream::tool_use(
            "toolu_1",
            "Bash",
            json!({ "command": "rm -rf build" }),
        ))
        .await;
    let mut mcp = Mcp::new(c.lead.addr, c.lead.token.clone());
    let args = json!({ "tool_name": "Bash", "input": { "command": "rm -rf build" }, "tool_use_id": "toolu_1" });
    tokio::spawn(async move { mcp.tool("permission_prompt", args).await });
    let card = item_where(&mut c.app, |item| item["body"]["kind"] == "approval").await;
    assert_eq!(card["item"]["body"]["status"], "pending");
    card
}

#[tokio::test]
async fn requests_left_by_the_last_run_stop_asking_in_the_chat() {
    let mut c = two_bots().await;
    open_request(&mut c).await;
    botloftd::approvals::expire_all(&c.t.daemon);
    let ended = item_where(&mut c.app, |item| item["body"]["kind"] == "approval").await;
    assert_eq!(ended["item"]["body"]["status"], "expired");
}

#[tokio::test]
async fn answering_a_card_that_already_ended_shows_how_it_ended() {
    let mut c = two_bots().await;
    let card = open_request(&mut c).await;
    // Expired without the chat hearing of it, as before this was fixed.
    c.t.daemon
        .store()
        .expire_approvals(None, botloft_core::now_ms())
        .expect("expire");
    let refused = c
        .app
        .call(
            "approvals.answer",
            json!({ "approvalId": card["item"]["body"]["approvalId"], "allow": true }),
        )
        .await
        .expect_err("already expired");
    assert!(refused.message.contains("already answered or expired"));
    let ended = item_where(&mut c.app, |item| item["body"]["kind"] == "approval").await;
    assert_eq!(ended["item"]["body"]["status"], "expired");
}

#[tokio::test]
async fn a_new_conversation_leaves_a_line_in_the_chat() {
    let mut c = two_bots().await;
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "bots.restart",
            json!({ "botId": lead["id"], "fresh": true }),
        )
        .await
        .expect("restart");
    let line = item_where(&mut c.app, |item| item["body"]["kind"] == "notice").await;
    assert_eq!(line["item"]["body"]["code"], "fresh_start");
    assert_eq!(line["item"]["botId"], lead["id"]);
}

/// A message written to a bot that stopped with the daemon before it began
/// it (no replay) is written again at start, without counting an attempt.
#[tokio::test]
async fn a_message_left_unread_by_the_last_run_is_written_again() {
    let mut c = two_bots().await;
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": "Check the logs" }),
        )
        .await
        .expect("send");
    let first = c.lead_process.wait_lines(1).await;
    assert!(first[0].to_string().contains("Check the logs"));

    botloftd::courier::requeue_left_behind(&c.t.daemon);
    let lines = c.lead_process.wait_lines(2).await;
    assert!(lines[1].to_string().contains("Check the logs"), "{lines:?}");
    let reopened = loop {
        let changed = c.app.notification("delivery.changed").await;
        if changed["state"] == "pending" && !changed["lastError"].is_null() {
            break changed;
        }
    };
    assert_eq!(reopened["attempts"], 0);
    assert_eq!(
        reopened["lastError"],
        "Botloft stopped before the agent began this message"
    );
}
