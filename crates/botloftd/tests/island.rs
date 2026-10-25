//! What the island reads (spec 31): the cards that wait for the owner, as
//! the phone sees them but with no account, and what the crews did today.

mod common;

use std::time::Duration;

use botloft_core::protocol::BotState;
use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon, stream};
use serde_json::{Value, json};

/// A lead in a turn, ready to ask; also its MCP client for `ask_owner`.
async fn lead(
    t: &TestDaemon,
    app: &mut Client,
) -> (Value, botloftd::runtime::fake::FakeProcess, Mcp) {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, mcp) = ready_bot(t, app, &crew, "Lead").await;
    app.call("messages.send", json!({ "botId": bot["id"], "body": "Go" }))
        .await
        .expect("send");
    let lines = process.wait_lines(1).await;
    process
        .emit(stream::replay(lines.last().expect("line")))
        .await;
    (bot, process, mcp)
}

/// Claude Code asks for permission; the decision comes out of the handle.
async fn ask(
    process: &botloftd::runtime::fake::FakeProcess,
    mcp: &Mcp,
    id: &str,
    tool: &str,
    input: &Value,
) -> tokio::task::JoinHandle<Value> {
    process
        .emit(stream::tool_use(id, tool, input.clone()))
        .await;
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    let args = json!({ "tool_name": tool, "input": input, "tool_use_id": id });
    tokio::spawn(async move { mcp.tool("permission_prompt", args).await.expect("decision") })
}

fn cards_bot_of_question(cards: &Value) -> Value {
    cards["questions"][0]["bot"]["id"].clone()
}

/// `cards.list` once it has `approvals` requests and `questions` questions.
async fn cards_when(app: &mut Client, approvals: usize, questions: usize) -> Value {
    let mut cards = Value::Null;
    for _ in 0..200 {
        cards = app.call("cards.list", json!({})).await.expect("cards");
        let count = |name: &str| cards[name].as_array().map_or(0, Vec::len);
        if count("approvals") == approvals && count("questions") == questions {
            return cards;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("expected {approvals} requests and {questions} questions, got {cards}");
}

#[tokio::test]
async fn the_island_sees_what_waits_as_the_phone_does_without_an_account() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let (bot, process, mut mcp) = lead(&t, &mut app).await;

    let plain = json!({ "command": "git status", "description": "Shows what changed" });
    let first = ask(&process, &mcp, "toolu_1", "Bash", &plain).await;
    let durable = json!({ "title": "Fix it" });
    let second = ask(
        &process,
        &mcp,
        "toolu_2",
        "mcp__github__create_issue",
        &durable,
    )
    .await;
    let long = json!({ "command": format!("echo {}", "x".repeat(9000)) });
    let _third = ask(&process, &mcp, "toolu_3", "Bash", &long).await;
    mcp.tool(
        "ask_owner",
        json!({ "question": "Coffee?", "options": ["Yes", "No"] }),
    )
    .await
    .expect("asked");

    let cards = cards_when(&mut app, 3, 1).await;
    let approvals = cards["approvals"].as_array().expect("approvals");
    // Oldest first, each with what the owner needs to decide.
    assert_eq!(approvals[0]["text"], "git status");
    assert_eq!(approvals[0]["explanation"], "Shows what changed");
    assert_eq!(approvals[0]["bot"]["name"], "Lead");
    // The island opens this bot's chat from the card.
    assert_eq!(approvals[0]["bot"]["id"], bot["id"]);
    assert_eq!(cards_bot_of_question(&cards), bot["id"]);
    assert_eq!(approvals[0]["crew"], "Ops");
    assert_eq!(
        (
            approvals[0]["cut"].clone(),
            approvals[0]["atComputer"].clone()
        ),
        (json!(false), json!(false))
    );
    assert_eq!(approvals[1]["atComputer"], true);
    assert_eq!(approvals[2]["cut"], true);
    assert_eq!(cards["questions"][0]["text"], "Coffee?");
    assert_eq!(cards["questions"][0]["options"], json!(["Yes", "No"]));

    // Answered at the computer, a card leaves the list.
    app.call(
        "approvals.answer",
        json!({ "approvalId": approvals[0]["approvalId"], "allow": true }),
    )
    .await
    .expect("answer");
    assert_eq!(first.await.expect("task")["behavior"], "allow");
    let cards = cards_when(&mut app, 2, 1).await;
    assert_eq!(
        cards["approvals"][0]["approvalId"],
        approvals[1]["approvalId"]
    );
    app.call(
        "approvals.answer",
        json!({ "approvalId": approvals[1]["approvalId"], "allow": false }),
    )
    .await
    .expect("deny");
    assert_eq!(second.await.expect("task")["behavior"], "deny");
}

#[tokio::test]
async fn today_counts_the_edits_answers_and_who_worked_most() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let (bot, process, mcp) = lead(&t, &mut app).await;

    let edit = json!({
        "file_path": "C:\\work\\notes.md",
        "old_string": "TODO",
        "new_string": "First\nSecond",
    });
    let decision = ask(&process, &mcp, "toolu_1", "Edit", &edit).await;
    let cards = cards_when(&mut app, 1, 0).await;
    app.call(
        "approvals.answer",
        json!({ "approvalId": cards["approvals"][0]["approvalId"], "allow": true }),
    )
    .await
    .expect("answer");
    decision.await.expect("task");
    process
        .emit(stream::tool_result("toolu_1", "Updated", false))
        .await;
    process.emit(stream::result(false)).await;
    t.until_state(&bot, BotState::Idle).await;

    let today = app
        .call("activity.today", json!({ "since": 0 }))
        .await
        .expect("today");
    assert_eq!(today["filesEdited"], 1);
    assert_eq!(today["lines"], json!({ "added": 2, "removed": 1 }));
    assert_eq!(today["requestsAnswered"], 1);
    assert_eq!(today["busiest"]["name"], "Lead");
    assert_eq!(today["busiest"]["crew"], "Ops");
    assert_eq!(today["busiest"]["workMs"], 1234);

    // A day that starts later saw nothing.
    let later = app
        .call("activity.today", json!({ "since": i64::MAX / 2 }))
        .await
        .expect("later");
    assert_eq!(later["filesEdited"], 0);
    assert_eq!(later["requestsAnswered"], 0);
    assert_eq!(later["busiest"], Value::Null);
    let err = app
        .call("activity.today", json!({ "since": -1 }))
        .await
        .expect_err("refused");
    assert!(err.message.contains("since"), "{}", err.message);
}
