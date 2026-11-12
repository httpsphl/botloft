//! An agent's own search (spec 8.8) across its conversations (spec 40):
//! each hit says which one it is in, and the talk around a hit is read in
//! that conversation only.

mod common;

use std::time::Duration;

use common::TestDaemon;
use common::conversations::{holding, open, send, sleeping_bot, turn, until_reply};
use common::mcp::Mcp;
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn the_agent_s_search_says_which_conversation_and_reads_around_in_it() {
    let t = TestDaemon::start_supervised().await;
    t.daemon
        .settings
        .set_sleep_after(Some(Duration::from_secs(600)));
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let trip = open(&mut app, &id).await;
    let rename = json!({ "conversationId": trip, "title": "Trip plans" });
    app.call("conversations.rename", rename)
        .await
        .expect("rename");

    send(&mut app, &bot, "The Lisbon budget is 800 euros", None).await;
    let (main, line) = holding(&t, 1, "Lisbon budget").await;
    turn(&main, &line, "session-main", "Noted: 800 euros for Lisbon.").await;
    send(&mut app, &bot, "Find hotels in Lisbon", Some(&trip)).await;
    let (other, line) = holding(&t, 2, "hotels in Lisbon").await;
    turn(
        &other,
        &line,
        "session-trip",
        "Three hotels in Lisbon look good.",
    )
    .await;
    until_reply(
        &mut app,
        &bot,
        Some(&trip),
        "Three hotels in Lisbon look good.",
    )
    .await;

    let token = other.env("BOTLOFT_BOT_TOKEN").expect("token");
    let mut mcp = Mcp::new(t.addr, token);
    let found = mcp
        .tool("search_history", json!({ "query": "Lisbon" }))
        .await
        .expect("search");
    let hits = found["hits"].as_array().expect("hits");
    let in_of = |kind: &str| -> Vec<Value> {
        hits.iter()
            .filter(|hit| hit["conversation"] == kind)
            .cloned()
            .collect()
    };
    assert_eq!(in_of("main").len(), 2, "{found}");
    let trip_hits = in_of("Trip plans");
    assert_eq!(trip_hits.len(), 2, "{found}");

    let around = mcp
        .tool(
            "search_history",
            json!({ "around": trip_hits[0]["item_id"] }),
        )
        .await
        .expect("around");
    assert_eq!(around["conversation"], "Trip plans");
    let text = around["items"].to_string();
    assert!(text.contains("Find hotels in Lisbon"), "{around}");
    assert!(!text.contains("800 euros"), "{around}");
}
