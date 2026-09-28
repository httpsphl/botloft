//! Terminal, hooks and restarts over the network: the app's view of M2.

mod common;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloftd::hooks::client::post;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

fn decode(frame: &Value) -> Vec<u8> {
    BASE64
        .decode(frame["data"].as_str().expect("data"))
        .expect("base64")
}

async fn crew_and_bot(app: &mut Client) -> Value {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    app.call("bots.create", bot).await.expect("bot")
}

/// Waits for `bot.state` to report `state`, skipping earlier states.
async fn state(app: &mut Client, state: &str) -> Value {
    loop {
        let event = app.notification("bot.state").await;
        if event["state"] == state {
            return event;
        }
    }
}

#[tokio::test]
async fn hooks_from_the_bot_change_the_state_the_app_sees() {
    let daemon = TestDaemon::start_supervised().await;
    let mut app = daemon.session().await;
    let bot = crew_and_bot(&mut app).await;
    let process = daemon.runtime.process(1).await;
    let launching = state(&mut app, "launching").await;
    assert_eq!(launching["botId"], bot["id"]);

    let token = process.env("BOTLOFT_BOT_TOKEN").expect("token");
    let port = daemon.addr.port();
    let body = json!({ "payload": { "hook_event_name": "SessionStart", "source": "startup" } });
    let body = serde_json::to_vec(&body).expect("json");
    tokio::task::spawn_blocking(move || post(port, "/hooks/session-start", &token, &body))
        .await
        .expect("join")
        .expect("hook accepted");
    let idle = state(&mut app, "idle").await;
    assert_eq!(idle["generation"], launching["generation"]);

    let listed = app.call("bots.list", json!(null)).await.expect("list");
    assert_eq!(listed[0]["state"], "idle");
    assert_eq!(listed[0]["generation"], launching["generation"]);

    let wrong =
        tokio::task::spawn_blocking(move || post(port, "/hooks/stop", "not-a-token", b"{}"))
            .await
            .expect("join");
    assert!(wrong.expect_err("refused").contains("401"));
}

#[tokio::test]
async fn attach_replays_the_screen_then_streams_and_reattach_skips_what_was_seen() {
    let daemon = TestDaemon::start_supervised().await;
    let mut app = daemon.session().await;
    let bot = crew_and_bot(&mut app).await;
    let process = daemon.runtime.process(1).await;
    state(&mut app, "launching").await;
    process.output(b"hello\r\n").await;
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let attached = app
        .call("terminal.attach", json!({ "botId": bot["id"] }))
        .await
        .expect("attach");
    assert_eq!(attached["reset"], true);
    assert_eq!(attached["offset"], 0);
    // The 7 buffered bytes are history; what comes after is live.
    assert_eq!(attached["liveOffset"], 7);
    let replay = app.notification("terminal.data").await;
    assert_eq!(decode(&replay), b"hello\r\n");

    process.output(b"more").await;
    let live = app.notification("terminal.data").await;
    assert_eq!(live["offset"], 7);
    assert_eq!(decode(&live), b"more");

    let written = json!({ "botId": bot["id"], "data": BASE64.encode("hi\r") });
    app.call("terminal.write", written).await.expect("write");
    assert_eq!(process.input(), b"hi\r");
    let resized = json!({ "botId": bot["id"], "cols": 100, "rows": 30 });
    app.call("terminal.resize", resized).await.expect("resize");
    assert_eq!(
        process.sizes().last().map(|s| (s.cols, s.rows)),
        Some((100, 30))
    );

    // Another window picks up where it left off: nothing to replay.
    let mut other = daemon.session().await;
    let resume = json!({ "botId": bot["id"], "generation": attached["generation"], "offset": 11 });
    let again = other
        .call("terminal.attach", resume)
        .await
        .expect("reattach");
    assert_eq!(again["reset"], false);
    assert_eq!(again["offset"], 11);
    assert_eq!(again["liveOffset"], 11);

    app.call("terminal.detach", json!({ "botId": bot["id"] }))
        .await
        .expect("detach");
    process.output(b"!").await;
    let seen_by_other = other.notification("terminal.data").await;
    assert_eq!(decode(&seen_by_other), b"!");
}

#[tokio::test]
async fn a_restart_shows_up_as_a_new_generation() {
    let daemon = TestDaemon::start_supervised().await;
    let mut app = daemon.session().await;
    let bot = crew_and_bot(&mut app).await;
    let first = daemon.runtime.process(1).await;
    let before = state(&mut app, "launching").await;
    app.call("terminal.attach", json!({ "botId": bot["id"] }))
        .await
        .expect("attach");

    let restarted = app
        .call("bots.restart", json!({ "botId": bot["id"], "fresh": true }))
        .await
        .expect("restart");
    assert_eq!(restarted["id"], bot["id"]);
    let second = daemon.runtime.process(2).await;
    assert!(first.killed());
    let after = state(&mut app, "launching").await;
    assert_ne!(after["generation"], before["generation"]);

    second.output(b"new screen").await;
    let data = app.notification("terminal.data").await;
    assert_eq!(data["generation"], after["generation"]);
    assert_eq!(data["offset"], 0);
}

#[tokio::test]
async fn terminal_calls_check_the_bot_and_its_process() {
    let daemon = TestDaemon::start().await;
    let mut app = daemon.session().await;
    let bot = crew_and_bot(&mut app).await;

    let write = json!({ "botId": bot["id"], "data": BASE64.encode("x") });
    let err = app
        .call("terminal.write", write)
        .await
        .expect_err("not running");
    assert_eq!(err.code, -32003);
    let bad = json!({ "botId": bot["id"], "data": "***" });
    assert_eq!(
        app.call("terminal.write", bad)
            .await
            .expect_err("base64")
            .code,
        -32004
    );
    let huge = json!({ "botId": bot["id"], "cols": 0, "rows": 30 });
    assert_eq!(
        app.call("terminal.resize", huge)
            .await
            .expect_err("size")
            .code,
        -32004
    );
    let missing = json!({ "botId": "bot_01J9Z3K8M4Q7R2T5V8X1Y4Z6A0" });
    assert_eq!(
        app.call("terminal.attach", missing)
            .await
            .expect_err("missing")
            .code,
        -32002
    );

    let paused = json!({ "botId": bot["id"], "paused": true });
    app.call("bots.setPaused", paused).await.expect("pause");
    let err = app
        .call("bots.restart", json!({ "botId": bot["id"] }))
        .await
        .expect_err("paused");
    assert_eq!(err.code, -32003);
}
