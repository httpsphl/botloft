//! The owner's hands in a bot's browser with the real Microsoft Edge (spec
//! 21.10): taking it, clicking and typing on the page, the bot waiting
//! meanwhile, and the bot asking for help. Skipped where Edge is not
//! installed.

mod common;

use std::time::Duration;

use common::Client;
use common::browsing::{Browsing, answer_site, call, pending_approval, setup};
use serde_json::{Value, json};

async fn send(app: &mut Client, bot: &Value, input: Value) {
    app.call(
        "browser.input",
        json!({ "botId": bot["id"], "input": input }),
    )
    .await
    .expect("input");
}

async fn click(app: &mut Client, bot: &Value, x: f64, y: f64) {
    for (action, buttons) in [("move", 0), ("down", 1), ("up", 0)] {
        let input = json!({
            "kind": "mouse", "action": action, "x": x, "y": y, "button": "left",
            "buttons": buttons, "clicks": 1, "modifiers": 0,
        });
        send(app, bot, input).await;
    }
}

/// Presses the keys of `text`, one by one, as the app sends them.
async fn press(app: &mut Client, bot: &Value, text: &str, modifiers: u32) {
    for key in text.chars() {
        let code = format!("Key{}", key.to_ascii_uppercase());
        let input =
            json!({ "kind": "key", "key": key.to_string(), "code": code, "modifiers": modifiers });
        send(app, bot, input).await;
    }
}

/// The bot opens the sign-in page; the owner allows the site.
async fn sign_in_page(b: &mut Browsing) -> String {
    let url = format!("{}/login", b.site);
    let opening = call(&b.mcp, "browser_open", json!({ "url": url }));
    answer_site(&mut b.app, true, None).await;
    opening.await.expect("task").expect("page")
}

async fn browser(app: &mut Client) -> Value {
    let list = app.call("browser.list", Value::Null).await.expect("list");
    list[0].clone()
}

/// Waits until the bot's page has `part` in its address and has loaded.
async fn until_url(app: &mut Client, part: &str) {
    for _ in 0..100 {
        let state = browser(app).await;
        if state["url"].as_str().is_some_and(|url| url.contains(part)) && state["loading"] == false
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the page never showed {part}");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_signs_in_for_the_bot_and_gives_the_browser_back() {
    let Some(mut b) = setup().await else { return };
    let page = sign_in_page(&mut b).await;
    assert!(page.contains("(password)"), "{page}");
    let bot = b.bot.clone();
    let id = json!({ "botId": bot["id"] });

    // Only the connection watching the browser takes it, and only then
    // sends events.
    assert!(b.app.call("browser.take", id.clone()).await.is_err());
    b.app
        .call("browser.watch", id.clone())
        .await
        .expect("watch");
    let early = json!({ "botId": bot["id"], "input": { "kind": "text", "text": "x" } });
    assert!(b.app.call("browser.input", early).await.is_err());

    let task = json!({ "task": "Sign in to your account" });
    let asking = call(&b.mcp, "browser_ask_owner", task);
    let asked = pending_approval(&mut b.app).await;
    assert_eq!(asked["toolName"], "mcp__botloft__browser_help");
    assert_eq!(asked["summary"], "Sign in to your account");
    assert_eq!(browser(&mut b.app).await["ask"], "Sign in to your account");

    let state = b.app.call("browser.take", id.clone()).await.expect("take");
    assert_eq!(state["control"], "owner");
    click(&mut b.app, &bot, 200.0, 115.0).await;
    press(&mut b.app, &bot, "ana", 0).await;
    click(&mut b.app, &bot, 200.0, 215.0).await;
    let secret = json!({ "kind": "text", "text": "hunter2" });
    send(&mut b.app, &bot, secret).await;
    let enter = json!({ "kind": "key", "key": "Enter", "code": "Enter", "modifiers": 0 });
    send(&mut b.app, &bot, enter).await;
    until_url(&mut b.app, "/account").await;

    let state = b.app.call("browser.release", id).await.expect("release");
    assert_eq!(state["control"], "bot");
    let page = asking.await.expect("task").expect("done");
    assert!(page.contains("The owner is done"), "{page}");
    assert!(page.contains("# Welcome, ana"), "{page}");
    assert!(page.contains("Password length: 7"), "{page}");
    assert!(!page.contains("hunter2"), "{page}");
    assert_eq!(browser(&mut b.app).await["ask"], Value::Null);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_bot_waits_while_the_owner_has_its_browser() {
    let Some(mut b) = setup().await else { return };
    sign_in_page(&mut b).await;
    let bot = b.bot.clone();
    let id = json!({ "botId": bot["id"] });
    b.app
        .call("browser.watch", id.clone())
        .await
        .expect("watch");
    b.app.call("browser.take", id.clone()).await.expect("take");

    // One pair of hands at a time.
    let mut other = b.t.session().await;
    other
        .call("browser.watch", id.clone())
        .await
        .expect("watch");
    assert!(other.call("browser.take", id.clone()).await.is_err());

    click(&mut b.app, &bot, 200.0, 115.0).await;
    press(&mut b.app, &bot, "abc", 0).await;
    // Ctrl+A (Cmd+A on a Mac) selects the field's text, so the next keys
    // replace it.
    let command = if cfg!(target_os = "macos") { 4 } else { 2 };
    press(&mut b.app, &bot, "a", command).await;
    press(&mut b.app, &bot, "xyz", 0).await;

    let looking = call(&b.mcp, "browser_look", json!({}));
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert!(
        !looking.is_finished(),
        "the bot acted while the owner had the browser"
    );
    // Watching stops, so the browser goes back to the bot.
    b.app
        .call("browser.unwatch", Value::Null)
        .await
        .expect("unwatch");
    let page = looking.await.expect("task").expect("page");
    assert!(page.contains("= \"xyz\""), "{page}");
    assert_eq!(browser(&mut b.app).await["control"], "bot");

    // The owner may say no, with a note the bot reads.
    let asking = call(
        &b.mcp,
        "browser_ask_owner",
        json!({ "task": "Solve the captcha" }),
    );
    let asked = pending_approval(&mut b.app).await;
    let no = json!({ "approvalId": asked["approvalId"], "allow": false, "note": "Not now" });
    b.app.call("approvals.answer", no).await.expect("answer");
    let refused = asking.await.expect("task").expect_err("denied");
    assert!(refused.contains("did not do it"), "{refused}");
    assert!(refused.contains("Not now"), "{refused}");
}
