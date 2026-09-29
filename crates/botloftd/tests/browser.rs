//! Each bot's browser with the real Microsoft Edge (spec 21): opening pages,
//! filling a form, asking the owner about new sites, files, new tabs,
//! dialogs and the live view. Skipped where Edge is not installed.

mod common;

use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon, site};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

struct Browsing {
    t: TestDaemon,
    app: Client,
    bot: Value,
    mcp: Mcp,
    site: String,
}

/// A bot in Manual mode and a local site, or `None` without Edge.
async fn setup() -> Option<Browsing> {
    if botloftd::browser::find_program("").is_none() {
        eprintln!("Microsoft Edge is not installed; skipping");
        return None;
    }
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Web" }))
        .await
        .expect("crew");
    let (bot, _, mcp) = ready_bot(&t, &mut app, &crew, "Scout").await;
    let site = format!("http://{}", site::serve().await);
    Some(Browsing {
        t,
        app,
        bot,
        mcp,
        site,
    })
}

fn call(mcp: &Mcp, name: &str, arguments: Value) -> JoinHandle<Result<String, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    let name = name.to_owned();
    tokio::spawn(async move { mcp.tool_text(&name, arguments).await })
}

/// Waits for the bot to ask about a site and answers.
async fn answer_site(app: &mut Client, allow: bool, note: Option<&str>) -> Value {
    let asked = loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            break body.clone();
        }
    };
    assert_eq!(asked["toolName"], "mcp__botloft__browser");
    let mut answer = json!({ "approvalId": asked["approvalId"], "allow": allow });
    if let Some(note) = note {
        answer["note"] = json!(note);
    }
    app.call("approvals.answer", answer).await.expect("answer");
    asked
}

/// The ref of the first control on the page whose line has `what`.
fn ref_of(page: &str, what: &str) -> String {
    let start = page
        .find(what)
        .unwrap_or_else(|| panic!("{what} is not on the page:\n{page}"));
    let open = page[..start].rfind('[').expect("a control");
    page[open + 1..]
        .split_whitespace()
        .next()
        .expect("ref")
        .to_owned()
}

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
    let open = b.app.call("browser.list", Value::Null).await.expect("list");
    assert_eq!(open[0]["tabs"], 2);

    let picture = b.mcp.tool_result("browser_screenshot", json!({})).await;
    assert_eq!(picture["content"][0]["type"], "image");
    assert_eq!(picture["content"][0]["mimeType"], "image/jpeg");

    b.app
        .call("browser.unwatch", Value::Null)
        .await
        .expect("unwatch");
}
