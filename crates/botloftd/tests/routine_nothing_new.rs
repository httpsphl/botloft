//! A routine run with nothing new to report (spec 20.5): the bot calls
//! `nothing_new` in the run's turn, and the run ends done and marked, with
//! no report for the owner to read.

mod common;

use std::time::Duration;

use botloft_core::protocol::{ChatBody, MessagesSendParams, Missed, Overlap, RunStatus};
use botloftd::routines;
use botloftd::seat::Seat;
use botloftd::service::messages;
use common::bots::ready_bot;
use common::routines::*;
use common::{TestDaemon, stream};
use serde_json::json;

#[tokio::test(start_paused = true)]
async fn the_run_of_the_turn_takes_the_mark_and_keeps_it_when_done() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    // The message waits: no turn carries the run yet.
    assert!(!routines::mark_nothing_new(
        &s.daemon,
        &botloftd::seat::Seat::main(&s.bot)
    ));

    let process = s.runtime.process(1).await;
    process.emit(stream::replay(&lines[0])).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(routines::mark_nothing_new(
        &s.daemon,
        &botloftd::seat::Seat::main(&s.bot)
    ));
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let seen = runs(&s, &routine.id);
    assert_eq!(seen[0].status, RunStatus::Done);
    assert!(seen[0].nothing_new, "it ended quietly");
    // Once the turn is over, nothing is left to mark.
    assert!(!routines::mark_nothing_new(
        &s.daemon,
        &botloftd::seat::Seat::main(&s.bot)
    ));

    // The next run starts without the mark and keeps none if the bot reports.
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 2).await;
    finish_turn(&s, &lines[1], false).await;
    let seen = runs(&s, &routine.id);
    assert_eq!(seen[1].status, RunStatus::Done);
    assert!(!seen[1].nothing_new);
}

/// The texts of the bot's replies in its chat.
fn replies(s: &common::supervised::Setup) -> Vec<String> {
    s.daemon
        .store()
        .chat_history(&s.bot, None, 50)
        .expect("chat")
        .into_iter()
        .filter_map(|item| match item.body {
            ChatBody::Reply(reply) => Some(reply.text),
            _ => None,
        })
        .collect()
}

/// A model may still write "nothing new" after calling the tool: the run
/// ends quietly anyway, with no reply to mark the bot unread.
#[tokio::test(start_paused = true)]
async fn a_reply_written_after_nothing_new_is_left_out() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    let process = s.runtime.process(1).await;
    process.emit(stream::replay(&lines[0])).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(routines::mark_nothing_new(&s.daemon, &Seat::main(&s.bot)));
    process
        .emit(stream::text("Nothing new in the inbox."))
        .await;
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(replies(&s).is_empty(), "{:?}", replies(&s));
    assert!(runs(&s, &routine.id)[0].nothing_new);

    // The next turn writes its replies as usual.
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 2).await;
    process.emit(stream::replay(&lines[1])).await;
    process.emit(stream::text("Two new files.")).await;
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(replies(&s), ["Two new files."]);
}

/// The owner wrote while the run's turn went on: the reply is for them.
#[tokio::test(start_paused = true)]
async fn a_reply_stays_when_the_owner_wrote_in_the_same_turn() {
    let s = routines_setup().await;
    create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    let process = s.runtime.process(1).await;
    process.emit(stream::replay(&lines[0])).await;
    messages::send(
        &s.daemon,
        MessagesSendParams {
            bot_id: s.bot.clone(),
            body: "and how are you?".into(),
            attachments: None,
            reply_to: None,
            conversation_id: None,
        },
    )
    .expect("send");
    s.clock.advance(Duration::from_secs(5));
    let lines = written(&s, 2).await;
    process.emit(stream::replay(&lines[1])).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(routines::mark_nothing_new(&s.daemon, &Seat::main(&s.bot)));
    process.emit(stream::text("All good, thanks.")).await;
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(replies(&s), ["All good, thanks."]);
}

#[tokio::test]
async fn outside_a_routine_the_tool_says_so() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Office" }))
        .await
        .expect("crew");
    let (_bot, _process, mut mcp) = ready_bot(&t, &mut app, &crew, "Mail").await;
    let refused = mcp
        .tool("nothing_new", json!({}))
        .await
        .expect_err("no routine run");
    assert!(refused.contains("No routine run is open"), "{refused}");
}
