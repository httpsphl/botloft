//! `chat.search` (spec 8.8): finding words in the chats, and opening a
//! chat at what was found with `chat.history {until}`.

mod common;

use common::TestDaemon;
use serde_json::{Value, json};

async fn setup() -> (TestDaemon, common::Client, Value, Value) {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    for body in [
        "Previsão do tempo para sábado, por favor",
        "Then summarize the weather report",
        "Thanks!",
    ] {
        app.call("messages.send", json!({ "botId": bot["id"], "body": body }))
            .await
            .expect("sent");
    }
    (t, app, crew, bot)
}

#[tokio::test]
async fn words_are_found_with_a_marked_snippet() {
    let (_t, mut app, crew, bot) = setup().await;
    let hits = app
        .call("chat.search", json!({ "query": "previsao SÁB" }))
        .await
        .expect("search");
    let hits = hits.as_array().expect("hits");
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0]["crewId"], crew["id"]);
    assert_eq!(hits[0]["item"]["botId"], bot["id"]);
    assert_eq!(hits[0]["item"]["body"]["kind"], "inbound");
    let snippet = hits[0]["snippet"].as_str().expect("snippet");
    assert!(snippet.contains("\u{2}Previsão\u{3}"), "{snippet:?}");
    assert!(snippet.contains("\u{2}sábado\u{3}"), "{snippet:?}");

    let only_crew = app
        .call(
            "chat.search",
            json!({ "query": "weather", "crewId": crew["id"], "limit": 1 }),
        )
        .await
        .expect("crew search");
    assert_eq!(only_crew.as_array().map(Vec::len), Some(1));

    for query in ["a", "  ", "- !"] {
        let err = app
            .call("chat.search", json!({ "query": query }))
            .await
            .expect_err("too short");
        assert_eq!(err.code, -32004, "{query:?}");
    }
}

#[tokio::test]
async fn the_chat_opens_at_a_result() {
    let (_t, mut app, _crew, bot) = setup().await;
    let hits = app
        .call("chat.search", json!({ "query": "weather" }))
        .await
        .expect("search");
    let found = hits[0]["item"]["id"].clone();
    let items = app
        .call(
            "chat.history",
            json!({ "botId": bot["id"], "until": found }),
        )
        .await
        .expect("history");
    let items = items.as_array().expect("items");
    assert_eq!(items.len(), 2, "the result and the message after it");
    assert_eq!(items[1]["id"], found);

    let err = app
        .call(
            "chat.history",
            json!({ "botId": bot["id"], "until": found, "limit": 5 }),
        )
        .await
        .expect_err("until with limit");
    assert_eq!(err.code, -32004);
}

#[tokio::test]
async fn archived_crews_are_not_searched() {
    let (_t, mut app, crew, _bot) = setup().await;
    app.call("crews.archive", json!({ "crewId": crew["id"] }))
        .await
        .expect("archived");
    let hits = app
        .call("chat.search", json!({ "query": "weather" }))
        .await
        .expect("search");
    assert_eq!(hits, json!([]));
}
