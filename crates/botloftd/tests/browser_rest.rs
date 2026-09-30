//! A bot's browser resting when nobody uses it, with the real Microsoft Edge
//! (spec 21.2): it rests the moment the bot's turn ends, keeps a picture
//! for whoever looks, and wakes for the bot's next tool, the owner's hands
//! and a reload. Skipped where Edge is not installed.

mod common;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use common::browsing::{Browsing, answer_site, call, setup};
use common::{Client, stream};
use serde_json::{Value, json};

async fn browser(app: &mut Client) -> Value {
    let list = app.call("browser.list", Value::Null).await.expect("list");
    list[0].clone()
}

/// Waits until the bot's browser rests, or stops resting.
async fn until_resting(app: &mut Client, resting: bool) {
    for _ in 0..100 {
        if browser(app).await["resting"] == resting {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("the browser never got to resting = {resting}");
}

async fn look(b: &mut Browsing) -> String {
    b.mcp
        .tool_text("browser_look", json!({}))
        .await
        .expect("look")
}

#[tokio::test(flavor = "multi_thread")]
async fn the_browser_rests_when_the_turn_ends_and_wakes_when_it_is_used() {
    let Some(mut b) = setup().await else { return };
    tokio::spawn(botloftd::browser::run(Arc::clone(&b.t.daemon)));
    let id = json!({ "botId": b.bot["id"] });
    let bot: BotId = b.bot["id"].as_str().expect("id").parse().expect("bot id");
    let process = b.t.process_of(&b.bot).await;

    // A turn: the owner asks, and the bot opens a page.
    let ask = json!({ "botId": b.bot["id"], "body": "Open the page" });
    b.app.call("messages.send", ask).await.expect("send");
    let line = process.wait_lines(1).await.pop().expect("line");
    b.t.until_state(&b.bot, BotState::Busy).await;
    let url = format!("{}/sight", b.site);
    let opening = call(&b.mcp, "browser_open", json!({ "url": url }));
    answer_site(&mut b.app, true, None).await;
    let page = opening.await.expect("task").expect("page");
    assert!(page.contains("Hidden 0 times"), "{page}");
    assert_eq!(browser(&mut b.app).await["resting"], false);

    // The turn ends: the browser rests at once, with nobody asking.
    stream::answer(&process, &line, "It is open.").await;
    b.t.until_state(&b.bot, BotState::Idle).await;
    until_resting(&mut b.app, true).await;

    // Whoever looks now sees the page as it was, though nobody watched.
    let view = b
        .app
        .call("browser.watch", id.clone())
        .await
        .expect("watch");
    assert_eq!(view["state"]["resting"], true);
    assert_eq!(view["state"]["status"], "open");
    assert!(view["frame"]["data"].as_str().expect("picture").len() > 100);

    // The bot's next tool wakes it: the page was out of sight once, and
    // nothing in it was lost. Frames come again.
    b.app.forget("browser.frame");
    let page = look(&mut b).await;
    assert!(page.contains("Hidden 1 times"), "{page}");
    assert_eq!(browser(&mut b.app).await["resting"], false);
    b.app.notification("browser.frame").await;

    // So do the owner's hands, and a browser in them does not rest.
    b.t.daemon.browsers.rest(&bot);
    until_resting(&mut b.app, true).await;
    b.app.call("browser.take", id.clone()).await.expect("take");
    until_resting(&mut b.app, false).await;
    b.t.daemon.browsers.rest(&bot);
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(browser(&mut b.app).await["resting"], false);
    b.app
        .call("browser.release", id.clone())
        .await
        .expect("release");

    // And a reload, without taking the browser.
    b.t.daemon.browsers.rest(&bot);
    until_resting(&mut b.app, true).await;
    b.app.call("browser.reload", id).await.expect("reload");
    until_resting(&mut b.app, false).await;
    let page = look(&mut b).await;
    assert!(page.contains("The owner reloaded the page."), "{page}");
    assert!(page.contains("Hidden 0 times"), "{page}");
}
