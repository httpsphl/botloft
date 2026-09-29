//! How the bot's own output and approvals move its state (spec 7.2), on a
//! FakeRuntime and a paused clock.

mod common;

use std::time::Duration;

use botloft_core::protocol::{BotState, BotsRestartParams};
use botloftd::clock::Clock as _;
use botloftd::service::bots;
use common::stream;
use common::supervised::setup;
use serde_json::json;

#[tokio::test(start_paused = true)]
async fn turns_and_approvals_drive_the_state() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let (_, generation) = s.daemon.supervisor.status(&s.bot).expect("slot");
    let generation = generation.expect("running");

    s.message("first");
    s.message("queued behind the first");
    assert_eq!(s.state(), BotState::Busy);
    assert_eq!(process.input_lines().len(), 2);

    s.daemon.supervisor.approval_opened(&s.bot, generation);
    assert_eq!(s.state(), BotState::NeedsApproval);
    s.daemon.supervisor.approval_closed(&s.bot, generation);
    assert_eq!(s.state(), BotState::Busy);

    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(s.state(), BotState::Busy, "one turn is still queued");
    process.emit(stream::result(false)).await;
    s.until(BotState::Idle).await;
    // News about a process that is gone changes nothing.
    s.daemon.supervisor.approval_opened(&s.bot, generation + 1);
    assert_eq!(s.state(), BotState::Idle);
}

/// Subagents finish after the turn that started them: Claude Code then goes
/// on by itself, with no message written for it (spec 7.2).
#[tokio::test(start_paused = true)]
async fn a_turn_claude_code_starts_on_its_own_is_busy() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;

    process.emit(stream::init("session-1")).await;
    s.until(BotState::Busy).await;
    process.emit(stream::result(false)).await;
    s.until(BotState::Idle).await;

    // A message written while it works keeps the bot busy past that turn.
    process.emit(stream::init("session-1")).await;
    s.until(BotState::Busy).await;
    s.message("and this too");
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(s.state(), BotState::Busy, "the message still has its turn");
    process.emit(stream::init("session-1")).await;
    process.emit(stream::result(false)).await;
    s.until(BotState::Idle).await;
}

#[tokio::test(start_paused = true)]
async fn subagents_in_the_background_keep_the_bot_busy() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("research this");

    let tasks = |kinds: &[&str]| {
        let tasks: Vec<_> = kinds
            .iter()
            .enumerate()
            .map(|(n, kind)| json!({ "task_id": format!("t{n}"), "task_type": kind }))
            .collect();
        json!({ "type": "system", "subtype": "background_tasks_changed", "tasks": tasks })
    };
    process.emit(tasks(&["local_agent", "local_agent"])).await;
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(s.state(), BotState::Busy, "the agents are still working");

    process.emit(tasks(&["local_agent"])).await;
    tokio::time::sleep(Duration::from_millis(20)).await;
    assert_eq!(s.state(), BotState::Busy);
    // A shell command left running is not work.
    process.emit(tasks(&["local_bash"])).await;
    s.until(BotState::Idle).await;
}

/// What keeps the computer awake (spec 14): only `busy` counts.
#[tokio::test(start_paused = true)]
async fn the_busy_count_follows_turns_approvals_and_crashes() {
    let s = setup().await;
    let busy = s.daemon.supervisor.busy_bots();
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    assert_eq!(*busy.borrow(), 0);
    let generation = s.message("work");
    assert_eq!(*busy.borrow(), 1);

    s.daemon.supervisor.approval_opened(&s.bot, generation);
    assert_eq!(*busy.borrow(), 0, "waiting for the owner is not working");
    s.daemon.supervisor.approval_closed(&s.bot, generation);
    assert_eq!(*busy.borrow(), 1);
    process.emit(stream::result(false)).await;
    s.until(BotState::Idle).await;
    assert_eq!(*busy.borrow(), 0);

    s.message("again");
    assert_eq!(*busy.borrow(), 1);
    process.exit(1).await;
    s.until(BotState::Backoff).await;
    assert_eq!(*busy.borrow(), 0, "a dead process works no more");
    s.runtime.process(2).await;
    s.until(BotState::Idle).await;
    assert_eq!(*busy.borrow(), 0);
}

#[tokio::test(start_paused = true)]
async fn sign_in_errors_stop_the_bot_until_the_owner_restarts_it() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    process
        .emit(stream::api_error(
            "authentication_failed",
            "Invalid API key",
        ))
        .await;
    s.until(BotState::AuthError).await;
    assert!(process.killed(), "restarting cannot fix a sign-in");

    tokio::time::sleep(Duration::from_secs(600)).await;
    assert_eq!(s.runtime.processes().len(), 1, "no restart loop");
    assert_eq!(s.state(), BotState::AuthError);

    let restart = BotsRestartParams {
        bot_id: s.bot.clone(),
        fresh: None,
    };
    bots::restart(&s.daemon, restart).expect("restart");
    s.runtime.process(2).await;
    s.until(BotState::Launching).await;
}

#[tokio::test(start_paused = true)]
async fn a_rate_limit_holds_the_bot_until_it_resets() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    // `resetsAt` is in seconds.
    let resets = s.clock.now_ms() / 1000 + 30;
    process
        .emit(json!({
            "type": "rate_limit_event",
            "rate_limit_info": { "status": "rejected", "resetsAt": resets, "rateLimitType": "five_hour",
                "unifiedWindows": { "five_hour": { "utilization": 1.0, "resetsAt": resets } } },
        }))
        .await;
    s.until(BotState::RateLimited).await;
    process.emit(stream::result(true)).await;
    tokio::time::sleep(Duration::from_secs(10)).await;
    assert_eq!(s.state(), BotState::RateLimited);
    let usage = s.daemon.usage().expect("usage");
    assert_eq!(
        (usage.status.as_str(), usage.resets_at),
        ("rejected", Some(resets * 1000))
    );
    assert_eq!(usage.windows[0].name, "five_hour");

    s.clock.advance(Duration::from_secs(31));
    s.until(BotState::Idle).await;
}
