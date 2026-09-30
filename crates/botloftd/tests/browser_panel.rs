//! The owner's panel on a bot's browser with the real Microsoft Edge (spec
//! 21.3, 21.7): the page taking the shape of the room the panel has.
//! Skipped where Edge is not installed.

mod common;

use std::time::Duration;

use common::Client;
use common::browsing::{Browsing, answer_site, call, setup};
use serde_json::{Value, json};

/// Reads the page until it says its window is `size`.
async fn until_window(b: &mut Browsing, size: &str) -> String {
    let mut page = String::new();
    for _ in 0..50 {
        page = b
            .mcp
            .tool_text("browser_look", json!({}))
            .await
            .expect("look");
        if page.contains(&format!("Window: {size}")) {
            return page;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the window never became {size}:\n{page}");
}

/// The next frame that is `height` tall.
async fn frame_of(app: &mut Client, height: u64) -> Value {
    for _ in 0..50 {
        let frame = app.notification("browser.frame").await;
        if frame["height"] == height {
            return frame;
        }
    }
    panic!("no frame came {height} tall");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_page_takes_the_shape_of_the_panel_while_the_owner_watches() {
    let Some(mut b) = setup().await else { return };
    let id = b.bot["id"].clone();
    let room = |width: u32, height: u32| json!({ "botId": id, "width": width, "height": height });

    // Only a connection watching the browser sizes it, and with sane numbers.
    assert!(b.app.call("browser.resize", room(536, 700)).await.is_err());
    b.app
        .call("browser.watch", json!({ "botId": id }))
        .await
        .expect("watch");
    assert!(b.app.call("browser.resize", room(0, 700)).await.is_err());
    assert!(
        b.app
            .call("browser.resize", room(536, 20_000))
            .await
            .is_err()
    );

    // Asked before the browser opens: it opens that size.
    b.app
        .call("browser.resize", room(536, 700))
        .await
        .expect("resize");
    let opening = call(
        &b.mcp,
        "browser_open",
        json!({ "url": format!("{}/size", b.site) }),
    );
    answer_site(&mut b.app, true, None).await;
    let page = opening.await.expect("task").expect("page");
    assert!(page.contains("Window: 1280 x 1671"), "{page}");
    let frame = frame_of(&mut b.app, 1671).await;
    assert_eq!(frame["width"], 1280);

    // The panel changes: the page follows, and so do the frames.
    b.app
        .call("browser.resize", room(640, 400))
        .await
        .expect("resize");
    until_window(&mut b, "1280 x 800").await;
    frame_of(&mut b.app, 800).await;

    // The owner's points go as far as the page does now.
    b.app
        .call("browser.resize", room(640, 700))
        .await
        .expect("resize");
    until_window(&mut b, "1280 x 1400").await;
    b.app
        .call("browser.take", json!({ "botId": id }))
        .await
        .expect("take");
    let click = |y: f64| {
        json!({ "botId": id, "input": {
            "kind": "mouse", "action": "move", "x": 10.0, "y": y, "button": "none",
            "buttons": 0, "clicks": 0, "modifiers": 0,
        }})
    };
    b.app
        .call("browser.input", click(1390.0))
        .await
        .expect("a point on the taller page");
    assert!(b.app.call("browser.input", click(1410.0)).await.is_err());

    // Nobody watches anymore: back to the size bots work in alone.
    b.app
        .call("browser.unwatch", Value::Null)
        .await
        .expect("unwatch");
    until_window(&mut b, "1280 x 800").await;
}
