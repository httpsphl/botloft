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
