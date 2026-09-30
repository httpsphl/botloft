//! A bot's permission mode (spec 7.4): the flag it starts with, the restart
//! into a new mode once nothing is in progress, and leaving plan mode.

mod common;

use std::time::Duration;

use botloft_core::protocol::{BotState, BotsSetPermissionModeParams, PermissionMode};
use botloftd::runtime::fake::FakeProcess;
use botloftd::service::modes;
use common::stream;
use common::supervised::{Setup, session_of, setup};
use serde_json::json;

fn arg_after(process: &FakeProcess, flag: &str) -> Option<String> {
    let args = process.args();
    let at = args.iter().position(|arg| arg == flag)?;
    args.get(at + 1).cloned()
}

fn set_mode(s: &Setup, mode: PermissionMode) {
    modes::set_permission_mode(
        &s.daemon,
        BotsSetPermissionModeParams {
            bot_id: s.bot.clone(),
            mode,
        },
    )
    .expect("set mode");
}

fn stored_mode(s: &Setup) -> PermissionMode {
    s.daemon
        .store()
        .bot(&s.bot)
        .expect("read")
        .expect("bot")
        .permission_mode
}

fn init_with_mode(session: &str, mode: &str) -> serde_json::Value {
    json!({ "type": "system", "subtype": "init", "session_id": session, "permissionMode": mode })
}

#[tokio::test(start_paused = true)]
async fn a_new_bot_asks_before_everything() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    assert_eq!(
        arg_after(&process, "--permission-mode").as_deref(),
        Some("default")
    );
    assert_eq!(stored_mode(&s), PermissionMode::Default);
}

#[tokio::test(start_paused = true)]
async fn an_idle_bot_restarts_into_its_new_mode_in_the_same_conversation() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = s.turn(&first).await;

    set_mode(&s, PermissionMode::AcceptEdits);
    let second = s.runtime.process(2).await;
    assert!(first.killed());
    assert_eq!(
        arg_after(&second, "--permission-mode").as_deref(),
        Some("acceptEdits")
    );
    assert_eq!(arg_after(&second, "--resume"), Some(session));
}

#[tokio::test(start_paused = true)]
async fn a_busy_bot_finishes_its_turn_before_changing_mode() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("work");
    set_mode(&s, PermissionMode::Auto);
    tokio::time::sleep(Duration::from_secs(30)).await;
    assert_eq!(s.runtime.processes().len(), 1, "the turn is not cut short");
    assert!(s.daemon.supervisor.relaunch_pending(&s.bot));

    first.emit(stream::began("session-1")).await;
    first.emit(stream::result(false)).await;
    let second = s.runtime.process(2).await;
    assert_eq!(
        arg_after(&second, "--permission-mode").as_deref(),
        Some("auto")
    );
    assert!(!s.daemon.supervisor.relaunch_pending(&s.bot));
}

#[tokio::test(start_paused = true)]
async fn approving_a_plan_moves_the_bot_out_of_plan_mode() {
    let s = setup().await;
    s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    set_mode(&s, PermissionMode::Plan);
    let planning = s.runtime.process(2).await;
    s.until(BotState::Idle).await;
    let session = session_of(&planning);

    // Claude Code left plan mode when the plan was approved.
    planning.emit(init_with_mode(&session, "default")).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(stored_mode(&s), PermissionMode::Default);
    assert_eq!(
        s.runtime.processes().len(),
        2,
        "no restart: it already changed"
    );
}

#[tokio::test(start_paused = true)]
async fn a_turn_of_the_old_process_does_not_undo_the_owners_choice() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = arg_after(&first, "--session-id").expect("session");
    s.message("queued");
    set_mode(&s, PermissionMode::AcceptEdits);

    first.emit(init_with_mode(&session, "default")).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(stored_mode(&s), PermissionMode::AcceptEdits);
    first.emit(stream::init(&session)).await;
    assert_eq!(stored_mode(&s), PermissionMode::AcceptEdits);
}

#[tokio::test(start_paused = true)]
async fn a_plan_approved_while_a_new_model_waits_still_leaves_plan_mode() {
    let s = setup().await;
    s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    set_mode(&s, PermissionMode::Plan);
    let planning = s.runtime.process(2).await;
    s.until(BotState::Idle).await;
    let session = session_of(&planning);
    s.message("plan it");
    botloftd::service::models::set_model(
        &s.daemon,
        botloft_core::protocol::BotsSetModelParams {
            bot_id: s.bot.clone(),
            model: botloft_core::protocol::BotModel::Sonnet,
        },
    )
    .expect("set model");

    planning.emit(init_with_mode(&session, "default")).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(stored_mode(&s), PermissionMode::Default);
    planning.emit(stream::began("session-1")).await;
    planning.emit(stream::result(false)).await;
    let next = s.runtime.process(3).await;
    assert_eq!(
        arg_after(&next, "--permission-mode").as_deref(),
        Some("default")
    );
}
