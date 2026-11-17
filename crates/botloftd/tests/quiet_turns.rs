//! A turn that went quiet is stopped (spec 7.9): half an hour with nothing
//! printed and no request for the owner interrupts it, and a process that
//! does not answer starts again. Time on the paused tokio clock.

mod common;

use std::time::Duration;

use botloft_core::protocol::{BotState, ChatBody, NoticeCode, TurnItem};
use botloftd::runtime::fake::FakeProcess;
use common::stream;
use common::supervised::{Setup, session_of, setup};
use serde_json::json;

async fn minutes(n: u32) {
    tokio::time::sleep(Duration::from_secs(60) * n).await;
}

/// The notices that say Botloft stopped a quiet turn.
fn stuck_notices(s: &Setup) -> usize {
    s.daemon
        .store()
        .chat_history(&s.bot, None, 100)
        .expect("history")
        .into_iter()
        .filter(|item| {
            matches!(&item.body, ChatBody::Notice(notice)
                if notice.code == Some(NoticeCode::StuckTurn))
        })
        .count()
}

/// The line that closed the last turn.
fn last_turn(s: &Setup) -> TurnItem {
    s.daemon
        .store()
        .chat_history(&s.bot, None, 100)
        .expect("history")
        .into_iter()
        .rev()
        .find_map(|item| match item.body {
            ChatBody::Turn(turn) => Some(turn),
            _ => None,
        })
        .expect("a turn")
}

/// How many times the daemon asked the process to stop its turn; it also
/// asks Claude Code about its settings and context on its own.
fn interrupts(process: &FakeProcess) -> usize {
    process
        .control_requests()
        .iter()
        .filter(|subtype| *subtype == "interrupt")
        .count()
}

/// The bot in a turn: it began a message and printed nothing since.
async fn in_turn(s: &Setup) -> FakeProcess {
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    process.emit(stream::began("session-1")).await;
    process
}

#[tokio::test(start_paused = true)]
async fn a_turn_quiet_for_half_an_hour_is_stopped() {
    let s = setup().await;
    let process = in_turn(&s).await;
    minutes(29).await;
    assert_eq!(interrupts(&process), 0);

    minutes(2).await;
    assert_eq!(interrupts(&process), 1);
    assert_eq!(stuck_notices(&s), 1);
    // Claude Code ends the turn as cut, so the process goes on, and the
    // line says Botloft stopped it.
    process
        .emit(
            json!({ "type": "result", "subtype": "error_during_execution",
            "is_error": true, "duration_ms": 1_860_000, "terminal_reason": "aborted_streaming" }),
        )
        .await;
    s.until(BotState::Idle).await;
    let turn = last_turn(&s);
    assert_eq!((turn.stopped, turn.quiet, turn.error), (true, true, None));
    minutes(5).await;
    assert!(!process.killed());
    assert_eq!(s.runtime.processes().len(), 1);
    assert_eq!(interrupts(&process), 1);
}

#[tokio::test(start_paused = true)]
async fn a_process_that_does_not_answer_starts_again() {
    let s = setup().await;
    let process = in_turn(&s).await;
    minutes(31).await;
    assert_eq!(interrupts(&process), 1);
    assert!(!process.killed());

    minutes(2).await;
    assert!(process.killed());
    // It comes back in the same conversation.
    let again = s.runtime.process(2).await;
    assert_eq!(session_of(&again), "session-1");
    assert_eq!(stuck_notices(&s), 1);
}

#[tokio::test(start_paused = true)]
async fn waiting_for_the_owner_or_printing_is_not_quiet() {
    let s = setup().await;
    let process = in_turn(&s).await;
    let generation = s
        .daemon
        .supervisor
        .status(&s.bot)
        .and_then(|(_, generation)| generation)
        .expect("generation");
    s.daemon.supervisor.approval_opened(&s.bot, generation);
    minutes(45).await;
    // The owner answers: the half hour starts from there.
    s.daemon.supervisor.approval_closed(&s.bot, generation);
    minutes(20).await;
    for _ in 0..3 {
        process.emit(stream::text("Still on it.")).await;
        minutes(20).await;
    }
    assert_eq!(interrupts(&process), 0);
    assert_eq!(stuck_notices(&s), 0);
}

#[tokio::test(start_paused = true)]
async fn time_the_computer_slept_does_not_count() {
    let s = setup().await;
    let process = in_turn(&s).await;
    minutes(20).await;
    // Asleep for two hours: no pass runs meanwhile.
    tokio::time::advance(Duration::from_secs(2 * 3600)).await;
    minutes(1).await;
    assert_eq!(interrupts(&process), 0);

    minutes(30).await;
    assert_eq!(interrupts(&process), 1);
}
