//! Which conversation a bot's next process resumes (spec 7.3), on a
//! FakeRuntime: Claude Code has nothing on disk for a conversation before
//! its first turn began, and may have lost one it had.

mod common;

use std::time::Duration;

use botloft_core::protocol::{
    BotState, BotsRestartParams, BotsSetPermissionModeParams, ChatBody, PermissionMode,
};
use botloftd::service::{bots, modes};
use common::stream;
use common::supervised::{Setup, arg_after, session_of, setup};

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

/// The conversation a daemon that starts now would resume.
fn stored(s: &Setup) -> Option<String> {
    s.daemon.store().session_id(&s.bot).expect("read")
}

#[tokio::test(start_paused = true)]
async fn a_bot_restarted_before_its_first_turn_starts_a_new_conversation() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    // The owner picks a mode right after creating the bot.
    set_mode(&s, PermissionMode::AcceptEdits);

    let second = s.runtime.process(2).await;
    assert!(first.killed());
    assert_eq!(
        arg_after(&second, "--permission-mode").as_deref(),
        Some("acceptEdits")
    );
    assert_eq!(
        arg_after(&second, "--resume"),
        None,
        "Claude Code saved nothing to resume"
    );
    assert!(arg_after(&second, "--session-id").is_some());
}

#[tokio::test(start_paused = true)]
async fn a_conversation_is_resumed_once_a_turn_began_in_it_not_before() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    // Gone between `system/init` and the replay of the message: with the
    // real Claude Code, that conversation cannot be resumed.
    s.message("hi");
    first.emit(stream::init(&session_of(&first))).await;
    first.exit(1).await;

    let second = s.runtime.process(2).await;
    assert_eq!(arg_after(&second, "--resume"), None);
    assert_eq!(stored(&s), None);

    s.until(BotState::Idle).await;
    let session = session_of(&second);
    s.message("hi");
    second.emit(stream::init(&session)).await;
    second.emit(stream::began(&session)).await;
    second.exit(1).await;

    let third = s.runtime.process(3).await;
    assert_eq!(arg_after(&third, "--resume"), Some(session.clone()));
    assert_eq!(stored(&s), Some(session));
}

#[tokio::test(start_paused = true)]
async fn a_conversation_claude_code_lost_is_replaced_without_a_failed_turn() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = s.turn(&first).await;
    first.exit(0).await;

    let resumed = s.runtime.process(2).await;
    assert_eq!(arg_after(&resumed, "--resume"), Some(session.clone()));
    resumed.emit(stream::no_conversation(&session)).await;
    // Slower to exit than `fresh_start_if_dies_within`.
    tokio::time::sleep(Duration::from_secs(20)).await;
    resumed.exit(1).await;

    let fresh = s.runtime.process(3).await;
    assert_eq!(arg_after(&fresh, "--resume"), None);
    assert_ne!(session_of(&fresh), session);
    s.until(BotState::Idle).await;
    assert_eq!(stored(&s), None, "a daemon restart would not try it again");

    let history = s
        .daemon
        .store()
        .chat_history(&s.bot, None, 50)
        .expect("history");
    let turns: Vec<_> = history
        .iter()
        .filter_map(|item| match &item.body {
            ChatBody::Turn(turn) => Some(turn.error.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(turns, [None], "only the turn the bot really had");
}

#[tokio::test(start_paused = true)]
async fn a_fresh_restart_leaves_the_old_conversation_for_good() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = s.turn(&first).await;
    assert_eq!(stored(&s), Some(session.clone()));

    let restart = BotsRestartParams {
        bot_id: s.bot.clone(),
        fresh: Some(true),
    };
    bots::restart(&s.daemon, restart).expect("restart");
    let second = s.runtime.process(2).await;
    assert_ne!(session_of(&second), session);
    s.until(BotState::Idle).await;
    assert_eq!(stored(&s), None, "a daemon restart would not resume it");

    // Restarted again before the new conversation had a turn.
    set_mode(&s, PermissionMode::Plan);
    let third = s.runtime.process(3).await;
    assert_eq!(arg_after(&third, "--resume"), None);
}
