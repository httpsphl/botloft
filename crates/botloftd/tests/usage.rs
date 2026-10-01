//! Tokens per turn and per bot (spec 8.7), end to end.

mod common;

use botloft_core::protocol::BotState;
use common::bots::two_bots;
use common::stream;
use serde_json::json;

#[tokio::test]
async fn each_turn_keeps_its_tokens_and_the_bots_add_them_up() {
    let mut c = two_bots().await;
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    for body in ["First", "Second"] {
        c.app
            .call(
                "messages.send",
                json!({ "botId": lead["id"], "body": body }),
            )
            .await
            .expect("send");
        let lines = c.lead_process.wait_lines(1).await;
        let line = lines.last().cloned().expect("line");
        stream::answer(&c.lead_process, &line, "Done.").await;
        c.t.until_state(&lead, BotState::Idle).await;
    }

    let history = c
        .app
        .call("chat.history", json!({ "botId": lead["id"], "limit": 1 }))
        .await
        .expect("history");
    assert_eq!(
        history[0]["body"]["tokens"],
        json!({ "input": 12, "cacheWrite": 300, "reloaded": 0, "cacheRead": 4000, "output": 45 }),
        "the turn's own tokens, from the result's usage"
    );

    let usage = c
        .app
        .call("usage.tokens", json!({ "since": 0 }))
        .await
        .expect("usage");
    let bots = usage.as_array().expect("bots");
    assert_eq!(bots.len(), 1, "only bots that worked: {usage}");
    assert_eq!(bots[0]["botId"], lead["id"]);
    assert_eq!(bots[0]["name"], lead["name"]);
    assert_eq!(bots[0]["turns"], 2);
    assert_eq!(
        bots[0]["tokens"],
        json!({ "input": 24, "cacheWrite": 600, "reloaded": 0, "cacheRead": 8000, "output": 90 })
    );

    assert!(
        c.app
            .call("usage.tokens", json!({ "since": -1 }))
            .await
            .is_err()
    );
}
