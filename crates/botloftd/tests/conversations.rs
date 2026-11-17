//! Several conversations of one bot (spec 40): each its own process, with
//! its own session, running at the same time, and at most three awake.

mod common;

use std::time::Duration;

use botloft_core::ids::ConversationId;
use botloft_core::protocol::BotState;
use common::TestDaemon;
use common::conversations::{
    holding, nth_process, open, replies, resumes, send, sleeping_bot, turn, until_reply,
};

const NAP: Duration = Duration::from_millis(300);

#[tokio::test(flavor = "multi_thread")]
async fn two_conversations_run_at_once_each_with_its_own_session() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(NAP));
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let other = open(&mut app, &id).await;

    send(&mut app, &bot, "Hello main", None).await;
    send(&mut app, &bot, "Hello other", Some(&other)).await;
    let (main_process, main_line) = holding(&t, 2, "Hello main").await;
    let (other_process, other_line) = holding(&t, 2, "Hello other").await;
    // Both are up together, and neither resumes anything yet.
    assert!(!main_process.killed() && !other_process.killed());
    assert_eq!(resumes(&main_process), None);
    assert_eq!(resumes(&other_process), None);

    turn(&other_process, &other_line, "session-other", "Other here").await;
    turn(&main_process, &main_line, "session-main", "Main here").await;

    // Each reply is in its own conversation's chat, and only there.
    until_reply(&mut app, &bot, None, "Main here").await;
    until_reply(&mut app, &bot, Some(&other), "Other here").await;
    assert_eq!(replies(&mut app, &bot, None).await, ["Main here"]);
    assert_eq!(replies(&mut app, &bot, Some(&other)).await, ["Other here"]);

    // Each one keeps its session, and resumes it once asleep.
    let main = ConversationId::main_of(&id);
    let store_sessions = || {
        let store = t.daemon.store();
        (
            store.conversation_session(&main).expect("main"),
            store.conversation_session(&other).expect("other"),
        )
    };
    assert_eq!(
        store_sessions(),
        (Some("session-main".into()), Some("session-other".into()))
    );
    // The bot sleeps once all of its conversations do.
    t.until_state(&bot, BotState::Sleeping).await;
    assert!(main_process.killed() && other_process.killed());
    send(&mut app, &bot, "Again", Some(&other)).await;
    let again = nth_process(&t, 2).await;
    assert_eq!(resumes(&again).as_deref(), Some("session-other"));
    let line = again.wait_lines(1).await.remove(0);
    assert!(line.to_string().contains("Again"), "{line}");
}

#[tokio::test(flavor = "multi_thread")]
async fn at_most_three_conversations_of_a_bot_are_awake() {
    let t = TestDaemon::start_supervised().await;
    t.daemon
        .settings
        .set_sleep_after(Some(Duration::from_secs(600)));
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let mut others: Vec<ConversationId> = Vec::new();
    for _ in 0..3 {
        others.push(open(&mut app, &id).await);
    }

    send(&mut app, &bot, "To main", None).await;
    for (n, other) in others.iter().enumerate() {
        send(&mut app, &bot, &format!("To other {n}"), Some(other)).await;
    }
    // Three start; the fourth waits while all three work.
    let _ = nth_process(&t, 2).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(t.runtime.processes().len(), 3);

    // One finishes: it goes to sleep to make room, and the last one starts.
    let first = nth_process(&t, 0).await;
    let line = first.wait_lines(1).await.remove(0);
    turn(&first, &line, "session-first", "Done").await;
    let fourth = nth_process(&t, 3).await;
    assert!(first.killed(), "the idle one made room");
    // It holds the message none of the first three got.
    let mut first_three = Vec::new();
    for n in 0..3 {
        let line = nth_process(&t, n).await.wait_lines(1).await.remove(0);
        first_three.push(line["message"]["content"][0]["text"].clone());
    }
    let waited = fourth.wait_lines(1).await.remove(0);
    let waited = &waited["message"]["content"][0]["text"];
    assert!(
        waited.as_str().is_some_and(|text| text.contains("To ")),
        "{waited}"
    );
    assert!(!first_three.contains(waited), "{waited} was already sent");
}
