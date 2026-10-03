//! Each bot's browser with the real Microsoft Edge (spec 21): opening pages,
//! filling a form, asking the owner about new sites, files, new tabs,
//! dialogs and the live view. Skipped where Edge is not installed.

mod common;

use std::time::{Duration, Instant};

use common::browsing::{answer_site, call, ref_of, setup};
use serde_json::{Value, json};

#[tokio::test(flavor = "multi_thread")]
async fn a_bot_fills_a_form_after_the_owner_allows_the_site() {
    let Some(mut b) = setup().await else { return };
    let opening = call(
        &b.mcp,
        "browser_open",
        json!({ "url": format!("{}/", b.site) }),
    );
    let asked = answer_site(&mut b.app, true, None).await;
    assert_eq!(asked["summary"], "127.0.0.1");
    let page = opening.await.expect("task").expect("page");
    assert!(page.contains("Page: Order"), "{page}");
    assert!(page.contains("# Order a cake"), "{page}");
    assert!(!page.contains("Hidden text"), "{page}");

    let name = ref_of(&page, "textbox \"Name\"");
    let size = ref_of(&page, "select \"Size\"");
    let send = ref_of(&page, "button \"Send\"");
    let typed = b
        .mcp
        .tool_text("browser_type", json!({ "ref": name, "text": "Ana" }))
        .await
        .expect("typed");
    assert!(typed.contains("= \"Ana\""), "{typed}");
    let chose = b
        .mcp
        .tool_text("browser_select", json!({ "ref": size, "option": "large" }))
        .await
        .expect("chose");
    assert!(chose.contains("Chose \"Large\""), "{chose}");
    let sent = b
        .mcp
        .tool_text("browser_click", json!({ "ref": send }))
        .await
        .expect("sent");
    assert!(sent.contains("# Thanks, Ana"), "{sent}");
    assert!(sent.contains("Size: Large"), "{sent}");

    // The site is known now: no second question.
    let again = b
        .mcp
        .tool_text(
            "browser_open",
            json!({ "url": format!("{}/done?name=Bo", b.site) }),
        )
        .await
        .expect("again");
    assert!(again.contains("Thanks, Bo"), "{again}");
    let back = b
        .mcp
        .tool_text("browser_back", json!({}))
        .await
        .expect("back");
    assert!(back.contains("Thanks, Ana"), "{back}");

    let open = b.app.call("browser.list", Value::Null).await.expect("list");
    assert_eq!(open[0]["botId"], b.bot["id"]);
    assert_eq!(open[0]["status"], "open");
    assert!(open[0]["url"].as_str().expect("url").contains("/done"));

    let stale = b
        .mcp
        .tool_text("browser_click", json!({ "ref": send }))
        .await;
    assert!(
        stale
            .expect_err("stale")
            .contains("not on the page anymore")
    );

    b.mcp
        .tool_text("browser_close", json!({}))
        .await
        .expect("closed");
    let open = b.app.call("browser.list", Value::Null).await.expect("list");
    assert_eq!(open, json!([]));
    let closed = b.mcp.tool_text("browser_look", json!({})).await;
    assert!(closed.expect_err("closed").contains("not open"));
}

#[tokio::test(flavor = "multi_thread")]
async fn words_made_of_one_element_per_letter_keep_their_spaces() {
    let Some(mut b) = setup().await else { return };
    let opening = call(
        &b.mcp,
        "browser_open",
        json!({ "url": format!("{}/letters", b.site) }),
    );
    answer_site(&mut b.app, true, None).await;
    let page = opening.await.expect("task").expect("page");
    assert!(page.contains("Two words"), "{page}");
}

#[tokio::test(flavor = "multi_thread")]
async fn sites_see_a_common_browser() {
    let Some(mut b) = setup().await else { return };
    let opening = call(
        &b.mcp,
        "browser_open",
        json!({ "url": format!("{}/signals", b.site) }),
    );
    answer_site(&mut b.app, true, None).await;
    let page = opening.await.expect("task").expect("page");
    assert!(page.contains("Webdriver: false"), "{page}");
    assert!(page.contains("Brands: Chromium"), "{page}");
    assert!(!page.contains("Headless"), "{page}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_site_the_owner_denies_stays_closed() {
    let Some(mut b) = setup().await else { return };
    let opening = call(&b.mcp, "browser_open", json!({ "url": b.site.clone() }));
    answer_site(&mut b.app, false, Some("Not that one")).await;
    let refused = opening.await.expect("task").expect_err("denied");
    assert!(refused.contains("did not allow 127.0.0.1"), "{refused}");
    assert!(refused.contains("Not that one"), "{refused}");
    let _ = &b.t;
}

#[tokio::test(flavor = "multi_thread")]
async fn files_open_only_from_the_bot_folders() {
    let Some(mut b) = setup().await else { return };
    let workspace = std::path::PathBuf::from(b.bot["workspace"].as_str().expect("workspace"));
    std::fs::write(
        workspace.join("page.html"),
        "<h1>My page</h1><p>Made by Scout</p>",
    )
    .expect("write");
    let page = b
        .mcp
        .tool_text("browser_open", json!({ "url": "page.html" }))
        .await
        .expect("own file");
    assert!(page.contains("# My page"), "{page}");
    assert!(page.contains("URL: file:///"), "{page}");

    let outside = b.t.paths.home.join("elsewhere.html");
    std::fs::write(&outside, "<p>not yours</p>").expect("write");
    let refused = b
        .mcp
        .tool_text("browser_open", json!({ "url": outside.to_string_lossy() }))
        .await
        .expect_err("outside");
    assert!(refused.contains("not a file in your folders"), "{refused}");
    let nothing = b
        .mcp
        .tool_text("browser_open", json!({ "url": "javascript:alert(1)" }))
        .await;
    assert!(nothing.is_err());
    let _ = &mut b.app;
}

#[tokio::test(flavor = "multi_thread")]
async fn new_tabs_and_dialogs_reach_the_bot_and_the_owner_watches() {
    let Some(mut b) = setup().await else { return };
    let view = b
        .app
        .call("browser.watch", json!({ "botId": b.bot["id"] }))
        .await
        .expect("watch");
    assert_eq!(view["state"]["status"], "closed");
    assert_eq!(view["frame"], Value::Null);

    let opening = call(
        &b.mcp,
        "browser_open",
        json!({ "url": format!("{}/links", b.site) }),
    );
    answer_site(&mut b.app, true, None).await;
    let page = opening.await.expect("task").expect("page");
    let frame = b.app.notification("browser.frame").await;
    assert_eq!(frame["botId"], b.bot["id"]);
    assert_eq!(frame["width"], 1280);
    assert!(frame["data"].as_str().expect("jpeg").len() > 100);

    let warn = ref_of(&page, "button \"Warn me\"");
    let warned = b
        .mcp
        .tool_text("browser_click", json!({ "ref": warn }))
        .await
        .expect("clicked");
    assert!(
        warned.contains("dialog (alert) and it was accepted: \"Hi there\""),
        "{warned}"
    );
    let action = loop {
        let action = b.app.notification("browser.action").await;
        if action["kind"] == "click" {
            break action;
        }
    };
    assert_eq!(action["label"], "Warn me");
    assert!(action["x"].as_f64().expect("x") > 0.0);

    let link = ref_of(&page, "link \"Open in a new tab\"");
    let tab = b
        .mcp
        .tool_text("browser_click", json!({ "ref": link }))
        .await
        .expect("new tab");
    assert!(tab.contains("Thanks, Tab"), "{tab}");
    // The new tab is the active one, after the tab that opened it. The app
    // may hear of its address a moment after the bot read the page: the
    // events that bring it wait while the new tab is set up.
    let deadline = Instant::now() + Duration::from_secs(5);
    let tabs = loop {
        let open = b.app.call("browser.list", Value::Null).await.expect("list");
        let tabs = open[0]["tabs"].as_array().expect("tabs").clone();
        let shown = tabs.len() == 2
            && tabs[1]["url"]
                .as_str()
                .is_some_and(|url| url.contains("/done"));
        if shown || Instant::now() > deadline {
            break tabs;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    };
    assert_eq!(tabs.len(), 2, "{tabs:?}");
    assert_eq!(tabs[0]["active"], false);
    assert_eq!(tabs[1]["active"], true);
    assert!(
        tabs[1]["url"].as_str().expect("url").contains("/done"),
        "{tabs:?}"
    );

    let picture = b.mcp.tool_result("browser_screenshot", json!({})).await;
    assert_eq!(picture["content"][0]["type"], "image");
    assert_eq!(picture["content"][0]["mimeType"], "image/jpeg");

    b.app
        .call("browser.unwatch", Value::Null)
        .await
        .expect("unwatch");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_sees_the_cursor_get_there_before_the_click() {
    let Some(mut b) = setup().await else { return };
    b.app
        .call("browser.watch", json!({ "botId": b.bot["id"] }))
        .await
        .expect("watch");
    let opening = call(
        &b.mcp,
        "browser_open",
        json!({ "url": format!("{}/links", b.site) }),
    );
    answer_site(&mut b.app, true, None).await;
    let page = opening.await.expect("task").expect("page");

    let warn = ref_of(&page, "button \"Warn me\"");
    let clicking = call(&b.mcp, "browser_click", json!({ "ref": warn }));
    let action = loop {
        let action = b.app.notification("browser.action").await;
        if action["kind"] == "click" {
            break action;
        }
    };
    let pointed = Instant::now();
    assert_eq!(action["label"], "Warn me");
    assert!(action["x"].as_f64().expect("x") > 0.0);
    // The click waits for the cursor's glide in the panel.
    assert!(!clicking.is_finished());
    let warned = clicking.await.expect("task").expect("clicked");
    assert!(warned.contains("\"Hi there\""), "{warned}");
    assert!(pointed.elapsed() >= Duration::from_millis(500));
}
