//! The effort and the context over the app's protocol (spec 11.2, 11.3):
//! `bots.setEffort`, `bots.compact` and the `bot.context` notification.

mod common;

use botloft_core::protocol::BotState;
use common::TestDaemon;
use common::bots::ready_bot;
use serde_json::json;

const CONFLICT: i64 = -32003;
const INVALID_PARAMS: i64 = -32602;

#[tokio::test]
async fn the_app_sets_the_effort_and_compacts_the_conversation() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, _) = ready_bot(&t, &mut app, &crew, "Scout").await;
    assert_eq!(bot["effort"], "default");
    assert_eq!(bot["effortDefault"], json!(null));
    assert_eq!(bot["context"], json!(null));

    // Claude Code tells how full the conversation is; the app hears it.
    process
        .answer_control(
            "get_context_usage",
            json!({ "totalTokens": 556_000, "maxTokens": 1_000_000,
                    "autoCompactThreshold": 967_000, "isAutoCompactEnabled": true }),
        )
        .await;
    let told = app.notification("bot.context").await;
    assert_eq!(told["botId"], bot["id"]);
    assert_eq!(
        told["context"],
        json!({ "usedTokens": 556_000, "windowTokens": 1_000_000, "autoCompactTokens": 967_000,
                "compacting": false, "updatedAt": told["context"]["updatedAt"] })
    );
    let listed = app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(listed[0]["context"]["usedTokens"], 556_000);

    let compacting = app
        .call("bots.compact", json!({ "botId": bot["id"] }))
        .await
        .expect("compact");
    assert_eq!(compacting["context"]["compacting"], true);
    assert_eq!(
        app.notification("bot.context").await["context"]["compacting"],
        true
    );

    let failure = app
        .call(
            "bots.setEffort",
            json!({ "botId": bot["id"], "effort": "turbo" }),
        )
        .await
        .expect_err("an unknown level");
    assert_eq!(failure.code, INVALID_PARAMS);
    let changed = app
        .call(
            "bots.setEffort",
            json!({ "botId": bot["id"], "effort": "xhigh" }),
        )
        .await
        .expect("set effort");
    assert_eq!(changed["effort"], "xhigh");

    app.call(
        "bots.setPaused",
        json!({ "botId": bot["id"], "paused": true }),
    )
    .await
    .expect("pause");
    t.until_state(&bot, BotState::Offline).await;
    let failure = app
        .call("bots.compact", json!({ "botId": bot["id"] }))
        .await
        .expect_err("a paused bot");
    assert_eq!(failure.code, CONFLICT);
}
