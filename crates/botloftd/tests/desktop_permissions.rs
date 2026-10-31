//! An app given to a bot from Settings before it asks, and one allowed only
//! for a turn (spec 24.2, 24.10), with a real window of the test's own.
//! Windows only.
#![cfg(windows)]

mod common;

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use common::browsing::{call, pending_approval};
use common::desktop::{TITLE, TestWindow, setup};
use common::{Client, stream};
use serde_json::{Value, json};

const IN_TIME: Duration = Duration::from_secs(20);

/// The settled request that comes next in the chat.
async fn settled_approval(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] != "pending" {
            return body.clone();
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_app_given_in_settings_is_seen_without_asking() {
    let window = TestWindow::open();
    let mut s = setup().await;
    let own = std::env::current_exe().expect("exe");
    let own = own.to_string_lossy();
    let apps = s
        .app
        .call("desktop.apps", Value::Null)
        .await
        .expect("apps");
    let ours = apps
        .as_array()
        .expect("list")
        .iter()
        .find(|app| {
            app["path"]
                .as_str()
                .is_some_and(|path| path.eq_ignore_ascii_case(&own))
        })
        .unwrap_or_else(|| panic!("the test's own app is not listed: {apps}"))
        .clone();

    let give = json!({ "botId": s.bot["id"], "path": ours["path"], "level": "see" });
    let given = s.app.call("desktop.grantApp", give).await.expect("grant");
    assert_eq!(given["grants"][0]["appName"], ours["name"]);
    assert_eq!(given["grants"][0]["level"], "see");
    let changed = s.app.notification("bot.desktop").await;
    assert_eq!(changed["botId"], s.bot["id"]);

    // The bot reads the window without anyone being asked.
    let look = json!({ "window": window.id, "why": "To read the form" });
    let text = tokio::time::timeout(IN_TIME, s.mcp.tool_text("desktop_look", look))
        .await
        .expect("read without asking")
        .expect("reading");
    assert!(text.contains(TITLE), "{text}");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_app_allowed_only_this_time_is_asked_for_again_in_the_next_turn() {
    let window = TestWindow::open();
    let mut s = setup().await;
    let bot: BotId = s.bot["id"].as_str().expect("id").parse().expect("bot id");
    let process = s.t.process_of(&s.bot).await;
    let look = json!({ "window": window.id, "why": "To read the form" });

    // A turn: the owner allows the app only this time.
    let ask = json!({ "botId": s.bot["id"], "body": "Read the form" });
    s.app.call("messages.send", ask).await.expect("send");
    let line = process.wait_lines(1).await.pop().expect("line");
    s.t.until_state(&s.bot, BotState::Busy).await;
    let reading_it = call(&s.mcp, "desktop_look", look.clone());
    let asked = pending_approval(&mut s.app).await;
    let once = json!({ "approvalId": asked["approvalId"], "allow": true, "once": true });
    s.app.call("approvals.answer", once).await.expect("answer");
    let text = reading_it.await.expect("task").expect("reading");
    assert!(text.contains(TITLE), "{text}");

    // The chat says so, and nothing was saved.
    let settled = settled_approval(&mut s.app).await;
    assert_eq!(settled["status"], "allowed");
    assert_eq!(settled["once"], true);
    let grants = s
        .app
        .call("desktop.grants", json!({ "botId": s.bot["id"] }))
        .await
        .expect("grants");
    assert_eq!(grants, json!([]));

    // While the turn lasts, the bot reads it again without asking.
    let again = tokio::time::timeout(IN_TIME, s.mcp.tool_text("desktop_look", look.clone()))
        .await
        .expect("read without asking")
        .expect("reading");
    assert!(again.contains(TITLE), "{again}");

    // The turn ends, and with it what was allowed for it.
    stream::answer(&process, &line, "Done.").await;
    s.t.until_state(&s.bot, BotState::Idle).await;
    for _ in 0..100 {
        if s.t.daemon.approvals.this_turn.apps(&bot).is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(s.t.daemon.approvals.this_turn.apps(&bot).is_empty());

    // The next turn asks again.
    let next = json!({ "botId": s.bot["id"], "body": "Read it again" });
    s.app.call("messages.send", next).await.expect("send");
    process.wait_lines(2).await;
    s.t.until_state(&s.bot, BotState::Busy).await;
    let reading_it = call(&s.mcp, "desktop_look", look);
    let asked = pending_approval(&mut s.app).await;
    let deny = json!({ "approvalId": asked["approvalId"], "allow": false });
    s.app.call("approvals.answer", deny).await.expect("answer");
    assert!(reading_it.await.expect("task").is_err());
}
