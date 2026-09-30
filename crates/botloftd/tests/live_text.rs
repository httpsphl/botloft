//! Live text (spec 8.3): pieces of a reply that arrive close together reach
//! the app as one `chat.delta`, always before what the bot does next.

mod common;

use common::bots::two_bots;
use common::stream;
use serde_json::Value;

/// Several stream-json events in one chunk of stdout.
fn chunk(events: &[Value]) -> Vec<u8> {
    events
        .iter()
        .flat_map(|event| format!("{event}\n").into_bytes())
        .collect()
}

#[tokio::test]
async fn pieces_that_arrive_together_go_out_as_one() {
    let mut c = two_bots().await;
    let pieces = ["Hel", "lo, ", "world"].map(stream::delta);
    c.lead_process.output(&chunk(&pieces)).await;

    let delta = c.app.notification("chat.delta").await;
    assert_eq!(delta["text"], "Hello, world");
}

#[tokio::test]
async fn live_text_goes_out_before_the_reply_it_becomes() {
    let mut c = two_bots().await;
    let events = [
        stream::delta("All "),
        stream::delta("done."),
        stream::text("All done."),
    ];
    c.lead_process.output(&chunk(&events)).await;

    loop {
        let changed = c.app.notification("chat.item").await;
        if changed["item"]["body"]["kind"] == "reply" {
            break;
        }
    }
    let before = c.app.queued("chat.delta");
    let texts: Vec<_> = before.iter().map(|delta| delta["text"].clone()).collect();
    assert_eq!(texts, ["All done."]);
}
