//! A bot on the owner's desktop with real windows (spec 24): a window of
//! the test's own, with a Save button that renames it, a field and a
//! password field, seen once the owner lets the bot see its app and used
//! once they let it use it. Windows only.
#![cfg(windows)]

mod common;

use std::time::Duration;

use common::browsing::{call, pending_approval};
use common::desktop::{TITLE, TestWindow, reading, ref_in, setup};
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn a_bot_sees_an_app_once_the_owner_lets_it_and_never_a_password() {
    let window = TestWindow::open();
    let mut s = setup().await;
    let look = json!({ "window": window.id, "why": "To read the patient's name" });

    // Before any grant, the window is there without its title.
    let listed = s
        .mcp
        .tool_text("desktop_windows", json!({}))
        .await
        .expect("windows");
    assert!(
        listed.contains(&format!("window {}:", window.id)),
        "{listed}"
    );
    assert!(!listed.contains(TITLE), "{listed}");

    // Reading it asks the owner, with the app and the bot's why.
    let reading_it = call(&s.mcp, "desktop_look", look.clone());
    let asked = pending_approval(&mut s.app).await;
    assert_eq!(asked["toolName"], "mcp__botloft__desktop");
    assert!(
        asked["input"]
            .as_str()
            .is_some_and(|input| input.contains("To read the patient's name")),
        "{asked}"
    );
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true });
    s.app.call("approvals.answer", allow).await.expect("answer");
    let text = reading_it.await.expect("task").expect("reading");
    assert!(text.contains(TITLE), "{text}");
    assert!(text.contains(r#"button "Save""#), "{text}");
    assert!(text.contains(r#"= "Ana Lima""#), "{text}");
    assert!(!text.contains("hunter2"), "{text}");

    // The grant is kept and the apps hear of it.
    let changed = s.app.notification("bot.desktop").await;
    assert_eq!(changed["grants"][0]["level"], "see");
    let grants = s
        .app
        .call("desktop.grants", json!({ "botId": s.bot["id"] }))
        .await
        .expect("grants");
    assert_eq!(grants.as_array().map(Vec::len), Some(1));

    // Now the title shows, and so does a picture, without asking again.
    let listed = s
        .mcp
        .tool_text("desktop_windows", json!({}))
        .await
        .expect("windows");
    assert!(listed.contains(TITLE), "{listed}");
    let picture = s.mcp.tool_result("desktop_screenshot", look.clone()).await;
    assert_eq!(picture["content"][0]["type"], "image");
    assert_eq!(picture["content"][0]["mimeType"], "image/jpeg");

    // Taken away, it asks again; refused, the bot reads why.
    let revoke = json!({ "grantId": grants[0]["id"] });
    s.app.call("desktop.revoke", revoke).await.expect("revoke");
    let reading_it = call(&s.mcp, "desktop_look", look);
    let asked = pending_approval(&mut s.app).await;
    let deny = json!({ "approvalId": asked["approvalId"], "allow": false, "note": "Not now" });
    s.app.call("approvals.answer", deny).await.expect("answer");
    let refused = reading_it.await.expect("task");
    assert!(refused.is_err());
    assert!(
        reading(&refused).contains("Not now"),
        "{}",
        reading(&refused)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn nothing_is_seen_while_the_owner_is_away() {
    let s = setup().await;
    s.t.daemon
        .desktop
        .set_owner_idle(std::sync::Arc::new(|| Some(Duration::from_secs(3600))));
    let mut mcp = s.mcp;
    let away = mcp.tool_text("desktop_windows", json!({})).await;
    assert!(away.is_err());
    assert!(
        reading(&away).contains("has not used the computer"),
        "{}",
        reading(&away)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bot_uses_an_app_once_the_owner_lets_it_and_never_types_a_password() {
    let window = TestWindow::open();
    let mut s = setup().await;
    let look = json!({ "window": window.id, "why": "To file the visit" });

    // Acting before any reading has nothing to name.
    let early = s
        .mcp
        .tool_text("desktop_click", json!({ "ref": "d1" }))
        .await;
    assert!(
        reading(&early).contains("Read the window first"),
        "{}",
        reading(&early)
    );

    // Seeing it is asked once.
    let looking = call(&s.mcp, "desktop_look", look);
    let asked = pending_approval(&mut s.app).await;
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true });
    s.app.call("approvals.answer", allow).await.expect("see");
    let page = looking.await.expect("task").expect("reading");
    let save = ref_in(&page, r#"button "Save""#);
    let password = ref_in(&page, "the owner types it");

    // A password field is refused before anything is asked.
    let typed = s
        .mcp
        .tool_text("desktop_type", json!({ "ref": password, "text": "x" }))
        .await;
    assert!(
        reading(&typed).contains("password field"),
        "{}",
        reading(&typed)
    );

    // Using the app asks again, now to use it; without a why, it says so.
    let no_why = s
        .mcp
        .tool_text("desktop_click", json!({ "ref": save }))
        .await;
    assert!(reading(&no_why).contains("why"), "{}", reading(&no_why));
    let clicking = call(
        &s.mcp,
        "desktop_click",
        json!({ "ref": save, "why": "To save the visit" }),
    );
    let asked = pending_approval(&mut s.app).await;
    assert!(
        asked["input"]
            .as_str()
            .is_some_and(|input| input.contains(r#""level":"act""#)),
        "{asked}"
    );
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true });
    s.app.call("approvals.answer", allow).await.expect("act");
    let clicked = clicking.await.expect("task").expect("clicked");
    assert!(
        clicked.starts_with(r#"Clicked button "Save"."#),
        "{clicked}"
    );
    assert!(clicked.contains(r#""Saved""#), "{clicked}");

    // Granted to act, the next action asks nothing.
    let grants = s
        .app
        .call("desktop.grants", json!({ "botId": s.bot["id"] }))
        .await
        .expect("grants");
    assert_eq!(grants[0]["level"], "act");
    let field = ref_in(&clicked, r#"= "Ana Lima""#);
    let typed = s
        .mcp
        .tool_text(
            "desktop_type",
            json!({ "ref": field, "text": "Ana Lima Souza" }),
        )
        .await
        .expect("typed");
    assert!(typed.contains(r#"= "Ana Lima Souza""#), "{typed}");
}
