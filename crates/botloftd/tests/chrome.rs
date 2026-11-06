//! The owner's own Chrome in the agents (spec 37.2): how an agent given it
//! starts, and how it starts again without it.

mod common;

use botloft_core::protocol::{BotState, ChromeSetParams};
use botloftd::chrome;
use botloftd::service::ApiError;
use common::supervised::setup;
use serde_json::{Value, json};

#[tokio::test(start_paused = true)]
async fn an_agent_starts_with_the_chrome_only_once_the_owner_gives_it() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    assert!(first.args().iter().any(|arg| arg == "--no-chrome"));
    assert!(!first.args().iter().any(|arg| arg == "--chrome"));
    let workspace = first.spec.cwd.clone();
    let settings: Value = serde_json::from_str(
        &std::fs::read_to_string(workspace.join(".claude/settings.json")).expect("settings"),
    )
    .expect("json");
    assert_eq!(
        settings["permissions"]["ask"],
        json!(["mcp__claude-in-chrome"])
    );
    let rules = std::fs::read_to_string(workspace.join(".claude/rules/chrome.md")).expect("rules");
    assert!(rules.contains("ask_chrome"), "{rules}");

    let give = |accepted_risks| ChromeSetParams {
        bot_id: s.bot.clone(),
        on: true,
        accepted_risks,
    };
    let refused = chrome::set(&s.daemon, give(false)).expect_err("no risks accepted");
    assert!(matches!(refused, ApiError::Conflict(_)), "{refused:?}");
    let given = chrome::set(&s.daemon, give(true)).expect("given");
    assert!(given.given_at.is_some());
    // It starts again with it, as nothing is in progress.
    let second = s.runtime.process(2).await;
    assert!(second.args().iter().any(|arg| arg == "--chrome"));
    assert!(!second.args().iter().any(|arg| arg == "--no-chrome"));
    let rules = std::fs::read_to_string(workspace.join(".claude/rules/chrome.md")).expect("rules");
    assert!(rules.contains("mcp__claude-in-chrome__*"), "{rules}");
    let all = botloftd::service::permissions::list(&s.daemon).expect("permissions");
    assert_eq!(all.chrome.len(), 1);
    assert_eq!(all.chrome[0].bot_id, s.bot);

    let taken = chrome::set(
        &s.daemon,
        ChromeSetParams {
            bot_id: s.bot.clone(),
            on: false,
            accepted_risks: false,
        },
    )
    .expect("taken");
    assert_eq!(taken.given_at, None);
    let third = s.runtime.process(3).await;
    assert!(third.args().iter().any(|arg| arg == "--no-chrome"));
}
