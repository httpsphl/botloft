//! Owner messages through the courier into a fake inbox (spec 9): waiting
//! for the bot, order, retries, restarts and archiving. Retry times follow
//! the manual clock; the courier itself polls on the real one.

mod common;

use std::time::Duration;

use botloftd::clock::Clock as _;
use botloftd::hooks::client::post;
use botloftd::runtime::fake::FakeProcess;
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

fn pipe(n: u32) -> String {
    format!(r"\\.\pipe\LOCAL\cc-msg-test-{n}")
}

/// Sends `SessionStart` through `/hooks`, as the bot's hook would, with the
/// inbox `pipe(n)` and messaging token `tok-n`.
async fn session_start(t: &TestDaemon, process: &FakeProcess, n: u32) {
    let token = process.env("BOTLOFT_BOT_TOKEN").expect("bot token");
    let body = json!({
        "payload": { "hook_event_name": "SessionStart" },
        "messagingSocket": pipe(n),
        "messagingToken": format!("tok-{n}"),
    });
    let body = serde_json::to_vec(&body).expect("json");
    let port = t.addr.port();
    tokio::task::spawn_blocking(move || post(port, "/hooks/session-start", &token, &body))
        .await
        .expect("join")
        .expect("hook accepted");
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
async fn an_owner_message_waits_for_the_bot_and_then_reaches_its_inbox() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    let process = t.runtime.process(1).await;

    let message = send(&mut app, &bot, "  Olá!\nTudo bem? ").await;
    assert_eq!(message["body"], "Olá!\nTudo bem?");
    assert_eq!(message["fromKind"], "owner");
    assert_eq!(message["kind"], "note");
    assert_eq!(app.notification("message.created").await, message);
    let pending = delivery(&mut app, "pending").await;
    assert_eq!(pending["messageId"], message["id"]);

    // Still launching: the courier waits and counts nothing.
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(t.inbox.attempts(), 0);

    session_start(&t, &process, 1).await;
    t.clock.advance(Duration::from_secs(5));
    let post = t.inbox.post(1).await;
    assert_eq!(post.address, pipe(1));
    assert_eq!(post.token, "tok-1");
    assert_eq!(
        post.text,
        "[botloft] from the owner · crew Ops\n\nOlá!\nTudo bem?"
    );
    delivery(&mut app, "sending").await;
    let sent = delivery(&mut app, "sent").await;
    assert_eq!(sent["attempts"], 0);

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
    let process = t.runtime.process(1).await;
    for body in ["one", "two", "three"] {
        send(&mut app, &bot, body).await;
    }
    session_start(&t, &process, 1).await;
    t.clock.advance(Duration::from_secs(5));
    for (n, body) in ["one", "two", "three"].iter().enumerate() {
        assert!(t.inbox.post(n + 1).await.text.ends_with(body));
    }
}

#[tokio::test]
async fn failed_writes_back_off_then_give_up_and_can_be_retried() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    let process = t.runtime.process(1).await;
    session_start(&t, &process, 1).await;
    t.inbox.fail_next(3);
    send(&mut app, &bot, "hello").await;

    // Three attempts (the test limit), 1 s then 2 s apart.
    for (attempts, wait) in [(1, 1), (2, 2)] {
        let retry = delivery_where(&mut app, |changed| {
            changed["state"] == "pending" && changed["attempts"] == attempts
        })
        .await;
        assert_eq!(retry["lastError"], "fake failure");
        let due_in = retry["nextAttemptAt"].as_i64().expect("time") - t.clock.now_ms();
        assert_eq!(due_in, wait * 1000);
        t.clock.advance(Duration::from_secs(wait as u64));
    }
    let dead = delivery(&mut app, "dead").await;
    assert_eq!(dead["attempts"], 3);
    let status = app
        .call("system.status", json!(null))
        .await
        .expect("status");
    assert_eq!(status["deliveries"], json!({ "pending": 0, "dead": 1 }));

    let retried = app
        .call("deliveries.retry", json!({ "deliveryId": dead["id"] }))
        .await
        .expect("retry");
    assert_eq!(
        (&retried["state"], &retried["attempts"]),
        (&json!("pending"), &json!(0))
    );
    assert!(t.inbox.post(1).await.text.ends_with("hello"));
    delivery(&mut app, "sent").await;
    let again = app
        .call("deliveries.retry", json!({ "deliveryId": dead["id"] }))
        .await
        .expect_err("only dead ones");
    assert_eq!(again.code, CONFLICT);
}

#[tokio::test]
async fn a_bot_restart_during_a_send_does_not_count_as_a_failure() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let bot = crew_and_bot(&mut app).await;
    let first = t.runtime.process(1).await;
    session_start(&t, &first, 1).await;

    t.inbox.hold();
    t.inbox.fail_next(1);
    send(&mut app, &bot, "survive the restart").await;
    delivery(&mut app, "sending").await;
    first.exit(1).await;
    let second = t.runtime.process(2).await;
    t.inbox.release();

    let back = delivery(&mut app, "pending").await;
    assert_eq!(
        back["attempts"], 0,
        "the old inbox going away is not a failure"
    );
    session_start(&t, &second, 2).await;
    t.clock.advance(Duration::from_secs(5));
    let post = t.inbox.post(1).await;
    assert_eq!((post.address, post.token), (pipe(2), "tok-2".to_owned()));
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
