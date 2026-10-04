//! Teaching a bot a task with the real Microsoft Edge (spec 21.13): what
//! the owner does becomes steps by what it means, never by what was typed.
//! Skipped where Edge is not installed.

mod common;

use common::browsing::{Browsing, answer_site, call, click, press, send, setup};
use serde_json::{Value, json};

async fn sign_in_page(b: &mut Browsing) {
    let url = format!("{}/login", b.site);
    let opening = call(&b.mcp, "browser_open", json!({ "url": url }));
    answer_site(&mut b.app, true, None).await;
    opening.await.expect("task").expect("page");
}

async fn teach(b: &mut Browsing, on: bool) -> Value {
    let params = json!({ "botId": b.bot["id"], "on": on });
    b.app.call("browser.teach", params).await.expect("teach")
}

#[tokio::test(flavor = "multi_thread")]
async fn a_lesson_keeps_the_steps_and_never_what_was_typed() {
    let Some(mut b) = setup().await else { return };
    sign_in_page(&mut b).await;
    let bot = b.bot.clone();
    let id = json!({ "botId": bot["id"] });
    b.app
        .call("browser.watch", id.clone())
        .await
        .expect("watch");

    // Only with the browser in the owner's hands.
    let early = json!({ "botId": bot["id"], "on": true });
    assert!(b.app.call("browser.teach", early).await.is_err());
    b.app.call("browser.take", id.clone()).await.expect("take");

    let started = teach(&mut b, true).await;
    assert_eq!(started[0]["kind"], "open");
    assert_eq!(started[0]["label"], format!("{}/login", b.site));

    click(&mut b.app, &bot, 200.0, 115.0).await;
    press(&mut b.app, &bot, "ana", 0).await;
    click(&mut b.app, &bot, 200.0, 215.0).await;
    send(
        &mut b.app,
        &bot,
        json!({ "kind": "text", "text": "hunter2" }),
    )
    .await;
    let enter = json!({ "kind": "key", "key": "Enter", "code": "Enter", "modifiers": 0 });
    send(&mut b.app, &bot, enter).await;

    // The steps reach the app as they happen.
    let mut lesson = Value::Null;
    for _ in 0..50 {
        let list = b.app.call("browser.list", Value::Null).await.expect("list");
        lesson = list[0]["lesson"].clone();
        if lesson.as_array().is_some_and(|steps| steps.len() >= 6) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let steps = teach(&mut b, false).await;
    assert_eq!(steps, lesson);
    let kinds: Vec<_> = steps
        .as_array()
        .expect("steps")
        .iter()
        .map(|step| {
            format!(
                "{} {}",
                step["kind"].as_str().unwrap_or(""),
                step["label"].as_str().unwrap_or("")
            )
        })
        .collect();
    assert_eq!(
        kinds[1..],
        [
            "click User",
            "type User",
            "click Password",
            "type Password",
            "press Enter"
        ]
    );
    assert_eq!(steps[4]["secret"], true);
    assert!(!steps.to_string().contains("ana\""), "{steps}");
    assert!(!steps.to_string().contains("hunter2"), "{steps}");

    // Ended, nothing more is kept.
    let list = b.app.call("browser.list", Value::Null).await.expect("list");
    assert_eq!(list[0]["lesson"], Value::Null);
}
