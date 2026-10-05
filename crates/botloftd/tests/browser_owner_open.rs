//! The owner opens a bot's closed browser for themselves with the real
//! Microsoft Edge (spec 21.10): on the page it showed last, without the
//! bot, then takes it and gives it back. Skipped where Edge is not
//! installed.

mod common;

use std::time::Duration;

use common::Client;
use common::browsing::{answer_site, call, setup};
use serde_json::{Value, json};

async fn browser(app: &mut Client) -> Value {
    let list = app.call("browser.list", Value::Null).await.expect("list");
    list[0].clone()
}

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_opens_the_closed_browser_on_its_last_page_and_takes_it() {
    let Some(mut b) = setup().await else { return };
    let url = format!("{}/login", b.site);
    let opening = call(&b.mcp, "browser_open", json!({ "url": url }));
    answer_site(&mut b.app, true, None).await;
    opening.await.expect("task").expect("page");
    b.mcp
        .tool_text("browser_close", json!({}))
        .await
        .expect("closed");

    // Only the connection watching it opens it.
    let id = json!({ "botId": b.bot["id"] });
    assert!(b.app.call("browser.start", id.clone()).await.is_err());
    b.app
        .call("browser.watch", id.clone())
        .await
        .expect("watch");
    b.app
        .call("browser.start", id.clone())
        .await
        .expect("start");

    // It opens on the page it showed last.
    let mut state = Value::Null;
    for _ in 0..150 {
        state = browser(&mut b.app).await;
        if state["status"] == "open"
            && state["url"]
                .as_str()
                .is_some_and(|url| url.contains("/login"))
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(state["status"], "open", "{state}");
    assert!(
        state["url"]
            .as_str()
            .is_some_and(|url| url.contains("/login")),
        "{state}"
    );

    let taken = b.app.call("browser.take", id.clone()).await.expect("take");
    assert_eq!(taken["control"], "owner");
    let given = b
        .app
        .call("browser.release", id.clone())
        .await
        .expect("release");
    assert_eq!(given["control"], "bot");

    // A paused bot's browser stays closed.
    b.app
        .call(
            "bots.setPaused",
            json!({ "botId": b.bot["id"], "paused": true }),
        )
        .await
        .expect("pause");
    assert!(b.app.call("browser.start", id).await.is_err());
}
