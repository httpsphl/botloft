//! Claude Code's sign-in (spec 7.3): checked with `claude auth status`,
//! and bots stopped by a sign-in error come back once the owner signs in.

mod common;

use std::time::Duration;

use botloft_core::protocol::BotState;
use botloftd::service;
use common::stream;
use common::supervised::{Setup, setup};

async fn sign_in_error(s: &Setup) {
    let process = s.runtime.process(s.runtime.processes().len()).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    process
        .emit(stream::api_error(
            "authentication_failed",
            "Invalid API key",
        ))
        .await;
    s.until(BotState::AuthError).await;
}

#[tokio::test(start_paused = true)]
async fn the_status_says_whether_claude_code_is_signed_in() {
    let s = setup().await;
    s.until(BotState::Launching).await;
    let status = service::status(&s.daemon).expect("status");
    assert_eq!(status.claude_signed_in, Some(true));
    assert!(status.claude_path.is_some());
    assert_eq!(s.runtime.sign_in_checks(), 1, "checked once, then trusted");
    tokio::time::sleep(Duration::from_secs(300)).await;
    assert_eq!(s.runtime.sign_in_checks(), 1);
}

#[tokio::test(start_paused = true)]
async fn signing_in_again_brings_the_stopped_bots_back() {
    let s = setup().await;
    s.runtime.process(1).await;
    s.runtime.set_signed_in(false);
    sign_in_error(&s).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(s.daemon.supervisor.claude_signed_in(), Some(false));

    // Still signed out: checked every 30 s, the bot stays stopped.
    tokio::time::sleep(Duration::from_secs(40)).await;
    assert_eq!(s.runtime.processes().len(), 1);
    assert!(s.runtime.sign_in_checks() >= 3);

    s.runtime.set_signed_in(true);
    tokio::time::sleep(Duration::from_secs(31)).await;
    s.runtime.process(2).await;
    assert_eq!(s.daemon.supervisor.claude_signed_in(), Some(true));
}

#[tokio::test(start_paused = true)]
async fn the_app_can_ask_for_a_check_right_after_signing_in() {
    let s = setup().await;
    s.runtime.process(1).await;
    s.runtime.set_signed_in(false);
    sign_in_error(&s).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    s.runtime.set_signed_in(true);
    s.daemon.supervisor.refresh_claude();
    tokio::time::timeout(Duration::from_secs(1), s.runtime.process(2))
        .await
        .expect("restarted without waiting for the next check");
}

/// Billing or a blocked organization also stop the bot, but Claude Code is
/// still signed in: nothing changes by itself, so nothing restarts in a loop.
#[tokio::test(start_paused = true)]
async fn an_account_problem_does_not_restart_in_a_loop() {
    let s = setup().await;
    s.runtime.process(1).await;
    sign_in_error(&s).await;
    tokio::time::sleep(Duration::from_secs(600)).await;
    assert_eq!(s.runtime.processes().len(), 1);
    assert_eq!(s.state(), BotState::AuthError);
    assert_eq!(s.daemon.supervisor.claude_signed_in(), Some(true));
}
