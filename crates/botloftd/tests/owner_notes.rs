//! Notes to the owner (spec 23.10): `tell_owner` leaves a note in the
//! owner's box and the bot's chat. It closes when the owner writes to that
//! bot, says Seen or replies to it, and a bot leaves at most ten an hour.

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

async fn open_items(app: &mut Client) -> Vec<Value> {
    let list = app.call("questions.list", json!({})).await.expect("list");
    list.as_array().expect("array").clone()
}

#[tokio::test]
async fn a_note_waits_in_the_box_until_the_owner_writes_to_its_bot() {
    let (_t, mut app, bot, process, mut mcp) = one_bot().await;

    let told = mcp
        .tool(
            "tell_owner",
            json!({ "note": "The **report** is in work/report.md." }),
        )
        .await
        .expect("told");
    let id = told["note_id"].as_str().expect("id").to_owned();
    assert!(id.starts_with("qst_"), "{id}");
    let said = told["note"].as_str().expect("note");
    assert!(said.contains(&format!("Reply to your note {id}")), "{said}");

    let changed = app.notification("question.changed").await;
    assert_eq!(changed["id"], id);
    assert_eq!(changed["kind"], "note");
    assert_eq!(changed["status"], "open");
    assert_eq!(changed["options"], json!([]));
    let lines = app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(lines[0]["lastActivity"]["kind"], "note");

    let asked = mcp
        .tool("ask_owner", json!({ "question": "Send it to Acme?" }))
        .await
        .expect("asked");
    app.notification("question.changed").await;
    let open = open_items(&mut app).await;
    assert_eq!(open.len(), 2);
    assert_eq!(open[0]["kind"], "question");
    assert_eq!(open[1]["kind"], "note");

    app.call(
        "messages.send",
        json!({ "botId": bot["id"], "body": "Thanks, reading it now." }),
    )
    .await
    .expect("sent");
    let closed = app.notification("question.changed").await;
    assert_eq!(closed["id"], id);
    assert_eq!(closed["status"], "dismissed");
    let open = open_items(&mut app).await;
    assert_eq!(open.len(), 1, "the question stays open");
    assert_eq!(open[0]["id"], asked["question_id"]);

    let lines = process.wait_lines(1).await;
    assert_eq!(text_of(&lines[0]), "Thanks, reading it now.");
    let history = app
        .call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history");
    let note = history
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["body"]["question"]["id"] == id)
        .expect("the note's card");
    assert_eq!(note["body"]["question"]["status"], "dismissed");
}

#[tokio::test]
async fn a_reply_to_a_note_reaches_the_bot_with_the_note_quoted() {
    let (_t, mut app, _bot, process, mut mcp) = one_bot().await;
    let told = mcp
        .tool(
            "tell_owner",
            json!({ "note": "The site is down since 3 am." }),
        )
        .await
        .expect("told");
    let id = told["note_id"].as_str().expect("id").to_owned();

    let replied = app
        .call(
            "questions.answer",
            json!({ "questionId": id, "answer": "Restart it, please." }),
        )
        .await
        .expect("replied");
    assert_eq!(replied["status"], "answered");
    assert!(open_items(&mut app).await.is_empty());

    let lines = process.wait_lines(1).await;
    assert_eq!(
        text_of(&lines[0]),
        format!("Reply to your note {id}: \"The site is down since 3 am.\"\n\nRestart it, please.")
    );
}

#[tokio::test]
async fn a_bot_leaves_at_most_ten_notes_an_hour() {
    let (_t, mut app, _bot, _process, mut mcp) = one_bot().await;
    let err = mcp
        .tool("tell_owner", json!({ "note": "  " }))
        .await
        .expect_err("empty");
    assert!(err.contains("must not be empty"), "{err}");
    let err = mcp
        .tool("tell_owner", json!({ "note": "Hi", "options": ["a", "b"] }))
        .await
        .expect_err("a note asks nothing");
    assert!(err.contains("invalid arguments"), "{err}");

    for n in 0..10 {
        mcp.tool("tell_owner", json!({ "note": format!("Note {n}.") }))
            .await
            .expect("told");
    }
    let first = open_items(&mut app).await.last().expect("first")["id"].clone();
    app.call("questions.dismiss", json!({ "questionId": first }))
        .await
        .expect("seen");
    let err = mcp
        .tool("tell_owner", json!({ "note": "One more." }))
        .await
        .expect_err("capped");
    assert!(err.contains("10 notes in the last hour"), "{err}");
    assert_eq!(
        open_items(&mut app).await.len(),
        9,
        "a seen note still counts"
    );

    mcp.tool("ask_owner", json!({ "question": "But may I still ask?" }))
        .await
        .expect("questions have their own cap");
}
