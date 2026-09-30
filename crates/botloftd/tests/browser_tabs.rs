//! The owner reloading a bot's page and moving between its tabs with the
//! real Microsoft Edge (spec 21.7, 21.10): the tabs the app sees, a new tab
//! taken to an address, switching, and what the bot reads afterwards.
//! Skipped where Edge is not installed.

mod common;

use std::time::Duration;

use common::Client;
use common::browsing::{Browsing, answer_site, call, pending_approval, setup};
use serde_json::{Value, json};

async fn browser(app: &mut Client) -> Value {
    let list = app.call("browser.list", Value::Null).await.expect("list");
    list[0].clone()
}

fn active(state: &Value) -> Value {
    let tabs = state["tabs"].as_array().cloned().unwrap_or_default();
    tabs.into_iter()
        .find(|tab| tab["active"] == true)
        .unwrap_or(Value::Null)
}

/// Waits until the bot's browser is as `ready` wants.
async fn until(app: &mut Client, what: &str, ready: impl Fn(&Value) -> bool) -> Value {
    for _ in 0..100 {
        let state = browser(app).await;
        if ready(&state) {
            return state;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the browser never got to: {what}");
}

/// The bot opens the page that counts its loads; the owner allows the site
/// and watches.
async fn visits(b: &mut Browsing) -> Value {
    let url = format!("{}/visits", b.site);
    let opening = call(&b.mcp, "browser_open", json!({ "url": url }));
    answer_site(&mut b.app, true, None).await;
    let page = opening.await.expect("task").expect("page");
    assert!(page.contains("Loaded 1 times"), "{page}");
    json!({ "botId": b.bot["id"] })
}

async fn look(b: &mut Browsing) -> String {
    b.mcp
        .tool_text("browser_look", json!({}))
        .await
        .expect("look")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_reloads_the_page_without_taking_the_browser() {
    let Some(mut b) = setup().await else { return };
    let id = visits(&mut b).await;

    // Only a connection watching the browser reloads it.
    assert!(b.app.call("browser.reload", id.clone()).await.is_err());
    b.app
        .call("browser.watch", id.clone())
        .await
        .expect("watch");
    b.app
        .call("browser.reload", id.clone())
        .await
        .expect("reload");
    assert_eq!(browser(&mut b.app).await["control"], "bot");

    // The bot reads the page again, and why it changed.
    let mut page = String::new();
    for _ in 0..50 {
        page.push_str(&look(&mut b).await);
        if page.contains("Loaded 2 times") {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(page.contains("Loaded 2 times"), "{page}");
    assert!(page.contains("The owner reloaded the page."), "{page}");
    let again = look(&mut b).await;
    assert!(!again.contains("reloaded"), "{again}");

    b.mcp
        .tool_text("browser_close", json!({}))
        .await
        .expect("closed");
    assert!(b.app.call("browser.reload", id).await.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_moves_between_tabs_and_the_bot_reads_where_it_is() {
    let Some(mut b) = setup().await else { return };
    let id = visits(&mut b).await;
    let bot = b.bot["id"].clone();
    let tab = |tab: &Value| json!({ "botId": bot, "tabId": tab });
    let address = |url: &str| json!({ "botId": bot, "url": url });
    b.app
        .call("browser.watch", id.clone())
        .await
        .expect("watch");

    let state = until(&mut b.app, "the page's title", |state| {
        state["tabs"][0]["title"] == "Visits"
    })
    .await;
    assert_eq!(state["tabs"].as_array().map(Vec::len), Some(1));
    assert_eq!(state["tabs"][0]["active"], true);
    let first = state["tabs"][0]["id"].clone();

    // Tabs and addresses are for the owner's hands only.
    assert!(b.app.call("browser.newTab", id.clone()).await.is_err());
    assert!(b.app.call("browser.switchTab", tab(&first)).await.is_err());
    assert!(b.app.call("browser.open", address(&b.site)).await.is_err());
    b.app.call("browser.take", id.clone()).await.expect("take");

    // A new tab, blank and active, after the one the bot opened.
    b.app
        .call("browser.newTab", id.clone())
        .await
        .expect("new tab");
    let state = until(&mut b.app, "a blank second tab", |state| {
        active(state)["url"] == "about:blank"
    })
    .await;
    assert_eq!(state["tabs"].as_array().map(Vec::len), Some(2));
    assert_eq!(state["tabs"][0]["id"], first);
    assert_eq!(state["tabs"][1]["active"], true);

    // The owner types where it goes; only web addresses open.
    for refused in ["javascript:alert(1)", "file:///C:/x.html", "two words"] {
        let error = b.app.call("browser.open", address(refused)).await;
        assert!(error.is_err(), "{refused} opened");
    }
    let done = format!("{}/done?name=Owner&size=Small", b.site);
    b.app
        .call("browser.open", address(&done))
        .await
        .expect("open");
    let state = until(&mut b.app, "the page the owner opened", |state| {
        active(state)["title"] == "Thanks" && state["loading"] == false
    })
    .await;
    assert!(state["url"].as_str().expect("url").contains("/done"));
    let second = active(&state)["id"].clone();
    assert_ne!(second, first);

    // Back to the first tab: it is the active one, and the frames are its.
    let gone = b.app.call("browser.switchTab", tab(&json!("nope"))).await;
    assert!(gone.expect_err("no such tab").message.contains("not open"));
    b.app.forget("browser.frame");
    b.app
        .call("browser.switchTab", tab(&first))
        .await
        .expect("switch");
    let state = until(&mut b.app, "the first tab again", |state| {
        active(state)["id"] == first
    })
    .await;
    assert_eq!(state["title"], "Visits");
    let frame = b.app.notification("browser.frame").await;
    assert_eq!(frame["botId"], bot);

    // Given back on the tab the bot was on: nothing to tell it.
    b.app
        .call("browser.release", id.clone())
        .await
        .expect("release");
    let page = look(&mut b).await;
    assert!(page.contains("Loaded 1 times"), "{page}");
    assert!(!page.contains("switched tabs"), "{page}");

    // Given back on another tab: the bot does not act before it reads.
    b.app.call("browser.take", id.clone()).await.expect("take");
    b.app
        .call("browser.switchTab", tab(&second))
        .await
        .expect("switch");
    b.app
        .call("browser.release", id.clone())
        .await
        .expect("release");
    let refused = b
        .mcp
        .tool_text("browser_press", json!({ "key": "Enter" }))
        .await
        .expect_err("not done");
    assert!(refused.contains("switched tabs"), "{refused}");
    assert!(refused.contains("browser_look"), "{refused}");
    let page = look(&mut b).await;
    assert!(page.contains("# Thanks, Owner"), "{page}");
    assert!(!page.contains("switched tabs"), "{page}");

    // A tool that only reads says so with the page, once.
    b.app.call("browser.take", id.clone()).await.expect("take");
    b.app
        .call("browser.switchTab", tab(&first))
        .await
        .expect("switch");
    b.app
        .call("browser.release", id.clone())
        .await
        .expect("release");
    let page = look(&mut b).await;
    assert!(page.contains("The owner switched tabs"), "{page}");
    assert!(page.contains("Loaded 1 times"), "{page}");
    assert!(!look(&mut b).await.contains("switched tabs"));

    // So does the answer to a request for help.
    let asking = call(
        &b.mcp,
        "browser_ask_owner",
        json!({ "task": "Pick the page" }),
    );
    pending_approval(&mut b.app).await;
    b.app.call("browser.take", id.clone()).await.expect("take");
    b.app
        .call("browser.switchTab", tab(&second))
        .await
        .expect("switch");
    b.app.call("browser.release", id).await.expect("release");
    let page = asking.await.expect("task").expect("done");
    assert!(page.contains("The owner is done"), "{page}");
    assert!(page.contains("The owner switched tabs"), "{page}");
    assert!(page.contains("# Thanks, Owner"), "{page}");
}
