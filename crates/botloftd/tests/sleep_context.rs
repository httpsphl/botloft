//! A sleeping bot's conversation (spec 7.7, 8.6): starting fresh forgets it
//! without waking the bot, and compacting wakes the bot and compacts as
//! soon as it is up.

mod common;

use std::time::Duration;

use botloft_core::protocol::BotState;
use common::{Client, TestDaemon, stream};
use serde_json::{Value, json};

const NAP: Duration = Duration::from_millis(300);

/// The `n`th process the runtime started, waiting up to 5 s for it.
async fn nth_process(t: &TestDaemon, n: usize) -> botloftd::runtime::fake::FakeProcess {
    for _ in 0..500 {
        if let Some(process) = t.runtime.processes().into_iter().nth(n) {
            return process;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("process {n} never started");
}

/// A bot that had one turn in "session-1" and fell asleep after it.
async fn asleep_after_a_turn(t: &TestDaemon, app: &mut Client) -> Value {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    t.until_state(&bot, BotState::Sleeping).await;
    app.call(
        "messages.send",
        json!({ "botId": bot["id"], "body": "Hello" }),
    )
    .await
    .expect("send");
    let first = nth_process(t, 0).await;
    let line = first.wait_lines(1).await.pop().expect("line");
    let mut opened = stream::replay(&line);
    opened["session_id"] = json!("session-1");
    first.emit(stream::init("session-1")).await;
    first.emit(opened).await;
    first.emit(stream::text("Hi")).await;
    first.emit(stream::result(false)).await;
    t.until_state(&bot, BotState::Sleeping).await;
    assert!(first.killed());
    bot
}

#[tokio::test(flavor = "multi_thread")]
async fn starting_fresh_forgets_the_conversation_and_lets_the_bot_sleep() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(NAP));
    let mut app = t.session().await;
    let bot = asleep_after_a_turn(&t, &mut app).await;

    let fresh = app
        .call("bots.restart", json!({ "botId": bot["id"], "fresh": true }))
        .await
        .expect("fresh");
    assert_eq!(fresh["context"], Value::Null);
    tokio::time::sleep(NAP * 3).await;
    assert_eq!(t.runtime.processes().len(), 1, "it was not woken");
    t.until_state(&bot, BotState::Sleeping).await;
    let history = app
        .call("chat.history", json!({ "botId": bot["id"] }))
        .await
        .expect("history");
    assert_eq!(history[0]["body"]["code"], "fresh_start");

    // The next message wakes it in a new conversation.
    app.call(
        "messages.send",
        json!({ "botId": bot["id"], "body": "Again" }),
    )
    .await
    .expect("send");
    let second = nth_process(&t, 1).await;
    let args = second.args();
    assert!(!args.iter().any(|arg| arg == "--resume"), "{args:?}");
    let line = second.wait_lines(1).await.pop().expect("line");
    assert!(line.to_string().contains("Again"), "{line}");
}

#[tokio::test(flavor = "multi_thread")]
async fn compacting_wakes_a_sleeping_bot_and_compacts_once_it_is_up() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(NAP));
    let mut app = t.session().await;
    let bot = asleep_after_a_turn(&t, &mut app).await;

    app.call("bots.compact", json!({ "botId": bot["id"] }))
        .await
        .expect("compact");
    let second = nth_process(&t, 1).await;
    let args = second.args();
    let at = args
        .iter()
        .position(|arg| arg == "--resume")
        .unwrap_or_else(|| panic!("{args:?}"));
    assert_eq!(args.get(at + 1).map(String::as_str), Some("session-1"));
    let line = second.wait_lines(1).await.pop().expect("line");
    assert_eq!(line["message"]["content"][0]["text"], "/compact", "{line}");

    // Asked again while it is on its way, nothing more goes.
    app.call("bots.compact", json!({ "botId": bot["id"] }))
        .await
        .expect("again");
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(second.input_lines().len(), 1);
}
