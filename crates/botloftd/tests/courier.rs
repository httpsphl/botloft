//! Messages through the courier into the bots' stdin (spec 9): waiting for
//! the bot, order, read receipts, processes that end before reading,
//! attachments and archiving. Retry times follow the manual clock; the
//! courier itself polls on the real one.

mod common;

use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::protocol::BotState;
use botloftd::clock::Clock as _;
use common::bots::text_of;
use common::stream;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

const CONFLICT: i64 = -32003;
const NOT_FOUND: i64 = -32002;
const VALIDATION: i64 = -32004;

async fn crew_and_bot(app: &mut Client) -> Value {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    app.call("bots.create", bot).await.expect("bot")
}

async fn send(app: &mut Client, bot: &Value, body: &str) -> Value {
    app.call("messages.send", json!({ "botId": bot["id"], "body": body }))
        .await
        .expect("send")
}

/// The next `delivery.changed` that matches, skipping others.
async fn delivery_where(app: &mut Client, wanted: impl Fn(&Value) -> bool) -> Value {
    loop {
        let changed = app.notification("delivery.changed").await;
        if wanted(&changed) {
            return changed;
        }
    }
}

async fn delivery(app: &mut Client, state: &str) -> Value {
    delivery_where(app, |changed| changed["state"] == state).await
}

#[tokio::test]
async fn an_owner_message_reaches_the_bot_as_its_user_and_is_read() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    let process = t.process_of(&bot).await;

    let message = send(&mut app, &bot, "  Olá!\nTudo bem? ").await;
    assert_eq!(message["body"], "Olá!\nTudo bem?");
    assert_eq!(
        (&message["fromKind"], &message["kind"]),
        (&json!("owner"), &json!("note"))
    );
    assert_eq!(app.notification("message.created").await, message);
    let pending = delivery(&mut app, "pending").await;
    assert_eq!(pending["messageId"], message["id"]);
    let item = app.notification("chat.item").await;
    assert_eq!(
        item["item"]["body"],
        json!({ "kind": "inbound", "message": message })
    );
    assert_eq!(
        (&item["activity"]["kind"], &item["activity"]["text"]),
        (&json!("owner"), &json!("Olá! Tudo bem?"))
    );

    // It waited for the bot to be ready, and went as soon as it was.
    let line = process.wait_lines(1).await.remove(0);
    assert_eq!(line["type"], "user");
    assert_eq!(
        text_of(&line),
        "Olá!\nTudo bem?",
        "no envelope for the owner"
    );
    let sent = delivery(&mut app, "sent").await;
    assert_eq!(
        (&sent["attempts"], &sent["readAt"]),
        (&json!(0), &Value::Null)
    );

    process.emit(stream::replay(&line)).await;
    let read = delivery_where(&mut app, |changed| !changed["readAt"].is_null()).await;
    assert_eq!(read["id"], sent["id"]);

    let listed = app
        .call("messages.list", json!({ "botId": bot["id"] }))
        .await
        .expect("list");
    assert_eq!(listed, json!([message]));
    let status = app
        .call("system.status", json!(null))
        .await
        .expect("status");
    assert_eq!(status["deliveries"], json!({ "pending": 0, "dead": 0 }));
}

#[tokio::test]
async fn messages_to_one_bot_arrive_in_order() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    let process = t.process_of(&bot).await;
    for body in ["one", "two", "three"] {
        send(&mut app, &bot, body).await;
    }
    let lines = process.wait_lines(3).await;
    let texts: Vec<_> = lines.iter().map(text_of).collect();
    assert_eq!(texts, ["one", "two", "three"]);
    let uuids: std::collections::HashSet<_> = lines.iter().map(|l| l["uuid"].clone()).collect();
    assert_eq!(uuids.len(), 3, "each message has its own uuid");
}

#[tokio::test]
async fn what_a_dead_process_never_read_goes_back_until_it_gives_up() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    t.until_state(&bot, BotState::Idle).await;
    send(&mut app, &bot, "hello").await;

    // Three attempts (the test limit): each process ends before reading.
    for (attempt, wait) in [(1, 1), (2, 2)] {
        let process = t.process_of(&bot).await;
        process.wait_lines(1).await;
        delivery(&mut app, "sent").await;
        process.exit(1).await;
        let back = delivery_where(&mut app, |changed| {
            changed["state"] == "pending" && changed["attempts"] == attempt
        })
        .await;
        assert!(back["lastError"].as_str().expect("error").contains("ended"));
        let due_in = back["nextAttemptAt"].as_i64().expect("time") - t.clock.now_ms();
        assert_eq!(due_in, wait * 1000);
        t.until_state(&bot, BotState::Idle).await;
        t.clock.advance(Duration::from_secs(wait as u64));
    }
    let last = t.process_of(&bot).await;
    last.wait_lines(1).await;
    last.exit(1).await;
    let dead = delivery(&mut app, "dead").await;
    assert_eq!(dead["attempts"], 3);

    let retried = app
        .call("deliveries.retry", json!({ "deliveryId": dead["id"] }))
        .await
        .expect("retry");
    assert_eq!(
        (&retried["state"], &retried["attempts"]),
        (&json!("pending"), &json!(0))
    );
    // The bot was still restarting at the retry: it goes once it is ready.
    let fresh = t.process_of(&bot).await;
    assert_eq!(text_of(&fresh.wait_lines(1).await[0]), "hello");
    let again = app
        .call("deliveries.retry", json!({ "deliveryId": dead["id"] }))
        .await
        .expect_err("only dead ones");
    assert_eq!(again.code, CONFLICT);
}

#[tokio::test]
async fn a_message_the_bot_began_is_not_sent_again() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    t.until_state(&bot, BotState::Idle).await;
    let first = t.process_of(&bot).await;
    send(&mut app, &bot, "only once").await;
    let line = first.wait_lines(1).await.remove(0);
    first.emit(stream::replay(&line)).await;
    delivery_where(&mut app, |changed| !changed["readAt"].is_null()).await;
    first.exit(1).await;
    t.until_state(&bot, BotState::Idle).await;
    t.clock.advance(Duration::from_secs(10));
    tokio::time::sleep(Duration::from_millis(100)).await;
    let second = t.process_of(&bot).await;
    assert!(second.input_lines().is_empty());
}

#[tokio::test]
async fn archiving_a_bot_drops_what_is_still_queued_for_it() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    send(&mut app, &bot, "never read").await;
    app.call("bots.archive", json!({ "botId": bot["id"] }))
        .await
        .expect("archive");
    t.clock.advance(Duration::from_secs(5));
    let dead = delivery(&mut app, "dead").await;
    assert_eq!(dead["lastError"], "the bot was archived");
    assert_eq!(dead["attempts"], 0);
    let listed = app
        .call("deliveries.list", json!({ "state": "dead" }))
        .await
        .expect("list");
    assert_eq!(listed, json!([dead]));

    let err = send_err(&mut app, json!({ "botId": bot["id"], "body": "hi" })).await;
    assert_eq!(err, CONFLICT);
}

async fn send_err(app: &mut Client, params: Value) -> i64 {
    app.call("messages.send", params)
        .await
        .expect_err("refused")
        .code
}

#[tokio::test]
async fn invalid_messages_are_refused() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    assert_eq!(
        send_err(&mut app, json!({ "botId": bot["id"], "body": " \n " })).await,
        VALIDATION
    );
    let long = "x".repeat(100_001);
    assert_eq!(
        send_err(&mut app, json!({ "botId": bot["id"], "body": long })).await,
        VALIDATION
    );
    let file = json!({ "name": "a.txt", "mediaType": "text/plain", "data": BASE64.encode(b"a") });
    let many = json!({ "botId": bot["id"], "body": "x", "attachments": vec![file; 11] });
    assert_eq!(send_err(&mut app, many).await, VALIDATION);
    let big = json!({ "botId": bot["id"], "body": "x", "attachments": [
        { "name": "big.bin", "mediaType": "", "data": BASE64.encode(vec![0u8; 1024 * 1024 + 1]) }
    ] });
    assert_eq!(send_err(&mut app, big).await, VALIDATION);
    let missing = json!({ "botId": "bot_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0", "body": "hi" });
    assert_eq!(send_err(&mut app, missing).await, NOT_FOUND);
    let err = app
        .call("messages.list", json!({ "limit": 0 }))
        .await
        .expect_err("limit");
    assert_eq!(err.code, VALIDATION);
    let err = app
        .call(
            "deliveries.retry",
            json!({ "deliveryId": "dlv_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0" }),
        )
        .await
        .expect_err("missing");
    assert_eq!(err.code, NOT_FOUND);
}

#[tokio::test]
async fn a_message_that_waited_for_the_bot_goes_as_soon_as_it_is_ready() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    t.until_state(&bot, BotState::Idle).await;
    let paused = |paused: bool| json!({ "botId": bot["id"], "paused": paused });
    app.call("bots.setPaused", paused(true))
        .await
        .expect("pause");
    send(&mut app, &bot, "hello").await;
    // The courier sees it and puts it off while the bot is paused.
    tokio::time::sleep(Duration::from_millis(100)).await;

    app.call("bots.setPaused", paused(false))
        .await
        .expect("resume");
    // The manual clock never moves: the bot being ready is what it waited for.
    let process = t.runtime.process(2).await;
    assert_eq!(text_of(&process.wait_lines(1).await[0]), "hello");
}
