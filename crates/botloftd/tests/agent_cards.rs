//! Cards a bot shows (spec 8.11): `show_card` puts a card in the chat and
//! shows it again in place as a new version; `chat.act` answers one version
//! with a button or the form, and the bot gets it as the owner's message.

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

fn report(extra: Value) -> Value {
    json!({
        "title": "Weekly sales",
        "blocks": [
            { "type": "text", "text": "Up **8%** this week." },
            { "type": "table", "columns": ["Item", "Sold"], "rows": [["Bread", 12], ["Cake", 3.5]] },
            { "type": "chart", "kind": "bar", "labels": ["Mon", "Tue"],
              "series": [{ "name": "Bread", "values": [5, null] }] },
            extra,
        ],
    })
}

fn buttons() -> Value {
    json!({ "type": "buttons", "buttons": [
        { "id": "publish", "label": "Publish" },
        { "id": "wait", "label": "Wait a day" },
    ] })
}

async fn card_item(app: &mut Client, bot: &Value, id: &str) -> Value {
    let history = app
        .call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history");
    history
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["id"] == id)
        .cloned()
        .expect("the card")
}

#[tokio::test]
async fn a_button_answers_one_version_and_reaches_the_bot() {
    let (_t, mut app, bot, process, mut mcp) = one_bot().await;
    let shown = mcp
        .tool("show_card", report(buttons()))
        .await
        .expect("shown");
    let id = shown["card_id"].as_str().expect("id").to_owned();
    assert!(id.starts_with("cht_"), "{id}");
    assert_eq!(shown["version"], 1);

    let card = card_item(&mut app, &bot, &id).await;
    assert_eq!(card["body"]["kind"], "card");
    assert_eq!(card["body"]["version"], 1);
    assert_eq!(card["body"]["title"], "Weekly sales");
    assert_eq!(
        card["body"]["blocks"][1]["rows"],
        json!([["Bread", "12"], ["Cake", "3.5"]])
    );
    assert_eq!(card["body"]["answer"], Value::Null);
    let lines = app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(lines[0]["lastActivity"]["kind"], "card");
    assert_eq!(lines[0]["lastActivity"]["text"], "Weekly sales");

    let answered = app
        .call(
            "chat.act",
            json!({ "itemId": id, "version": 1, "button": "publish" }),
        )
        .await
        .expect("pressed");
    assert_eq!(answered["body"]["answer"]["button"], "publish");
    assert_eq!(answered["body"]["answer"]["text"], "Publish");
    assert_eq!(answered["body"]["answer"]["version"], 1);

    let lines = process.wait_lines(1).await;
    assert_eq!(
        text_of(&lines[0]),
        format!("Answer on your card {id} \"Weekly sales\":\n\nPublish")
    );
    let history = app
        .call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history");
    assert_eq!(history[0]["body"]["kind"], "inbound");
    assert_eq!(history[0]["body"]["message"]["body"], "Publish");
    assert_eq!(history[0]["body"]["message"]["replyTo"]["itemId"], id);
    assert_eq!(
        history[0]["body"]["message"]["replyTo"]["text"],
        "Weekly sales"
    );

    let again = app
        .call(
            "chat.act",
            json!({ "itemId": id, "version": 1, "button": "wait" }),
        )
        .await
        .expect_err("one answer per version");
    assert_eq!(again.code, -32003);
    assert!(
        again.message.contains("already answered"),
        "{}",
        again.message
    );
}

#[tokio::test]
async fn showing_a_card_again_makes_a_new_version_in_place() {
    let (_t, mut app, bot, process, mut mcp) = one_bot().await;
    let shown = mcp
        .tool("show_card", report(buttons()))
        .await
        .expect("shown");
    let id = shown["card_id"].as_str().expect("id").to_owned();
    app.call(
        "chat.act",
        json!({ "itemId": id, "version": 1, "button": "wait" }),
    )
    .await
    .expect("pressed");
    process.wait_lines(1).await;

    let mut changed = report(buttons());
    changed["card"] = json!(id);
    changed["title"] = json!("Weekly sales, Tuesday");
    let again = mcp.tool("show_card", changed).await.expect("shown again");
    assert_eq!(again["card_id"], id);
    assert_eq!(again["version"], 2);
    let card = card_item(&mut app, &bot, &id).await;
    assert_eq!(card["body"]["version"], 2);
    assert_eq!(card["body"]["title"], "Weekly sales, Tuesday");
    assert_eq!(
        card["body"]["answer"],
        Value::Null,
        "a new version waits again"
    );

    let old = app
        .call(
            "chat.act",
            json!({ "itemId": id, "version": 1, "button": "publish" }),
        )
        .await
        .expect_err("an old version");
    assert_eq!(old.code, -32003);
    assert!(old.message.contains("changed this card"), "{}", old.message);
    app.call(
        "chat.act",
        json!({ "itemId": id, "version": 2, "button": "publish" }),
    )
    .await
    .expect("pressed on the new version");

    let err = mcp
        .tool("show_card", json!({ "card": "cht_nope", "title": "X", "blocks": [{ "type": "text", "text": "Hi" }] }))
        .await
        .expect_err("not a card id");
    assert!(err.contains("card_id show_card gave you"), "{err}");
    let err = mcp
        .tool(
            "show_card",
            json!({ "title": "X", "blocks": [{ "type": "video" }] }),
        )
        .await
        .expect_err("not a block");
    assert!(err.contains("block 1"), "{err}");
}

#[tokio::test]
async fn a_form_is_checked_and_reaches_the_bot_as_lines() {
    let (_t, mut app, _bot, process, mut mcp) = one_bot().await;
    let shown = mcp
        .tool(
            "show_card",
            json!({
                "title": "New client",
                "blocks": [{ "type": "form", "submit": "Create", "fields": [
                    { "id": "name", "label": "Name", "kind": "text", "required": true },
                    { "id": "size", "label": "Size", "kind": "choice", "options": ["Small", "Large"] },
                    { "id": "urgent", "label": "Urgent", "kind": "check" },
                ] }],
            }),
        )
        .await
        .expect("shown");
    let id = shown["card_id"].as_str().expect("id").to_owned();

    let err = app
        .call(
            "chat.act",
            json!({ "itemId": id, "version": 1, "form": { "name": "", "size": "Small" } }),
        )
        .await
        .expect_err("name is required");
    assert!(err.message.contains("must be filled in"), "{}", err.message);
    let err = app
        .call(
            "chat.act",
            json!({ "itemId": id, "version": 1, "button": "go" }),
        )
        .await
        .expect_err("no buttons");
    assert!(err.message.contains("no button"), "{}", err.message);

    app.call(
        "chat.act",
        json!({ "itemId": id, "version": 1,
                "form": { "name": "Acme", "size": "Large", "urgent": "true" } }),
    )
    .await
    .expect("sent");
    let lines = process.wait_lines(1).await;
    assert_eq!(
        text_of(&lines[0]),
        format!("Answer on your card {id} \"New client\":\n\nName: Acme\nSize: Large\nUrgent: ✓")
    );
}
