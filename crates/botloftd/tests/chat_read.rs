//! What the owner read (spec 15.1): the daemon keeps it, so the unread mark
//! of the conversation list does not come back when the app loses its own
//! storage.

mod common;

use botloft_core::ids::{BotId, ChatItemId};
use botloft_core::protocol::{ChatBody, ChatItem, ReplyItem};
use common::TestDaemon;
use common::bots::ready_bot;
use serde_json::{Value, json};

#[tokio::test]
async fn reading_a_chat_is_kept_in_the_bot() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, _) = ready_bot(&t, &mut app, &crew, "Lead").await;
    drop(process);
    let id: BotId = bot["id"].as_str().expect("id").parse().expect("bot id");
    assert_eq!(bot["readThrough"], Value::Null, "nothing read yet");

    // A reply at 5_000, then the owner reads the chat.
    t.daemon
        .store()
        .insert_chat_item(&ChatItem {
            id: ChatItemId::generate(),
            bot_id: id.clone(),
            body: ChatBody::Reply(ReplyItem {
                text: "Done.".into(),
            }),
            created_at: 5_000,
            updated_at: 5_000,
        })
        .expect("reply");
    app.call("chat.read", json!({ "botId": id }))
        .await
        .expect("read");

    let bots = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    let lead = bots
        .as_array()
        .expect("list")
        .iter()
        .find(|entry| entry["id"] == bot["id"])
        .expect("bot");
    assert_eq!(lead["readThrough"], json!(5_000));
    assert_eq!(lead["lastReplyAt"], json!(5_000));
}
