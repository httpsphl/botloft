//! How much of the machine Botloft may take (spec 42.4): `system.limits`
//! says what the system caps and how many agents are awake, and the most
//! agents awake at once is a setting the app changes while Botloft runs.

mod common;

use std::time::Duration;

use botloft_core::protocol::BotState;
use common::conversations::nth_process;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

async fn two_asleep(t: &TestDaemon, app: &mut Client) -> (Value, Value) {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let mut bots = Vec::new();
    for name in ["Scout", "Writer"] {
        let bot = json!({ "crewId": crew["id"], "name": name, "role": "", "instructions": "" });
        let bot = app.call("bots.create", bot).await.expect("bot");
        t.until_state(&bot, BotState::Sleeping).await;
        bots.push(bot);
    }
    let writer = bots.pop().expect("writer");
    (bots.pop().expect("scout"), writer)
}

#[tokio::test]
async fn the_limits_are_read_and_the_agents_cap_is_changed_from_the_app() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let limits = app
        .call("system.limits", Value::Null)
        .await
        .expect("limits");
    assert_eq!(limits["agentsAwake"], 0);
    assert_eq!(limits["agentsMax"], 0, "no limit unless the owner sets one");
    assert!(limits["cpus"].as_u64().expect("cpus") >= 1);
    assert_eq!(
        app.call("settings.get", Value::Null).await.expect("get")["maxRunning"],
        0
    );

    let refused = app
        .call("settings.update", json!({ "maxRunning": 101 }))
        .await
        .expect_err("too many");
    assert!(
        refused.message.contains("maxRunning"),
        "{}",
        refused.message
    );

    let saved = app
        .call("settings.update", json!({ "maxRunning": 6 }))
        .await
        .expect("set");
    assert_eq!(saved["maxRunning"], 6);
    assert_eq!(t.daemon.settings.max_running(), Some(6));
    let limits = app
        .call("system.limits", Value::Null)
        .await
        .expect("limits");
    assert_eq!(limits["agentsMax"], 6);

    app.call("settings.update", json!({ "maxRunning": 0 }))
        .await
        .expect("no limit again");
    assert_eq!(t.daemon.settings.max_running(), None);
}

/// More room set in the app starts the agent that waited for it at once.
#[tokio::test(flavor = "multi_thread")]
async fn more_room_starts_the_agent_that_waited() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_max_running(1);
    t.daemon
        .settings
        .set_sleep_after(Some(Duration::from_secs(3600)));
    let mut app = t.session().await;
    two_asleep(&t, &mut app).await;
    // Sleep off: both want to run, and only one fits.
    t.daemon.settings.set_sleep_after(None);
    t.daemon.supervisor.wake();
    nth_process(&t, 0).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(t.runtime.processes().len(), 1, "one runs, the other waits");

    app.call("settings.update", json!({ "maxRunning": 2 }))
        .await
        .expect("more room");
    nth_process(&t, 1).await;
    let limits = app
        .call("system.limits", Value::Null)
        .await
        .expect("limits");
    assert_eq!(limits["agentsMax"], 2);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let awake = app
            .call("system.limits", Value::Null)
            .await
            .expect("limits")["agentsAwake"]
            .as_u64()
            .expect("awake");
        if awake == 2 {
            break;
        }
        assert!(tokio::time::Instant::now() < deadline, "{awake} awake");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// `botloftd limits` asks as the owner, with no params for `system.limits`:
/// the request leaves them out, as JSON-RPC wants.
#[tokio::test]
async fn the_owners_command_line_reads_the_limits() {
    let t = TestDaemon::start().await;
    let mut owner = botloftd::local::Owner::connect(t.addr.port(), common::TOKEN, "test")
        .await
        .expect("connect");
    let limits = owner
        .call("system.limits", Value::Null)
        .await
        .expect("limits");
    assert!(limits["cpus"].as_u64().expect("cpus") >= 1);
    owner.close().await;
}
