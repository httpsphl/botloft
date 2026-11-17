//! The owner's instructions for every agent (spec 7.10): saved in Settings,
//! they reach the agent's rules when it starts, and an agent that is
//! running starts again, in the same conversation, to read them.

mod common;

use std::time::Duration;

use botloft_core::protocol::{BotState, OwnerInstructions};
use botloft_core::validate::INSTRUCTIONS_MAX_CHARS;
use botloftd::service::{ApiError, instructions};
use common::supervised::{session_of, setup};
use serde_json::json;

fn owner(text: &str) -> OwnerInstructions {
    OwnerInstructions {
        text: text.to_owned(),
    }
}

#[tokio::test(start_paused = true)]
async fn the_owners_instructions_reach_the_agent_when_it_starts_again() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = s.turn(&first).await;
    let rules = first.spec.cwd.join(".claude/rules/owner.md");
    assert!(!rules.exists(), "nothing written yet");

    let saved = instructions::set(&s.daemon, owner("  Answer in Portuguese.\n\n")).expect("set");
    assert_eq!(saved, owner("Answer in Portuguese."));
    assert_eq!(instructions::get(&s.daemon).expect("get"), saved);

    // The agent was idle: it starts again at once, in the same conversation.
    let second = s.runtime.process(2).await;
    assert!(first.killed());
    assert_eq!(session_of(&second), session);
    let text = std::fs::read_to_string(&rules).expect("owner rules");
    assert!(text.ends_with("Answer in Portuguese.\n"), "{text}");

    // The same words again change nothing.
    s.until(BotState::Idle).await;
    instructions::set(&s.daemon, owner("Answer in Portuguese.")).expect("same");
    tokio::time::sleep(Duration::from_secs(60)).await;
    assert_eq!(s.runtime.processes().len(), 2);

    // Taken out, the file goes at the next start.
    instructions::set(&s.daemon, owner(" \n")).expect("clear");
    let third = s.runtime.process(3).await;
    assert!(second.killed());
    assert!(!third.spec.cwd.join(".claude/rules/owner.md").exists());
    assert_eq!(instructions::get(&s.daemon).expect("get"), owner(""));
}

#[tokio::test(start_paused = true)]
async fn an_agent_in_a_turn_reads_them_once_the_turn_ends() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s.message("hi");
    first.emit(common::stream::began("session-1")).await;

    instructions::set(&s.daemon, owner("Never push to main.")).expect("set");
    tokio::time::sleep(Duration::from_secs(60)).await;
    assert!(!first.killed(), "the turn goes on");
    assert_eq!(s.runtime.processes().len(), 1);

    first.emit(common::stream::result(false)).await;
    let second = s.runtime.process(2).await;
    assert_eq!(session_of(&second), "session-1");
    let text = std::fs::read_to_string(second.spec.cwd.join(".claude/rules/owner.md"))
        .expect("owner rules");
    assert!(text.ends_with("Never push to main.\n"), "{text}");
}

#[tokio::test]
async fn the_app_reads_and_saves_them() {
    let t = common::TestDaemon::start().await;
    let mut app = t.session().await;
    let none = app
        .call("instructions.get", json!(null))
        .await
        .expect("get");
    assert_eq!(none, json!({ "text": "" }));
    let saved = app
        .call("instructions.set", json!({ "text": "Be brief.\n" }))
        .await
        .expect("set");
    assert_eq!(saved, json!({ "text": "Be brief." }));
    assert_eq!(
        app.call("instructions.get", json!(null))
            .await
            .expect("get"),
        saved
    );
}

#[tokio::test(start_paused = true)]
async fn instructions_longer_than_an_agents_own_are_refused() {
    let s = setup().await;
    let long = "x".repeat(INSTRUCTIONS_MAX_CHARS + 1);
    assert!(matches!(
        instructions::set(&s.daemon, owner(&long)),
        Err(ApiError::Validation(_))
    ));
    assert_eq!(instructions::get(&s.daemon).expect("get"), owner(""));
}
