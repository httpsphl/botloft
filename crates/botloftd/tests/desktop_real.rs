//! The owner's real mouse and keyboard for a bot (spec 24.7), with a real
//! window of the test's own: off until the owner turns them on for the
//! app, never the system's own keys, never while the owner uses them. The
//! test that types for real runs only with BOTLOFT_REAL_INPUT_TESTS=1, as
//! CI's Windows runner sets. Windows only.
#![cfg(windows)]

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::browsing::{call, pending_approval};
use common::desktop::{Setup, TestWindow, reading, ref_in, setup};
use serde_json::{Value, json};

/// The bot may see and use the test's window; its last reading is kept.
async fn using(s: &mut Setup, window: &TestWindow) -> String {
    let look = json!({ "window": window.id, "why": "To file the visit" });
    let looking = call(&s.mcp, "desktop_look", look);
    let asked = pending_approval(&mut s.app).await;
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true });
    s.app.call("approvals.answer", allow).await.expect("see");
    let page = looking.await.expect("task").expect("reading");
    let field = ref_in(&page, r#"= "Ana Lima""#);
    let typing = call(
        &s.mcp,
        "desktop_type",
        json!({ "ref": field, "text": "Ana Lima", "why": "To file the visit" }),
    );
    let asked = pending_approval(&mut s.app).await;
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true });
    s.app.call("approvals.answer", allow).await.expect("act");
    typing.await.expect("task").expect("typed")
}

async fn grant_id(s: &mut Setup) -> Value {
    let grants = s
        .app
        .call("desktop.grants", json!({ "botId": s.bot["id"] }))
        .await
        .expect("grants");
    grants[0]["id"].clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_real_mouse_and_keyboard_wait_for_the_owner_to_turn_them_on_and_leave_them() {
    let window = TestWindow::open();
    let mut s = setup().await;
    using(&mut s, &window).await;
    let press = |keys: &str| json!({ "window": window.id, "keys": keys });

    // The system's own keys never, before anything is asked.
    let system = s.mcp.tool_text("desktop_press", press("Win+R")).await;
    assert!(
        reading(&system).contains("belongs to Windows itself"),
        "{}",
        reading(&system)
    );

    // Off until the owner turns them on for the app.
    let off = s.mcp.tool_text("desktop_press", press("Enter")).await;
    assert!(
        reading(&off).contains("have not turned on"),
        "{}",
        reading(&off)
    );

    let id = grant_id(&mut s).await;
    let changed = s
        .app
        .call(
            "desktop.setOptions",
            json!({ "grantId": id, "realInput": true }),
        )
        .await
        .expect("options");
    assert_eq!(changed["grants"][0]["realInput"], true);

    // A point of a picture needs the picture first.
    let blind = json!({ "window": window.id, "x": 10.0, "y": 10.0 });
    let blind = s.mcp.tool_text("desktop_click_at", blind).await;
    assert!(
        reading(&blind).contains("desktop_screenshot"),
        "{}",
        reading(&blind)
    );

    // The owner is using the computer this very moment: the bot waits.
    let busy = s.mcp.tool_text("desktop_press", press("Enter")).await;
    assert!(reading(&busy).contains("right now"), "{}", reading(&busy));
}

#[tokio::test(flavor = "multi_thread")]
async fn typing_then_enter_goes_through_the_real_keyboard() {
    if std::env::var("BOTLOFT_REAL_INPUT_TESTS").as_deref() != Ok("1") {
        eprintln!("real input tests are off: set BOTLOFT_REAL_INPUT_TESTS=1");
        return;
    }
    let window = TestWindow::open();
    let mut s = setup().await;
    let page = using(&mut s, &window).await;
    let id = grant_id(&mut s).await;
    s.app
        .call(
            "desktop.setOptions",
            json!({ "grantId": id, "realInput": true }),
        )
        .await
        .expect("options");
    // At the computer lately, but not touching it now.
    s.t.daemon
        .desktop
        .set_owner_idle(Arc::new(|| Some(Duration::from_secs(30))));
    let field = ref_in(&page, r#"= "Ana Lima""#);
    let typed = s
        .mcp
        .tool_text(
            "desktop_type",
            json!({ "ref": field, "text": "Ana Lima Souza", "submit": true }),
        )
        .await
        .expect("typed");
    assert!(typed.contains("Pressed Enter."), "{typed}");
    assert!(typed.contains(r#"= "Ana Lima Souza""#), "{typed}");
}
