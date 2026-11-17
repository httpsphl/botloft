//! A bot searches its own past (spec 8.8): `search_history` finds words in
//! its own chat only, pages back, and reads the talk around a hit without
//! the tool calls.

mod common;

use std::time::Duration;

use common::Client;
use common::bots::{Crew, two_bots};
use common::stream;
use serde_json::{Value, json};

/// The texts of the messages and replies in `bot`'s chat.
async fn texts(app: &mut Client, bot: &Value) -> Vec<String> {
    let history = app
        .call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history");
    history
        .as_array()
        .expect("items")
        .iter()
        .filter_map(|item| {
            let body = &item["body"];
            body["text"]
                .as_str()
                .or_else(|| body["message"]["body"].as_str())
                .map(str::to_owned)
        })
        .collect()
}

async fn until_said(app: &mut Client, bot: &Value, text: &str) {
    for _ in 0..500 {
        if texts(app, bot).await.iter().any(|said| said == text) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("{text:?} never reached the chat");
}

/// @lead's chat: the owner's budget, a reply after reading a file, a note
/// from @writer and another reply. @writer's chat has the owner's secret.
async fn talked() -> (Crew, Value, Value) {
    let mut c = two_bots().await;
    let bots = c.app.call("bots.list", json!({})).await.expect("bots");
    let (lead, writer) = (bots[0].clone(), bots[1].clone());
    let budget =
        json!({ "botId": lead["id"], "body": "The budget for the Lisbon trip is 4200 euros" });
    c.app.call("messages.send", budget).await.expect("send");
    let lines = c.lead_process.wait_lines(1).await;
    let p = &c.lead_process;
    p.emit(stream::init("session-1")).await;
    p.emit(stream::replay(&lines[0])).await;
    let read = json!({ "file_path": "C:/trip/notes.md" });
    p.emit(stream::tool_use("toolu_1", "Read", read)).await;
    p.emit(stream::tool_result("toolu_1", "Lisbon notes", false))
        .await;
    p.emit(stream::text("Noted: the Lisbon budget is 4200."))
        .await;
    p.emit(stream::result(false)).await;
    until_said(&mut c.app, &lead, "Noted: the Lisbon budget is 4200.").await;

    let note = json!({ "to": "lead", "body": "Hotels in Lisbon are full in May" });
    c.writer.tool("send_message", note).await.expect("sent");
    let lines = c.lead_process.wait_lines(2).await;
    p.emit(stream::replay(&lines[1])).await;
    p.emit(stream::text("Thanks, I will look at June.")).await;
    p.emit(stream::result(false)).await;
    until_said(&mut c.app, &lead, "Thanks, I will look at June.").await;

    let secret =
        json!({ "botId": writer["id"], "body": "Lisbon is a surprise, keep it to yourself" });
    c.app.call("messages.send", secret).await.expect("send");
    (c, lead, writer)
}

fn told(items: &Value) -> Vec<(String, String)> {
    items
        .as_array()
        .expect("list")
        .iter()
        .map(|item| {
            (
                item["from"].as_str().expect("from").to_owned(),
                item["kind"].as_str().expect("kind").to_owned(),
            )
        })
        .collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter()
        .map(|(from, kind)| ((*from).to_owned(), (*kind).to_owned()))
        .collect()
}

#[tokio::test]
async fn finds_words_in_its_own_chat_only_and_pages_back() {
    let (mut c, _lead, _writer) = talked().await;
    let found = c
        .lead
        .tool("search_history", json!({ "query": "lisbon" }))
        .await
        .expect("found");
    assert_eq!(
        told(&found["hits"]),
        pairs(&[
            ("@writer", "message"),
            ("you", "reply"),
            ("owner", "message")
        ])
    );
    let hits = found["hits"].as_array().expect("hits");
    let excerpt = hits[2]["excerpt"].as_str().expect("excerpt");
    assert!(excerpt.contains("**Lisbon**"), "{excerpt}");
    assert!(excerpt.contains("4200 euros"), "{excerpt}");
    let at = hits[0]["at"].as_str().expect("at");
    assert_eq!((at.len(), &at[4..5], &at[13..14]), (16, "-", ":"), "{at}");
    assert_eq!(found["more"], false);
    assert!(found["note"].as_str().expect("note").contains("around"));

    // The owner's talk with @writer stays out of @lead's search.
    let secret = c
        .lead
        .tool("search_history", json!({ "query": "surprise" }))
        .await
        .expect("searched");
    assert_eq!(secret["hits"], json!([]));
    assert!(secret["note"].as_str().expect("note").contains("Nothing"));
    let mine = c
        .writer
        .tool("search_history", json!({ "query": "surpr" }))
        .await
        .expect("searched");
    assert_eq!(told(&mine["hits"]), pairs(&[("owner", "message")]));

    let first = c
        .lead
        .tool("search_history", json!({ "query": "lisbon", "limit": 1 }))
        .await
        .expect("page");
    assert_eq!(first["more"], true);
    let last = first["hits"][0]["item_id"].clone();
    let next = c
        .lead
        .tool(
            "search_history",
            json!({ "query": "Lisbon", "limit": 1, "before": last }),
        )
        .await
        .expect("next page");
    assert_eq!(told(&next["hits"]), pairs(&[("you", "reply")]));
}

#[tokio::test]
async fn reads_the_talk_around_a_hit_without_the_tools() {
    let (mut c, _lead, writer) = talked().await;
    let found = c
        .lead
        .tool("search_history", json!({ "query": "budget 4200" }))
        .await
        .expect("found");
    let owner_hit = found["hits"]
        .as_array()
        .expect("hits")
        .iter()
        .find(|hit| hit["from"] == "owner")
        .expect("the owner's message")["item_id"]
        .clone();

    let around = c
        .lead
        .tool("search_history", json!({ "around": owner_hit }))
        .await
        .expect("around");
    let items = &around["items"];
    assert_eq!(
        told(items),
        pairs(&[
            ("owner", "message"),
            ("you", "reply"),
            ("@writer", "message"),
            ("you", "reply"),
        ])
    );
    assert_eq!(items[0]["item_id"], owner_hit);
    assert_eq!(items[0]["this"], true);
    assert_eq!(items[1].get("this"), None);
    assert_eq!(items[1]["text"], "Noted: the Lisbon budget is 4200.");
    assert_eq!(items[2]["text"], "Hotels in Lisbon are full in May");

    // An item of another bot's chat is not there for @lead.
    let history = c
        .app
        .call("chat.history", json!({ "botId": writer["id"] }))
        .await
        .expect("history");
    let theirs = history[0]["id"].clone();
    let err = c
        .lead
        .tool("search_history", json!({ "around": theirs }))
        .await
        .expect_err("not in its chat");
    assert!(err.contains("is not in your chat"), "{err}");

    for (args, expected) in [
        (json!({ "around": owner_hit, "limit": 3 }), "go with query"),
        (json!({}), "pass either query"),
        (json!({ "query": "a" }), "at least 2"),
        (
            json!({ "query": "lisbon", "limit": 21 }),
            "between 1 and 20",
        ),
        (json!({ "around": "nope" }), "must be an item_id"),
    ] {
        let err = c
            .lead
            .tool("search_history", args)
            .await
            .expect_err(expected);
        assert!(err.contains(expected), "{err}");
    }
}
