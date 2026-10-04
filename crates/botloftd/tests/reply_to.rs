//! The owner replying to something in a bot's chat (spec 9.3): the message
//! keeps what it quotes, and the bot reads the quote above the words.

mod common;

use botloft_core::protocol::BotState;
use common::TestDaemon;
use common::bots::text_of;
use common::stream;
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;
const VALIDATION: i64 = -32004;

async fn history(app: &mut common::Client, bot: &Value) -> Vec<Value> {
    app.call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history")
        .as_array()
        .expect("items")
        .clone()
}

fn of_kind<'a>(items: &'a [Value], kind: &str) -> &'a Value {
    items
        .iter()
        .find(|item| item["body"]["kind"] == kind)
        .expect("item of kind")
}

#[tokio::test]
async fn a_reply_quotes_the_item_for_the_bot_and_keeps_it() {
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

    // Scout answers a first message.
    app.call(
        "messages.send",
        json!({ "botId": scout["id"], "body": "Who signs?" }),
    )
    .await
    .expect("send");
    let line = process.wait_lines(1).await.remove(0);
    stream::answer(&process, &line, "Acme signs   monthly.\nDana approves.").await;
    t.until_state(&scout, BotState::Idle).await;
    let items = history(&mut app, &scout).await;
    let reply = of_kind(&items, "reply");

    // The owner replies to it.
    let params = json!({ "botId": scout["id"], "body": "Make it yearly", "replyTo": reply["id"] });
    let message = app.call("messages.send", params).await.expect("reply");
    assert_eq!(message["replyTo"]["itemId"], reply["id"]);
    assert_eq!(
        message["replyTo"]["text"],
        "Acme signs monthly. Dana approves."
    );
    let line = process.wait_lines(2).await.remove(1);
    assert_eq!(
        text_of(&line),
        "Replying to: \"Acme signs monthly. Dana approves.\"\n\nMake it yearly"
    );

    // It stays with the message, as the chat and the lists read it.
    let items = history(&mut app, &scout).await;
    assert_eq!(
        items[0]["body"]["message"]["replyTo"]["itemId"],
        reply["id"]
    );
    let listed = app
        .call("messages.list", json!({ "botId": scout["id"] }))
        .await
        .expect("list");
    assert_eq!(
        listed[0]["replyTo"]["text"],
        "Acme signs monthly. Dana approves."
    );

    // Only an item of this bot's chat, with words.
    let refused = |item: &Value, to: &Value| json!({ "botId": to["id"], "body": "Hm", "replyTo": item["id"] });
    for (params, expected) in [
        (refused(reply, &writer), NOT_FOUND),
        (refused(of_kind(&items, "turn"), &scout), VALIDATION),
    ] {
        let code = app
            .call("messages.send", params)
            .await
            .expect_err("refused")
            .code;
        assert_eq!(code, expected);
    }
}
