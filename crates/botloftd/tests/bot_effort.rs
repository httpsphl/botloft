//! A bot's effort (spec 7.4): the flag it starts with, the restart onto a
//! new level once nothing is in progress, and what Claude Code reports about
//! the level the bot's model uses by itself.

mod common;

use std::time::Duration;

use botloft_core::protocol::{
    Bot, BotEffort, BotModel, BotState, BotsSetEffortParams, BotsSetModelParams, ModelEffort,
};
use botloftd::runtime::fake::FakeProcess;
use botloftd::service::{bots, models};
use common::stream;
use common::supervised::{Setup, arg_after, setup};
use serde_json::{Value, json};

fn set_effort(s: &Setup, effort: BotEffort) {
    let params = BotsSetEffortParams {
        bot_id: s.bot.clone(),
        effort,
    };
    models::set_effort(&s.daemon, params).expect("set effort");
}

fn bot(s: &Setup) -> Bot {
    bots::list(&s.daemon, Default::default())
        .expect("list")
        .into_iter()
        .find(|bot| bot.id == s.bot)
        .expect("bot")
}

/// Claude Code answering `get_settings` for a session on `model`.
async fn applies(process: &FakeProcess, model: &str, effort: Value) {
    process
        .answer_control(
            "get_settings",
            json!({ "effective": {}, "applied": { "model": model, "effort": effort } }),
        )
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[tokio::test(start_paused = true)]
async fn a_new_bot_runs_at_the_effort_of_its_model() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    assert!(!process.args().iter().any(|arg| arg == "--effort"));
    assert_eq!(bot(&s).effort, BotEffort::Default);
    assert_eq!(bot(&s).effort_default, None);
}

#[tokio::test(start_paused = true)]
async fn an_idle_bot_restarts_on_its_new_effort_in_the_same_conversation() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = s.turn(&first).await;

    set_effort(&s, BotEffort::Low);
    let second = s.runtime.process(2).await;
    assert!(first.killed());
    assert_eq!(arg_after(&second, "--effort").as_deref(), Some("low"));
    assert_eq!(arg_after(&second, "--resume"), Some(session));
    assert_eq!(bot(&s).effort, BotEffort::Low);

    // Back to the model's own level: no flag at all.
    s.until(BotState::Idle).await;
    set_effort(&s, BotEffort::Default);
    let third = s.runtime.process(3).await;
    assert!(!third.args().iter().any(|arg| arg == "--effort"));
}

#[tokio::test(start_paused = true)]
async fn a_busy_bot_finishes_its_turn_before_changing_effort() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("work");
    set_effort(&s, BotEffort::Max);
    tokio::time::sleep(Duration::from_secs(30)).await;
    assert_eq!(s.runtime.processes().len(), 1, "the turn is not cut short");
    assert!(s.daemon.supervisor.relaunch_pending(&s.bot));

    first.emit(stream::began("session-1")).await;
    first.emit(stream::result(false)).await;
    let second = s.runtime.process(2).await;
    assert_eq!(arg_after(&second, "--effort").as_deref(), Some("max"));
}

#[tokio::test(start_paused = true)]
async fn a_new_process_tells_its_model_and_the_effort_the_model_uses() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    assert!(
        process
            .control_requests()
            .contains(&"get_settings".to_owned())
    );
    applies(&process, "claude-sonnet-5-5", json!("medium")).await;
    let seen = bot(&s);
    assert_eq!(
        seen.model_in_use.as_deref(),
        Some("claude-sonnet-5-5"),
        "known before the first turn"
    );
    assert_eq!(seen.effort_default, Some(ModelEffort::Medium));
    assert_eq!(
        seen.effort,
        BotEffort::Default,
        "the choice stays the owner's"
    );
}

#[tokio::test(start_paused = true)]
async fn the_owners_level_is_not_taken_for_the_models_own() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    applies(&first, "claude-sonnet-5-5", json!("medium")).await;
    s.until(BotState::Idle).await;

    set_effort(&s, BotEffort::High);
    let second = s.runtime.process(2).await;
    applies(&second, "claude-sonnet-5-5", json!("high")).await;
    assert_eq!(bot(&s).effort_default, Some(ModelEffort::Medium));
}

#[tokio::test(start_paused = true)]
async fn a_model_without_effort_levels_is_told_and_a_new_model_is_asked_again() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    set_effort(&s, BotEffort::Low);
    let second = s.runtime.process(2).await;
    assert!(first.killed());
    // Haiku applies no level, whatever the flag says.
    applies(&second, "claude-haiku-4-5-20251001", Value::Null).await;
    assert_eq!(bot(&s).effort_default, Some(ModelEffort::None));
    // The same name later answers with a level: it takes one after all,
    // and which one it would pick is not known.
    applies(&second, "claude-haiku-5", json!("low")).await;
    assert_eq!(bot(&s).effort_default, None);

    s.until(BotState::Idle).await;
    let params = BotsSetModelParams {
        bot_id: s.bot.clone(),
        model: BotModel::Opus,
    };
    models::set_model(&s.daemon, params).expect("set model");
    assert_eq!(bot(&s).effort_default, None, "another model, another level");
    let third = s.runtime.process(3).await;
    assert_eq!(arg_after(&third, "--effort").as_deref(), Some("low"));
    // It runs at the owner's level: the model's own stays unknown.
    applies(&third, "claude-opus-5-5", json!("low")).await;
    assert_eq!(bot(&s).effort_default, None);
}

#[tokio::test(start_paused = true)]
async fn an_answer_without_the_effort_changes_nothing() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    applies(&process, "claude-opus-5-5", json!("medium")).await;
    // An older Claude Code, or a level this build does not know.
    process
        .answer_control(
            "get_settings",
            json!({ "applied": { "model": "claude-opus-5-5" } }),
        )
        .await;
    applies(&process, "claude-opus-5-5", json!("ultra")).await;
    assert_eq!(bot(&s).effort_default, Some(ModelEffort::Medium));
}
