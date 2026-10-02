//! Requests run off the runtime's threads (spec 11.1). An answer still
//! goes out before the notifications that follow it, except for the reads
//! the app never sets against notifications, which hold nothing back.

mod common;

use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;

use common::bots::{Crew, two_bots};
use common::stream;
use serde_json::{Value, json};

/// Holds the store for a while on another thread, so the next request
/// waits for it, then sends `request` and has the lead bot write text.
/// Returns which went out first: the answer or the text.
async fn race(c: &mut Crew, request: Value) -> Vec<&'static str> {
    // Both bots are ready and the courier has nothing to do.
    for _ in 0..2 {
        while c.app.notification("bot.state").await["state"] != "idle" {}
    }
    let (locked, is_locked) = mpsc::channel();
    let holder = std::thread::spawn({
        let daemon = Arc::clone(&c.t.daemon);
        move || {
            let _store = daemon.store();
            locked.send(()).expect("locked");
            std::thread::sleep(Duration::from_millis(1000));
        }
    });
    is_locked.recv().expect("store held");
    c.app.send_raw(&request.to_string()).await;
    // Give the daemon time to take the request off the socket, so the text
    // comes after it. Nothing shows when it has, but this is far below the
    // time the store is held.
    tokio::time::sleep(Duration::from_millis(200)).await;
    // The block that ends sends the text at once, without the timer.
    let stop =
        json!({ "type": "stream_event", "event": { "type": "content_block_stop", "index": 0 } });
    c.lead_process
        .output(format!("{}\n{stop}\n", stream::delta("Still here.")).as_bytes())
        .await;

    let mut order = Vec::new();
    while order.len() < 2 {
        let frame = c.app.recv().await.expect("open");
        if frame["id"] == request["id"] {
            assert!(frame.get("result").is_some(), "{frame}");
            order.push("answer");
        } else if frame["method"] == "chat.delta" {
            assert_eq!(frame["params"]["text"], "Still here.");
            order.push("text");
        }
    }
    holder.join().expect("holder");
    order
}

#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn a_slow_read_holds_back_no_notification() {
    let mut c = two_bots().await;
    let bots = c.app.call("bots.list", json!({})).await.expect("bots");
    let bot = bots[0]["id"].clone();
    let request =
        json!({ "jsonrpc": "2.0", "id": 900, "method": "files.list", "params": { "botId": bot } });
    assert_eq!(race(&mut c, request).await, ["text", "answer"]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn an_answer_goes_out_before_the_notifications_after_it() {
    let mut c = two_bots().await;
    let request = json!({ "jsonrpc": "2.0", "id": 901, "method": "crews.list" });
    assert_eq!(race(&mut c, request).await, ["answer", "text"]);
}

#[tokio::test]
async fn requests_are_answered_in_the_order_they_came() {
    let mut c = two_bots().await;
    for id in 1000..1010 {
        let request = json!({ "jsonrpc": "2.0", "id": id, "method": "crews.list" });
        c.app.send_raw(&request.to_string()).await;
    }
    let mut ids = Vec::new();
    while ids.len() < 10 {
        let frame = c.app.recv().await.expect("open");
        if let Some(id) = frame.get("id").and_then(Value::as_u64) {
            ids.push(id);
        }
    }
    assert_eq!(ids, (1000..1010).collect::<Vec<_>>());
}
