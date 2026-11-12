//! The owner's side of conversations (spec 40.5): listing, opening, naming
//! and archiving them, the agent's state as their summary, and the controls
//! that act on one of them.

mod common;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::ConversationId;
use botloft_core::protocol::BotState;
use botloft_core::protocol::error_code::{NOT_FOUND, VALIDATION};
use common::conversations::{begin, holding, open, send, sleeping_bot, turn, until_reply};
use common::{Client, TestDaemon};
use serde_json::{Value, json};

/// A daemon whose bots sleep after ten idle minutes, with the loop that
/// tells the app about conversations.
async fn start() -> TestDaemon {
    let t = TestDaemon::start_supervised().await;
    t.daemon
        .settings
        .set_sleep_after(Some(Duration::from_secs(600)));
    tokio::spawn(botloftd::service::conversations::follow(Arc::clone(
        &t.daemon,
    )));
    t
}

/// The next `conversation.changed` about `id` that `matches`.
async fn changed(app: &mut Client, id: &ConversationId, matches: impl Fn(&Value) -> bool) -> Value {
    let wait = async {
        loop {
            let shown = app.notification("conversation.changed").await;
            if shown["id"] == id.as_str() && matches(&shown) {
                return shown;
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(5), wait)
        .await
        .expect("conversation.changed")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_opens_names_and_archives_conversations() {
    let t = start().await;
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let main = ConversationId::main_of(&id);
    let list = async |app: &mut Client, method: &str| {
        let listed = app
            .call(method, json!({ "botId": id }))
            .await
            .expect("list");
        listed.as_array().expect("list").clone()
    };
    let listed = list(&mut app, "conversations.list").await;
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["id"], main.as_str());
    assert_eq!(listed[0]["main"], true);

    let other = open(&mut app, &id).await;
    let shown = changed(&mut app, &other, |_| true).await;
    assert_eq!(shown["state"], "sleeping");
    assert_eq!(shown["title"], Value::Null);

    let named = json!({ "conversationId": other, "title": "Trip plans" });
    let renamed = app
        .call("conversations.rename", named)
        .await
        .expect("rename");
    assert_eq!(renamed["title"], "Trip plans");
    assert_eq!(renamed["titleBy"], "owner");
    // The main one keeps its name and its place.
    let main_named = json!({ "conversationId": main, "title": "Mine" });
    let refused = app.call("conversations.rename", main_named).await;
    assert_eq!(refused.expect_err("main").code, i64::from(VALIDATION));
    let main_only = json!({ "conversationId": main });
    let refused = app.call("conversations.archive", main_only).await;
    assert_eq!(refused.expect_err("main").code, i64::from(VALIDATION));

    let archived = app
        .call("conversations.archive", json!({ "conversationId": other }))
        .await
        .expect("archive");
    assert_eq!(archived["state"], "archived");
    assert!(archived["archivedAt"].is_number());
    assert_eq!(list(&mut app, "conversations.list").await.len(), 1);
    assert_eq!(list(&mut app, "conversations.archived").await.len(), 1);
    // It takes nothing while archived.
    let to_archived = json!({ "botId": bot["id"], "body": "Hi", "conversationId": other });
    let refused = app.call("messages.send", to_archived).await;
    assert_eq!(refused.expect_err("archived").code, i64::from(NOT_FOUND));

    let back = app
        .call(
            "conversations.unarchive",
            json!({ "conversationId": other }),
        )
        .await
        .expect("unarchive");
    assert_eq!(back["archivedAt"], Value::Null);
    let listed = list(&mut app, "conversations.list").await;
    assert_eq!(listed.len(), 2);
    assert_eq!(listed[0]["id"], main.as_str());
    assert_eq!(listed[1]["title"], "Trip plans");
}

#[tokio::test(flavor = "multi_thread")]
async fn each_conversation_has_its_own_state_and_stop() {
    let t = start().await;
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let main = ConversationId::main_of(&id);
    let other = open(&mut app, &id).await;

    send(&mut app, &bot, "Plan the trip", Some(&other)).await;
    let (process, line) = holding(&t, 1, "Plan the trip").await;
    begin(&process, &line, "session-other").await;
    // The agent works while one of its conversations does; the main one
    // still sleeps.
    t.until_state(&bot, BotState::Busy).await;
    changed(&mut app, &other, |shown| shown["state"] == "busy").await;
    assert!(matches!(
        t.daemon.supervisor.status_of(&main),
        Some((BotState::Sleeping, _))
    ));

    // Stop acts on the conversation it names: the main one has nothing.
    let interrupts = || {
        process
            .control_lines()
            .iter()
            .filter(|line| line["request"]["subtype"] == "interrupt")
            .count()
    };
    app.call("bots.interrupt", json!({ "botId": id }))
        .await
        .expect("main");
    assert_eq!(interrupts(), 0);
    let stop_other = json!({ "botId": id, "conversationId": other });
    app.call("bots.interrupt", stop_other).await.expect("other");
    assert_eq!(interrupts(), 1);

    process.emit(common::stream::result(false)).await;
    // Ready again, though its main conversation sleeps.
    t.until_state(&bot, BotState::Idle).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_message_left_for_an_archived_conversation_goes_to_the_main_one() {
    let t = start().await;
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let other = open(&mut app, &id).await;
    let paused = |paused: bool| json!({ "botId": id, "paused": paused });
    app.call("bots.setPaused", paused(true))
        .await
        .expect("pause");
    send(&mut app, &bot, "Still there?", Some(&other)).await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    app.call("conversations.archive", json!({ "conversationId": other }))
        .await
        .expect("archive");
    app.call("bots.setPaused", paused(false))
        .await
        .expect("resume");
    // The courier tries again a moment later, and wakes the main one.
    t.clock.advance(Duration::from_secs(6));
    t.daemon.courier.wake();

    let (process, line) = holding(&t, 1, "Still there?").await;
    turn(&process, &line, "session-main", "Here").await;
    until_reply(&mut app, &bot, None, "Here").await;
}

#[tokio::test(flavor = "multi_thread")]
async fn compacting_one_conversation_wakes_only_that_one() {
    let t = start().await;
    let mut app = t.session().await;
    let (_bot, id) = sleeping_bot(&t, &mut app).await;
    let other = open(&mut app, &id).await;
    let compact = json!({ "botId": id, "conversationId": other });
    app.call("bots.compact", compact).await.expect("compact");

    let (process, _) = holding(&t, 1, "/compact").await;
    let main = ConversationId::main_of(&id);
    assert!(matches!(
        t.daemon.supervisor.status_of(&main),
        Some((BotState::Sleeping, _))
    ));
    assert!(!process.killed());
    assert_eq!(t.runtime.processes().len(), 1);
}
