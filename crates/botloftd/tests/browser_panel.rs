//! The owner's panel on a bot's browser with the real Microsoft Edge (spec
//! 21.3, 21.7): the page taking the shape of the room the panel has, drawn
//! as sharp as the owner's screen. Skipped where Edge is not installed.

mod common;

use std::time::Duration;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use common::Client;
use common::browsing::{Browsing, answer_site, call, ref_of, setup};
use serde_json::{Value, json};

/// The width and height of a JPEG, base64, from its frame header.
fn jpeg_size(data: &str) -> (u32, u32) {
    let bytes = BASE64.decode(data).expect("base64");
    let mut at = 2;
    while at + 9 < bytes.len() {
        assert_eq!(bytes[at], 0xFF, "a JPEG marker");
        let marker = bytes[at + 1];
        let length = usize::from(bytes[at + 2]) << 8 | usize::from(bytes[at + 3]);
        if (0xC0..=0xCF).contains(&marker) && ![0xC4, 0xC8, 0xCC].contains(&marker) {
            let height = u32::from(bytes[at + 5]) << 8 | u32::from(bytes[at + 6]);
            let width = u32::from(bytes[at + 7]) << 8 | u32::from(bytes[at + 8]);
            return (width, height);
        }
        at += 2 + length;
    }
    panic!("no size in the picture");
}

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
    // A narrow panel: the desktop layout's width, in the panel's shape.
    assert!(page.contains("Window: 800 x 1044"), "{page}");
    let frame = frame_of(&mut b.app, 1044).await;
    assert_eq!(frame["width"], 800);

    // The panel changes: the page follows, and so do the frames.
    b.app
        .call("browser.resize", room(1000, 400))
        .await
        .expect("resize");
    until_window(&mut b, "1000 x 600").await;
    frame_of(&mut b.app, 600).await;

    // The owner's points go as far as the page does now.
    b.app
        .call("browser.resize", room(1000, 1400))
        .await
        .expect("resize");
    until_window(&mut b, "1000 x 1400").await;
    frame_of(&mut b.app, 1400).await;
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

/// The owner's screen is scaled above 100% (Windows at 125%, 150%, 200%): the
/// app says so in `browser.resize`. The page must stay in its own pixels and
/// every click, the bot's and the owner's, must land where it aimed. With a
/// sharper page the browser divided the clicks by the factor (spec 19).
#[tokio::test(flavor = "multi_thread")]
async fn a_scaled_screen_keeps_the_page_in_its_pixels_and_the_clicks_where_aimed() {
    let Some(mut b) = setup().await else { return };
    let id = b.bot["id"].clone();
    b.app
        .call("browser.watch", json!({ "botId": id }))
        .await
        .expect("watch");

    for scale in [100u32, 125, 150, 200] {
        let room = json!({ "botId": id, "width": 1000, "height": 700, "scale": scale });
        b.app.call("browser.resize", room).await.expect("resize");
        let opening = call(
            &b.mcp,
            "browser_open",
            json!({ "url": format!("{}/size", b.site) }),
        );
        if scale == 100 {
            answer_site(&mut b.app, true, None).await;
        }
        let page = opening.await.expect("task").expect("page");
        // The page keeps its size and its pixels, whatever the screen asks.
        assert!(
            page.contains("Window: 1000 x 700, pixels 1"),
            "{scale}%: {page}"
        );
        let frame = frame_of(&mut b.app, 700).await;
        assert_eq!(
            jpeg_size(frame["data"].as_str().expect("picture")),
            (1000, 700)
        );

        // The bot's click lands on the middle of the button, (350, 220).
        let page = b
            .mcp
            .tool_text(
                "browser_open",
                json!({ "url": format!("{}/where", b.site) }),
            )
            .await
            .expect("where");
        let add = ref_of(&page, "button \"Add\"");
        let clicked = b
            .mcp
            .tool_text("browser_click", json!({ "ref": add }))
            .await
            .expect("click");
        assert!(clicked.contains("Click 350,220"), "{scale}%: {clicked}");

        // So does the owner's, at the point they aimed.
        b.app
            .call("browser.take", json!({ "botId": id }))
            .await
            .expect("take");
        common::browsing::click(&mut b.app, &b.bot, 360.0, 230.0).await;
        b.app
            .call("browser.release", json!({ "botId": id }))
            .await
            .expect("release");
        let seen = b
            .mcp
            .tool_text("browser_look", json!({}))
            .await
            .expect("look");
        assert!(seen.contains("Click 360,230"), "{scale}%: {seen}");
    }

    // The bot's own picture is the size of the page, scrolled where the page is.
    b.mcp
        .tool_text(
            "browser_open",
            json!({ "url": format!("{}/tall#end", b.site) }),
        )
        .await
        .expect("tall page");
    let picture = b.mcp.tool_result("browser_screenshot", json!({})).await;
    let data = picture["content"][0]["data"].as_str().expect("picture");
    assert_eq!(jpeg_size(data), (1000, 700));
}
