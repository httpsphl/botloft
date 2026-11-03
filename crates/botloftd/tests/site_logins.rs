//! Shared site logins with the real Microsoft Edge (spec 35): a login in
//! the logins profile is copied to an agent's browser, whose site then
//! gets the cookie, and taken back. Skipped where Edge is not installed.

mod common;

use botloft_core::protocol::BotState;
use botloftd::browser::CookieChange;
use common::TestDaemon;
use common::browsing::setup;
use common::mcp::Mcp;
use serde_json::{Value, json};

/// The agent starts again with its new logins (spec 35.3), as its
/// `number`th process, with a new token.
async fn restarted(b: &mut common::browsing::Browsing, number: usize) {
    let process = b.t.runtime.process(number).await;
    b.t.until_state(&b.bot, BotState::Idle).await;
    b.mcp = Mcp::new(b.t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
}

/// What the agent's browser sends to the site's `/whoami`.
async fn cookies_seen(b: &mut common::browsing::Browsing) -> String {
    let url = format!("{}/whoami", b.site);
    b.mcp
        .tool_text("browser_open", json!({ "url": url }))
        .await
        .expect("page")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_login_made_once_is_given_to_an_agent_and_taken_back() {
    let Some(mut b) = setup().await else { return };
    let daemon = std::sync::Arc::clone(&b.t.daemon);
    // The owner signed in: the site left its cookie in the logins profile.
    let site = "127.0.0.1";
    let profile = daemon.browsers.logins_profile();
    let owner = json!({ "name": "session", "value": "owner", "domain": site, "path": "/",
        "expires": 4_000_000_000_u64 });
    daemon
        .browsers
        .profile_cookies(&profile, Some(CookieChange::Set(vec![owner])))
        .await
        .expect("seed");
    assert!(
        botloftd::logins::signed_in(&daemon, site)
            .await
            .expect("saved")
    );
    // A site the profile has nothing of is not a login.
    assert!(
        !botloftd::logins::signed_in(&daemon, "example.com")
            .await
            .expect("none")
    );

    let list = b.app.call("logins.list", Value::Null).await.expect("list");
    assert_eq!(list["logins"][0]["site"], site);
    assert_eq!(list["logins"][0]["botIds"], json!([]));
    assert_eq!(list["window"], Value::Null);

    let given = b
        .app
        .call(
            "logins.share",
            json!({ "site": site, "botIds": [b.bot["id"]] }),
        )
        .await
        .expect("share");
    assert_eq!(given["botIds"], json!([b.bot["id"]]));
    restarted(&mut b, 2).await;
    // The browser was closed: it opened for the copy and closed again,
    // keeping the cookie, and the site needs no approval now.
    assert!(
        cookies_seen(&mut b).await.contains("session=owner"),
        "the agent's browser sends the owner's cookie"
    );
    let workspace = std::path::PathBuf::from(b.bot["workspace"].as_str().expect("workspace"));
    let rules = std::fs::read_to_string(workspace.join(".claude/rules/logins.md")).expect("rules");
    assert!(rules.contains("- 127.0.0.1"), "{rules}");

    // Taken back from the open browser: the site gets nothing.
    b.app
        .call("logins.share", json!({ "site": site, "botIds": [] }))
        .await
        .expect("take");
    restarted(&mut b, 3).await;
    let seen = cookies_seen(&mut b).await;
    assert!(!seen.contains("session=owner"), "{seen}");
    assert!(!workspace.join(".claude/rules/logins.md").exists());

    let left = b
        .app
        .call("logins.remove", json!({ "site": site }))
        .await
        .expect("remove");
    assert_eq!(left["logins"], json!([]));
    let all = daemon
        .browsers
        .profile_cookies(&profile, None)
        .await
        .expect("read");
    assert!(
        botloftd::browser::cookies_of_site(&all, site).is_empty(),
        "the logins profile is signed out too"
    );
}

#[tokio::test]
async fn a_bad_address_or_an_unknown_login_is_refused() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let bad = app
        .call(
            "logins.open",
            json!({ "address": "ftp://files.example.com" }),
        )
        .await
        .expect_err("not the web");
    assert!(bad.message.contains("not a web address"), "{bad:?}");
    let unknown = app
        .call(
            "logins.share",
            json!({ "site": "reddit.com", "botIds": [] }),
        )
        .await
        .expect_err("no such login");
    assert!(unknown.message.contains("reddit.com"), "{unknown:?}");
    let list = app.call("logins.list", Value::Null).await.expect("list");
    assert_eq!(list, json!({ "logins": [], "window": null }));
}
