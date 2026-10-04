//! The owner's reactions to a bot's replies (spec 8.9): kept on the reply,
//! and read by the bot with the owner's next message, never on their own.

mod common;

use botloft_core::protocol::BotState;
use common::TestDaemon;
use common::bots::text_of;
use common::stream;
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;
const VALIDATION: i64 = -32004;

async fn reactions(app: &mut common::Client, bot: &Value) -> Vec<Value> {
    app.call("reactions.list", json!({ "botId": bot["id"] }))
        .await
        .expect("reactions")
        .as_array()
        .expect("list")
        .clone()
}

#[tokio::test]
async fn reactions_wait_for_the_owner_next_message() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let new_bot =
        |name: &str| json!({ "crewId": crew["id"], "name": name, "role": "", "instructions": "" });
    let scout = app
        .call("bots.create", new_bot("Scout"))
        .await
        .expect("bot");
    let writer = app
        .call("bots.create", new_bot("Writer"))
        .await
        .expect("bot");
    t.until_state(&scout, BotState::Idle).await;
    let process = t.process_of(&scout).await;

    app.call(
        "messages.send",
        json!({ "botId": scout["id"], "body": "Who signs?" }),
    )
    .await
    .expect("send");
    let line = process.wait_lines(1).await.remove(0);
    stream::answer(&process, &line, "Acme signs **annual**.").await;
    t.until_state(&scout, BotState::Idle).await;
    let history = app
        .call("chat.history", json!({ "botId": scout["id"] }))
        .await
        .expect("history");
    let items = history.as_array().expect("items");
    let find = |kind: &str| {
        items
            .iter()
            .find(|item| item["body"]["kind"] == kind)
            .expect("item")
            .clone()
    };
    let (reply, turn) = (find("reply"), find("turn"));
    let react = |bot: &Value, item: &Value, emoji: Value| json!({ "botId": bot["id"], "itemId": item["id"], "emoji": emoji });

    // Only one of the set, on a reply of this bot.
    for (params, code) in [
        (react(&scout, &reply, json!("🦄")), VALIDATION),
        (react(&scout, &turn, json!("👍")), VALIDATION),
        (react(&writer, &reply, json!("👍")), NOT_FOUND),
    ] {
        let refused = app
            .call("reactions.set", params)
            .await
            .expect_err("refused");
        assert_eq!(refused.code, code);
    }
    let put = app
        .call("reactions.set", react(&scout, &reply, json!("👍")))
        .await
        .expect("react");
    assert_eq!(put["emoji"], "👍");
    assert_eq!(put["sentIn"], Value::Null);
    // The bot is not woken for it.
    assert_eq!(process.input_lines().len(), 1);

    // The owner's next message takes it along, once.
    let next = app
        .call(
            "messages.send",
            json!({ "botId": scout["id"], "body": "Next?" }),
        )
        .await
        .expect("send");
    let line = process.wait_lines(2).await.remove(1);
    assert_eq!(
        text_of(&line),
        "Reacted 👍 to: \"Acme signs **annual**.\"\n\nNext?"
    );
    assert_eq!(reactions(&mut app, &scout).await[0]["sentIn"], next["id"]);
    stream::answer(&process, &line, "Noted.").await;
    t.until_state(&scout, BotState::Idle).await;
    app.call(
        "messages.send",
        json!({ "botId": scout["id"], "body": "And?" }),
    )
    .await
    .expect("send");
    assert_eq!(text_of(&process.wait_lines(3).await.remove(2)), "And?");

    // Changed, it waits again; taken off, it is gone.
    app.call("reactions.set", react(&scout, &reply, json!("🎉")))
        .await
        .expect("change");
    let listed = reactions(&mut app, &scout).await;
    assert_eq!(
        (listed[0]["emoji"].clone(), listed[0]["sentIn"].clone()),
        (json!("🎉"), Value::Null)
    );
    let off = app
        .call("reactions.set", react(&scout, &reply, Value::Null))
        .await
        .expect("off");
    assert_eq!(off, Value::Null);
    assert!(reactions(&mut app, &scout).await.is_empty());
}
