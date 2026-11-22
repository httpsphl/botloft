//! A daemon on a server (spec 42.2): nobody sits at its screen, so what
//! needs one is refused, the bots' tools leave out the owner's own
//! computer, and the waits and routines the bots make keep the owner's time
//! zone instead of the server's.

mod common;

use common::mcp::Mcp;
use common::{Client, TOKEN, TestDaemon};
use serde_json::{Value, json};

const ON_A_SERVER: i64 = -32006;

/// An app that says the owner's zone, a crew, a running bot and its MCP
/// client.
async fn crew_with_bot(t: &TestDaemon, zone: Option<&str>) -> (Client, Value, Mcp) {
    let mut app = t.client().await;
    let mut client = json!({ "name": "tests", "version": "0" });
    if let Some(zone) = zone {
        client["timeZone"] = json!(zone);
    }
    app.call(
        "session.hello",
        json!({ "token": TOKEN, "client": client, "protocol": 2 }),
    )
    .await
    .expect("hello");
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "Finds things", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    let process = t.process_of(&bot).await;
    let token = process.env("BOTLOFT_BOT_TOKEN").expect("token");
    (app, bot, Mcp::new(t.addr, token))
}

#[tokio::test]
async fn a_server_says_so_and_refuses_what_needs_a_screen() {
    let t = TestDaemon::start_on_a_server().await;
    let (mut app, bot, _mcp) = crew_with_bot(&t, None).await;
    let status = app.call("system.status", json!({})).await.expect("status");
    assert_eq!(status["server"], true);

    for (method, params) in [
        (
            "desktop.grantApp",
            json!({ "botId": bot["id"], "path": "C:\\Apps\\notes.exe" }),
        ),
        ("desktop.watch", json!({ "botId": bot["id"] })),
        ("logins.open", json!({ "address": "reddit.com" })),
        ("chrome.set", json!({ "botId": bot["id"], "on": true })),
        ("settings.update", json!({ "keepAwake": true })),
        ("settings.update", json!({ "startWithWindows": false })),
    ] {
        let refused = app.call(method, params).await.expect_err(method);
        assert_eq!(refused.code, ON_A_SERVER, "{method}: {}", refused.message);
        assert!(
            refused.message.contains("runs on a server"),
            "{method}: {}",
            refused.message
        );
    }
    // Reads and the rest of the settings answer as usual.
    let grants = app
        .call("desktop.grants", json!({ "botId": bot["id"] }))
        .await
        .expect("grants");
    assert_eq!(grants, json!([]));
    let settings = app
        .call("settings.update", json!({ "sleepAfterMinutes": 15 }))
        .await
        .expect("settings");
    assert_eq!(settings["sleepAfterMinutes"], 15);
}

#[tokio::test]
async fn the_bots_tools_leave_out_the_owners_own_computer() {
    let t = TestDaemon::start_on_a_server().await;
    let (_app, _bot, mut mcp) = crew_with_bot(&t, None).await;
    let listed = mcp.request("tools/list", json!({})).await;
    let names: Vec<String> = listed.body["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|tool| tool["name"].as_str().expect("name").to_owned())
        .collect();
    assert!(
        names
            .iter()
            .all(|name| !name.starts_with("desktop_") && name != "ask_chrome"),
        "{names:?}"
    );
    assert!(names.iter().any(|name| name == "browser_open"), "{names:?}");
    assert_eq!(
        names.last().map(String::as_str),
        Some("permission_prompt"),
        "permission_prompt stays last"
    );
    let discovered = mcp.request("server/discover", json!({})).await;
    let said = discovered.body["result"]["instructions"]
        .as_str()
        .expect("instructions");
    assert!(!said.contains("desktop_"), "{said}");
    assert!(said.ends_with("start with [botloft]."), "{said}");

    // A call that comes anyway says why it cannot work.
    let refused = mcp
        .tool("desktop_windows", json!({}))
        .await
        .expect_err("refused");
    assert!(refused.contains("runs on a server"), "{refused}");
}

#[tokio::test]
async fn a_wait_a_bot_sets_keeps_the_owners_time_zone() {
    let t = TestDaemon::start_on_a_server().await;
    let (mut app, bot, mut mcp) = crew_with_bot(&t, Some("Asia/Tokyo")).await;
    mcp.tool(
        "wait_for",
        json!({ "name": "Wait for the report", "what": "file", "path": "out/report.pdf",
                "hours": 2, "then": "Read it and tell the owner." }),
    )
    .await
    .expect("wait");
    let routines = app
        .call("routines.list", json!({ "botId": bot["id"] }))
        .await
        .expect("routines");
    assert_eq!(routines[0]["timezone"], "Asia/Tokyo", "{routines}");
}

#[tokio::test]
async fn a_zone_the_daemon_does_not_know_is_left_out() {
    let t = TestDaemon::start_on_a_server().await;
    let (mut app, bot, mut mcp) = crew_with_bot(&t, Some("Mars/Olympus")).await;
    mcp.tool(
        "wait_for",
        json!({ "name": "Render", "what": "file", "path": "out/frames.zip",
                "hours": 1, "then": "Look at the frames." }),
    )
    .await
    .expect("wait");
    let routines = app
        .call("routines.list", json!({ "botId": bot["id"] }))
        .await
        .expect("routines");
    assert_ne!(routines[0]["timezone"], "Mars/Olympus", "{routines}");
}
