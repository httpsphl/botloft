//! A turn the account's usage limit cut short goes on by itself once the
//! limit resets (spec 7.3): Botloft's note waits in the bot's queue while
//! the limit lasts, and the courier sends it when the bot is back.

mod common;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::protocol::{BotState, DeliveryState, SenderKind};
use botloft_store::MessageFilter;
use botloftd::clock::Clock as _;
use botloftd::courier;
use botloftd::service::resume;
use common::bots::text_of;
use common::stream;
use common::supervised::{Setup, setup};
use serde_json::{Value, json};

fn rejected(resets_at_s: i64) -> Value {
    json!({
        "type": "rate_limit_event",
        "rate_limit_info": { "status": "rejected", "resetsAt": resets_at_s,
            "rateLimitType": "five_hour" },
    })
}

/// Botloft's notes to the bot, oldest first.
fn notes(s: &Setup) -> Vec<String> {
    let store = s.daemon.store();
    let filter = MessageFilter {
        bot: Some(&s.bot),
        limit: 50,
        ..MessageFilter::default()
    };
    let mut notes: Vec<String> = store
        .messages(filter)
        .expect("messages")
        .into_iter()
        .filter(|message| message.from_kind == SenderKind::System)
        .map(|message| message.body)
        .collect();
    notes.reverse();
    notes
}

#[tokio::test(start_paused = true)]
async fn a_turn_the_limit_cut_goes_on_once_it_resets() {
    let s = setup().await;
    tokio::spawn(courier::run(Arc::clone(&s.daemon)));
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    let resets = s.clock.now_ms() / 1000 + 30;
    process.emit(rejected(resets)).await;
    process.emit(stream::began("session-1")).await;
    process
        .emit(stream::api_error(
            "rate_limit",
            "Claude usage limit reached",
        ))
        .await;
    process.emit(stream::result(true)).await;
    s.until(BotState::RateLimited).await;
    // A message already in Claude Code's queue starts and is cut too: one
    // note is enough.
    process
        .emit(stream::api_error(
            "rate_limit",
            "Claude usage limit reached",
        ))
        .await;
    process.emit(stream::result(true)).await;
    assert_eq!(notes(&s), [resume::NOTE]);

    // It waits while the limit lasts.
    tokio::time::sleep(Duration::from_secs(10)).await;
    assert_eq!(s.state(), BotState::RateLimited);
    assert_eq!(process.input_lines().len(), 1, "only the owner's message");
    let waiting = s
        .daemon
        .store()
        .deliveries(Some(DeliveryState::Pending), Some(&s.bot))
        .expect("deliveries");
    assert_eq!(waiting.len(), 1);

    // Back at work, the bot gets the note at once.
    s.clock.advance(Duration::from_secs(31));
    s.until(BotState::Busy).await;
    let lines = process.wait_lines(2).await;
    let text = text_of(&lines[1]);
    assert!(
        text.starts_with("[botloft] from Botloft · crew Ops\n\n"),
        "{text}"
    );
    assert!(text.ends_with(resume::NOTE), "{text}");
}

#[tokio::test(start_paused = true)]
async fn a_bot_cut_again_after_reading_the_note_gets_a_new_one() {
    let s = setup().await;
    tokio::spawn(courier::run(Arc::clone(&s.daemon)));
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    process.emit(rejected(s.clock.now_ms() / 1000 + 30)).await;
    process.emit(stream::began("session-1")).await;
    process.emit(stream::api_error("rate_limit", "limit")).await;
    process.emit(stream::result(true)).await;
    s.until(BotState::RateLimited).await;
    s.clock.advance(Duration::from_secs(31));
    s.until(BotState::Busy).await;
    let note = process.wait_lines(2).await.remove(1);

    // The bot begins the note, and the next window cuts that turn too.
    process.emit(stream::replay(&note)).await;
    process.emit(rejected(s.clock.now_ms() / 1000 + 60)).await;
    process.emit(stream::api_error("rate_limit", "limit")).await;
    process.emit(stream::result(true)).await;
    s.until(BotState::RateLimited).await;
    assert_eq!(notes(&s), [resume::NOTE, resume::NOTE]);
}

/// A reset time that already passed would let the bot try again at once:
/// the limit lasts 5 minutes instead.
#[tokio::test(start_paused = true)]
async fn a_reset_time_already_past_waits_five_minutes() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    process.emit(rejected(s.clock.now_ms() / 1000 - 10)).await;
    s.until(BotState::RateLimited).await;
    process.emit(stream::began("session-1")).await;
    process.emit(stream::result(true)).await;
    tokio::time::sleep(Duration::from_secs(10)).await;
    assert_eq!(s.state(), BotState::RateLimited);

    s.clock.advance(Duration::from_secs(5 * 60));
    s.until(BotState::Idle).await;
}
