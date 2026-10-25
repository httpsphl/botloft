//! A bot asks the owner for a key (spec 32.5): `ask_key` returns at once,
//! the request is a card in its chat, and the owner's answer reaches the bot
//! as a message from Botloft, after the restart that brings the key.

mod common;

use common::bots::{ready_bot, text_of};
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

const TOKEN: &str = "ghp-live-1234567890abcdef";

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

async fn card(app: &mut Client, bot: &Value) -> Value {
    let history = app
        .call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history");
    history
        .as_array()
        .expect("array")
        .iter()
        .find(|item| item["body"]["kind"] == "key_request")
        .cloned()
        .expect("a key request in the chat")
}

#[tokio::test]
async fn a_saved_key_closes_the_request_and_the_restarted_bot_is_told() {
    let (t, mut app, bot, first, mut mcp) = one_bot().await;

    let asked = mcp
        .tool(
            "ask_key",
            json!({ "name": "GITHUB_TOKEN", "why": "To open the pull request" }),
        )
        .await
        .expect("asked");
    let id = asked["request_id"].as_str().expect("id").to_owned();
    assert!(id.starts_with("cht_"), "{id}");
    let shown = card(&mut app, &bot).await;
    assert_eq!(shown["id"], id);
    assert_eq!(shown["body"]["name"], "GITHUB_TOKEN");
    assert_eq!(shown["body"]["status"], "open");
    let lines = app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(lines[0]["lastActivity"]["kind"], "question");

    // The key must be the one asked for.
    let wrong = app
        .call(
            "keys.set",
            json!({ "botId": bot["id"], "name": "OTHER_TOKEN", "value": TOKEN, "requestId": id }),
        )
        .await
        .expect_err("another name");
    assert!(wrong.message.contains("GITHUB_TOKEN"), "{}", wrong.message);

    app.call(
        "keys.set",
        json!({ "botId": bot["id"], "name": "GITHUB_TOKEN", "value": TOKEN, "requestId": id }),
    )
    .await
    .expect("saved");
    assert_eq!(card(&mut app, &bot).await["body"]["status"], "saved");

    for _ in 0..500 {
        if first.killed() && t.runtime.processes().len() > 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    assert!(first.killed(), "the bot restarts with its key");
    let second = t.process_of(&bot).await;
    assert_eq!(second.env("GITHUB_TOKEN").as_deref(), Some(TOKEN));
    let lines = second.wait_lines(1).await;
    let text = text_of(&lines[0]);
    assert!(text.contains("saved the key GITHUB_TOKEN"), "{text}");
    assert!(!text.contains(TOKEN), "{text}");

    let again = app
        .call(
            "keys.set",
            json!({ "botId": bot["id"], "name": "GITHUB_TOKEN", "value": TOKEN, "requestId": id }),
        )
        .await
        .expect_err("closed");
    assert_eq!(again.code, -32003);
}

#[tokio::test]
async fn not_now_closes_the_request_and_tells_the_bot() {
    let (_t, mut app, bot, process, mut mcp) = one_bot().await;
    let asked = mcp
        .tool(
            "ask_key",
            json!({ "name": "STRIPE_KEY", "why": "To read last month's invoices" }),
        )
        .await
        .expect("asked");
    app.call("keys.dismiss", json!({ "requestId": asked["request_id"] }))
        .await
        .expect("dismissed");
    assert_eq!(card(&mut app, &bot).await["body"]["status"], "dismissed");
    let lines = process.wait_lines(1).await;
    let text = text_of(&lines[0]);
    assert!(text.contains("did not give the key STRIPE_KEY"), "{text}");
    let keys = app
        .call("keys.list", json!({ "botId": bot["id"] }))
        .await
        .expect("keys");
    assert_eq!(keys, json!([]));
}

#[tokio::test]
async fn bad_requests_come_back_to_the_bot_and_open_ones_are_capped() {
    let (_t, _app, _bot, _process, mut mcp) = one_bot().await;
    for (args, says) in [
        (json!({ "name": "lower", "why": "x" }), "capital letters"),
        (json!({ "name": "PATH", "why": "x" }), "used by the system"),
        (json!({ "name": "A_KEY", "why": "  " }), "what you need"),
        (
            json!({ "name": "A_KEY", "why": "x".repeat(501) }),
            "at most 500",
        ),
        (
            json!({ "name": "A_KEY", "why": "x", "value": "y" }),
            "invalid arguments",
        ),
    ] {
        let err = mcp.tool("ask_key", args).await.expect_err("refused");
        assert!(err.contains(says), "{err}");
    }
    for n in 0..5 {
        mcp.tool(
            "ask_key",
            json!({ "name": format!("KEY_{n}"), "why": "For a test" }),
        )
        .await
        .expect("asked");
    }
    let err = mcp
        .tool("ask_key", json!({ "name": "KEY_5", "why": "For a test" }))
        .await
        .expect_err("capped");
    assert!(err.contains("wait for an answer"), "{err}");
}
