//! Questions to the owner (spec 23): `ask_owner` returns at once, the
//! question waits in the box and the chat, and the owner's answer reaches
//! the bot as a message that quotes it.

mod common;

use common::bots::{ready_bot, text_of};
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

async fn one_bot() -> (
    TestDaemon,
    Client,
    Value,
    botloftd::runtime::fake::FakeProcess,
    Mcp,
) {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    (t, app, bot, process, mcp)
}

async fn open_questions(app: &mut Client) -> Vec<Value> {
    let list = app.call("questions.list", json!({})).await.expect("list");
    list.as_array().expect("array").clone()
}

#[tokio::test]
async fn the_bot_asks_without_waiting_and_the_answer_comes_back_as_a_message() {
    let (_t, mut app, bot, process, mut mcp) = one_bot().await;

    let asked = mcp
        .tool(
            "ask_owner",
            json!({ "question": "Which **client** first?", "options": ["Acme", "Globex"] }),
        )
        .await
        .expect("asked");
    let id = asked["question_id"].as_str().expect("id").to_owned();
    assert!(id.starts_with("qst_"), "{id}");
    let note = asked["note"].as_str().expect("note");
    assert!(
        note.contains(&format!("Answer to your question {id}")),
        "{note}"
    );

    let changed = app.notification("question.changed").await;
    assert_eq!(changed["id"], id);
    assert_eq!(changed["status"], "open");
    let open = open_questions(&mut app).await;
    assert_eq!(open.len(), 1);
    assert_eq!(open[0]["botId"], bot["id"]);
    assert_eq!(open[0]["options"], json!(["Acme", "Globex"]));

    let history = app
        .call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history");
    let card = &history[0];
    assert_eq!(card["body"]["kind"], "question");
    assert_eq!(card["body"]["question"]["text"], "Which **client** first?");
    let lines = app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(lines[0]["lastActivity"]["kind"], "question");

    let answered = app
        .call(
            "questions.answer",
            json!({ "questionId": id, "answer": " Globex " }),
        )
        .await
        .expect("answered");
    assert_eq!(answered["status"], "answered");
    assert_eq!(answered["answer"], "Globex");
    assert!(open_questions(&mut app).await.is_empty());

    let lines = process.wait_lines(1).await;
    let text = text_of(&lines[0]);
    assert_eq!(
        text,
        format!("Answer to your question {id}: \"Which **client** first?\"\n\nGlobex")
    );

    let history = app
        .call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history");
    assert_eq!(history[0]["body"]["kind"], "inbound");
    assert_eq!(history[0]["body"]["message"]["questionId"], id);
    assert_eq!(history[0]["body"]["message"]["fromKind"], "owner");
    assert_eq!(history[1]["body"]["question"]["status"], "answered");

    let again = app
        .call(
            "questions.answer",
            json!({ "questionId": id, "answer": "Acme" }),
        )
        .await
        .expect_err("closed");
    assert_eq!(again.code, -32003);
}

#[tokio::test]
async fn a_dismissed_question_closes_without_telling_the_bot() {
    let (_t, mut app, _bot, process, mut mcp) = one_bot().await;
    let asked = mcp
        .tool("ask_owner", json!({ "question": "Coffee?" }))
        .await
        .expect("asked");
    let dismissed = app
        .call(
            "questions.dismiss",
            json!({ "questionId": asked["question_id"] }),
        )
        .await
        .expect("dismissed");
    assert_eq!(dismissed["status"], "dismissed");
    assert_eq!(dismissed["answer"], Value::Null);
    assert!(open_questions(&mut app).await.is_empty());
    let all = app
        .call("questions.list", json!({ "status": "dismissed" }))
        .await
        .expect("list");
    assert_eq!(all.as_array().expect("array").len(), 1);
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    assert!(process.input_lines().is_empty(), "the bot is not told");
}

#[tokio::test]
async fn bad_questions_come_back_to_the_bot_and_open_ones_are_capped() {
    let (_t, mut app, _bot, _process, mut mcp) = one_bot().await;
    for (args, says) in [
        (json!({ "question": "  " }), "must not be empty"),
        (json!({ "question": "x".repeat(2001) }), "at most 2000"),
        (
            json!({ "question": "Pick", "options": ["Only one"] }),
            "2 to 5",
        ),
        (
            json!({ "question": "Pick", "options": ["A", "a"] }),
            "repeated",
        ),
        (
            json!({ "question": "Pick", "options": ["A", "B\nC"] }),
            "one line",
        ),
        (
            json!({ "question": "Pick", "extra": 1 }),
            "invalid arguments",
        ),
    ] {
        let err = mcp.tool("ask_owner", args).await.expect_err("refused");
        assert!(err.contains(says), "{err}");
    }
    assert!(open_questions(&mut app).await.is_empty());

    for n in 0..5 {
        mcp.tool("ask_owner", json!({ "question": format!("Question {n}?") }))
            .await
            .expect("asked");
    }
    let err = mcp
        .tool("ask_owner", json!({ "question": "One more?" }))
        .await
        .expect_err("capped");
    assert!(err.contains("wait for an answer"), "{err}");
    assert_eq!(open_questions(&mut app).await.len(), 5);
}

#[tokio::test]
async fn questions_of_an_archived_bot_leave_the_box_and_cannot_be_answered() {
    let (_t, mut app, bot, _process, mut mcp) = one_bot().await;
    let asked = mcp
        .tool("ask_owner", json!({ "question": "Still there?" }))
        .await
        .expect("asked");
    app.call("bots.archive", json!({ "botId": bot["id"] }))
        .await
        .expect("archived");
    assert!(open_questions(&mut app).await.is_empty());
    let err = app
        .call(
            "questions.answer",
            json!({ "questionId": asked["question_id"], "answer": "Yes" }),
        )
        .await
        .expect_err("archived");
    assert_eq!(err.code, -32003);

    app.call("bots.delete", json!({ "botId": bot["id"] }))
        .await
        .expect("deleted");
    let err = app
        .call(
            "questions.dismiss",
            json!({ "questionId": asked["question_id"] }),
        )
        .await
        .expect_err("gone");
    assert_eq!(err.code, -32002);
}
