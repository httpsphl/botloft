//! A bot on the owner's desktop while the owner is away (spec 24.8): only
//! in the reach of a grant they turned that on for, after accepting the
//! risks (24.10), and the app hears of each app the bot used then.
//! Windows only.
#![cfg(windows)]

mod common;

use std::sync::Arc;
use std::time::Duration;

use common::browsing::{call, pending_approval};
use common::desktop::{TITLE, TestWindow, reading, ref_in, setup};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn a_bot_works_while_the_owner_is_away_only_where_they_allowed_it() {
    let window = TestWindow::open();
    let mut s = setup().await;
    let look = json!({ "window": window.id, "why": "To file the night's visits" });

    // The owner lets the bot see the app while at the computer.
    let reading_it = call(&s.mcp, "desktop_look", look.clone());
    let asked = pending_approval(&mut s.app).await;
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true });
    s.app.call("approvals.answer", allow).await.expect("answer");
    reading_it.await.expect("task").expect("reading");
    let grants = s
        .app
        .call("desktop.grants", json!({ "botId": s.bot["id"] }))
        .await
        .expect("grants");
    let grant = grants[0]["id"].clone();

    // Then they leave: without the option, nothing works and nobody is asked.
    s.t.daemon
        .desktop
        .set_owner_idle(Arc::new(|| Some(Duration::from_secs(3600))));
    let away = s.mcp.tool_text("desktop_look", look.clone()).await;
    assert!(
        reading(&away).contains("has not used the computer"),
        "{}",
        reading(&away)
    );
    let listed = s.mcp.tool_text("desktop_windows", json!({})).await;
    assert!(listed.is_err(), "{}", reading(&listed));

    // The option goes on only with the risks accepted.
    let unaccepted = json!({ "grantId": grant, "unattended": true });
    assert!(s.app.call("desktop.setOptions", unaccepted).await.is_err());
    let accepted = json!({ "grantId": grant, "unattended": true, "acceptedRisks": true });
    let options = s
        .app
        .call("desktop.setOptions", accepted)
        .await
        .expect("options");
    assert_eq!(options["grants"][0]["unattended"], true);
    assert!(options["grants"][0]["acceptedRisksAt"].is_i64());

    // Now the bot sees that app with the owner away.
    let listed = s
        .mcp
        .tool_text("desktop_windows", json!({}))
        .await
        .expect("windows");
    assert!(listed.contains(TITLE), "{listed}");
    let text = s
        .mcp
        .tool_text("desktop_look", look)
        .await
        .expect("reading");
    assert!(text.contains(r#"button "Save""#), "{text}");

    // The app hears of it, to tell the owner when they are back.
    let used = s.app.notification("desktop.away").await;
    assert_eq!(used[0]["botId"], s.bot["id"]);
    assert!(used[0]["app"].is_string(), "{used}");

    // Using it needs a grant to use, and the owner is not there to give it.
    let save = ref_in(&text, r#"button "Save""#);
    let click = json!({ "ref": save, "why": "To save the visit" });
    let refused = s.mcp.tool_text("desktop_click", click).await;
    assert!(
        reading(&refused).contains("has not used the computer"),
        "{}",
        reading(&refused)
    );

    // Back, the owner dismisses the notice.
    let uses = s
        .app
        .call("desktop.awayUses", json!(null))
        .await
        .expect("uses");
    assert_eq!(uses.as_array().map(Vec::len), Some(1));
    s.app
        .call("desktop.dismissAway", json!(null))
        .await
        .expect("dismiss");
    let uses = s
        .app
        .call("desktop.awayUses", json!(null))
        .await
        .expect("uses");
    assert_eq!(uses.as_array().map(Vec::len), Some(0));
}
