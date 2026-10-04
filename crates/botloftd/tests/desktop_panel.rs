//! A bot's desktop panel with real windows (spec 24.9): the window it is
//! using and what it did, its pictures while the panel is open, and the
//! owner stopping it, one bot or all of them. Windows only.
#![cfg(windows)]

mod common;

use common::browsing::{call, pending_approval};
use common::desktop::{TITLE, TestWindow, reading, setup};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn the_panel_shows_the_window_the_bot_uses_and_its_pictures() {
    let window = TestWindow::open();
    let mut s = setup().await;
    let id = json!({ "botId": s.bot["id"] });

    // Nothing used yet: no window, no picture.
    let view = s
        .app
        .call("desktop.watch", id.clone())
        .await
        .expect("watch");
    assert_eq!(view["state"]["window"], serde_json::Value::Null);
    assert_eq!(view["state"]["stopped"], false);
    assert_eq!(view["frame"], serde_json::Value::Null);

    let look = json!({ "window": window.id, "why": "To read the visit" });
    let looking = call(&s.mcp, "desktop_look", look);
    let asked = pending_approval(&mut s.app).await;
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true });
    s.app.call("approvals.answer", allow).await.expect("answer");
    looking.await.expect("task").expect("reading");

    // The panel hears which window, and its picture follows.
    let changed = s.app.notification("desktop.changed").await;
    assert_eq!(changed["window"]["title"], TITLE);
    assert_eq!(changed["action"], serde_json::Value::Null);
    let frame = s.app.notification("desktop.frame").await;
    assert_eq!(frame["botId"], s.bot["id"]);
    assert!(frame["data"].as_str().is_some_and(|data| data.len() > 100));

    s.app
        .call("desktop.unwatch", serde_json::Value::Null)
        .await
        .expect("unwatch");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_stopped_bot_cannot_use_the_desktop_until_the_owner_lets_it_go_on() {
    let _window = TestWindow::open();
    let mut s = setup().await;
    let id = json!({ "botId": s.bot["id"] });

    let stopped = s.app.call("desktop.stop", id.clone()).await.expect("stop");
    assert_eq!(stopped["stopped"], true);
    let refused = s.mcp.tool_text("desktop_windows", json!({})).await;
    assert!(
        reading(&refused).contains("stopped you"),
        "{}",
        reading(&refused)
    );

    let going = s
        .app
        .call("desktop.resume", id.clone())
        .await
        .expect("resume");
    assert_eq!(going["stopped"], false);
    s.mcp
        .tool_text("desktop_windows", json!({}))
        .await
        .expect("windows again");

    // The shortcut stops every bot at once.
    botloftd::service::desktop::stop_all(&s.t.daemon);
    let refused = s.mcp.tool_text("desktop_windows", json!({})).await;
    assert!(
        reading(&refused).contains("stopped you"),
        "{}",
        reading(&refused)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_notice_on_screen_speaks_the_language_the_owner_reads_the_app_in() {
    let mut s = setup().await;
    assert_eq!(s.t.daemon.desktop.locale(), "en");
    s.app
        .call("session.setLocale", json!({ "locale": "pt-BR" }))
        .await
        .expect("locale");
    assert_eq!(s.t.daemon.desktop.locale(), "pt-BR");
    let words = botloftd::desktop::notice::words(&s.t.daemon.desktop.locale(), "Scout");
    assert!(words.contains("Mexa o mouse para parar"), "{words}");
}
