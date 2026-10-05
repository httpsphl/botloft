//! A bot with the owner's whole desktop (spec 24.2, 24.4, D6): only the
//! owner gives it, after accepting the risks; then every window shows with
//! its title, a window is read without asking, and the whole screen comes
//! as one picture. Windows only.
#![cfg(windows)]

mod common;

use common::desktop::{TITLE, TestWindow, reading, setup};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_gives_the_whole_desktop_and_the_bot_sees_the_screen() {
    let window = TestWindow::open();
    let mut s = setup().await;

    // Without it, no whole screen, and a window's title stays hidden.
    let refused = s
        .mcp
        .tool_text("desktop_screenshot", json!({ "why": "To see the screen" }))
        .await;
    assert!(
        reading(&refused).contains("not given you their whole desktop"),
        "{}",
        reading(&refused)
    );
    let listed = s
        .mcp
        .tool_text("desktop_windows", json!({}))
        .await
        .expect("windows");
    assert!(!listed.contains(TITLE), "{listed}");

    // Only with the risks accepted.
    let unaccepted = json!({ "botId": s.bot["id"], "level": "see", "acceptedRisks": false });
    assert!(s.app.call("desktop.grantWhole", unaccepted).await.is_err());
    let accepted = json!({ "botId": s.bot["id"], "level": "see", "acceptedRisks": true });
    let given = s
        .app
        .call("desktop.grantWhole", accepted)
        .await
        .expect("given");
    assert_eq!(given["grants"][0]["scope"], "desktop");
    assert!(given["grants"][0]["acceptedRisksAt"].is_i64());
    let changed = s.app.notification("bot.desktop").await;
    assert_eq!(changed["grants"][0]["scope"], "desktop");

    // Every window with its title; one is read without asking.
    let listed = s
        .mcp
        .tool_text("desktop_windows", json!({}))
        .await
        .expect("windows");
    assert!(listed.contains(TITLE), "{listed}");
    let look = json!({ "window": window.id, "why": "To read the visit" });
    let text = s.mcp.tool_text("desktop_look", look).await.expect("read");
    assert!(text.contains(r#"button "Save""#), "{text}");

    // And the whole screen as one picture.
    let picture = s
        .mcp
        .tool_result("desktop_screenshot", json!({ "why": "To see the screen" }))
        .await;
    assert_eq!(picture["content"][0]["type"], "image", "{picture}");
    assert!(
        picture["content"][1]["text"]
            .as_str()
            .is_some_and(|text| text.contains("whole screen")),
        "{picture}"
    );
}
