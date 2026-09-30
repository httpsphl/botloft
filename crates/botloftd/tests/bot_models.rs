//! A bot's model (spec 7.4): the flag it starts with, the restart onto a new
//! model once nothing is in progress, the model Claude Code reports and a
//! model the account cannot use.

mod common;

use std::time::Duration;

use botloft_core::protocol::{BotModel, BotState, BotsSetModelParams, ChatBody, NoticeCode};
use botloftd::service::{bots, models};
use common::stream;
use common::supervised::{Setup, arg_after, setup};
use serde_json::json;

fn set_model(s: &Setup, model: BotModel) {
    models::set_model(
        &s.daemon,
        BotsSetModelParams {
            bot_id: s.bot.clone(),
            model,
        },
    )
    .expect("set model");
}

fn bot(s: &Setup) -> botloft_core::protocol::Bot {
    bots::list(&s.daemon, Default::default())
        .expect("list")
        .into_iter()
        .find(|bot| bot.id == s.bot)
        .expect("bot")
}

#[tokio::test(start_paused = true)]
async fn a_new_bot_runs_on_the_plans_default_model() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    assert!(!process.args().iter().any(|arg| arg == "--model"));
    assert_eq!(bot(&s).model, BotModel::Default);
    assert_eq!(bot(&s).model_in_use, None);
}

#[tokio::test(start_paused = true)]
async fn an_idle_bot_restarts_on_its_new_model_in_the_same_conversation() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = s.turn(&first).await;

    set_model(&s, BotModel::Haiku);
    let second = s.runtime.process(2).await;
    assert!(first.killed());
    assert_eq!(arg_after(&second, "--model").as_deref(), Some("haiku"));
    assert_eq!(arg_after(&second, "--resume"), Some(session));

    // Back to the plan's default: no flag at all.
    s.until(BotState::Idle).await;
    set_model(&s, BotModel::Default);
    let third = s.runtime.process(3).await;
    assert!(!third.args().iter().any(|arg| arg == "--model"));
}

#[tokio::test(start_paused = true)]
async fn a_busy_bot_finishes_its_turn_before_changing_model() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("work");
    set_model(&s, BotModel::Opus);
    tokio::time::sleep(Duration::from_secs(30)).await;
    assert_eq!(s.runtime.processes().len(), 1, "the turn is not cut short");
    assert!(s.daemon.supervisor.relaunch_pending(&s.bot));

    first.emit(stream::began("session-1")).await;
    first.emit(stream::result(false)).await;
    let second = s.runtime.process(2).await;
    assert_eq!(arg_after(&second, "--model").as_deref(), Some("opus"));
}

#[tokio::test(start_paused = true)]
async fn the_reported_model_is_kept_for_the_app() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = arg_after(&process, "--session-id").expect("session");
    process
        .emit(json!({
            "type": "system", "subtype": "init", "session_id": session,
            "model": "claude-opus-5-5", "permissionMode": "default",
        }))
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(bot(&s).model_in_use.as_deref(), Some("claude-opus-5-5"));
    assert_eq!(
        bot(&s).model,
        BotModel::Default,
        "the choice stays the owner's"
    );
}

#[tokio::test(start_paused = true)]
async fn a_model_the_account_cannot_use_is_told_in_the_chat() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    process
        .emit(stream::api_error(
            "model_not_found",
            "There's an issue with the selected model (fable).",
        ))
        .await;
    process.emit(stream::began("session-1")).await;
    process.emit(stream::result(true)).await;
    s.until(BotState::Idle).await;

    let items = s
        .daemon
        .store()
        .chat_history(&s.bot, None, 50)
        .expect("history");
    let notice = items
        .iter()
        .find_map(|item| match &item.body {
            ChatBody::Notice(notice) => Some(notice.clone()),
            _ => None,
        })
        .expect("a notice");
    assert_eq!(notice.code, Some(NoticeCode::ModelUnavailable));
    assert_eq!(s.runtime.processes().len(), 1, "the bot keeps running");
}
