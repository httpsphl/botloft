//! The owner's claude.ai connectors in an agent (spec 34): how an agent
//! given some starts, and which MCP servers' tools it may use.

mod common;

use botloft_core::protocol::{BotState, BotsSetConnectorsParams, ConnectorState};
use botloftd::connectors;
use common::supervised::{arg_after, setup};
use serde_json::json;

#[tokio::test(start_paused = true)]
async fn an_agent_given_a_connector_runs_without_the_fence_and_with_the_rest_blocked() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    assert!(first.args().iter().any(|arg| arg == "--strict-mcp-config"));

    // What the last look found, as an earlier daemon kept it.
    let dir = s.daemon.paths.home.join("connectors");
    std::fs::create_dir_all(&dir).expect("dir");
    std::fs::write(
        dir.join("known.json"),
        json!({ "connectors": [
            { "name": "Gmail", "state": "connected" },
            { "name": "Google Calendar", "state": "connected" }
        ], "checkedAt": 1 })
        .to_string(),
    )
    .expect("known");
    // A server someone put in the agent's folder.
    let workspace = connectors_workspace(&s);
    std::fs::write(
        workspace.join(".mcp.json"),
        json!({ "mcpServers": { "planted": { "command": "x" } } }).to_string(),
    )
    .expect("planted");

    let bot = connectors::set_bot(
        &s.daemon,
        BotsSetConnectorsParams {
            bot_id: s.bot.clone(),
            names: vec!["Google Calendar".into()],
        },
    )
    .expect("set");
    assert_eq!(bot.connectors, ["Google Calendar"]);
    // It starts again with them, as nothing is in progress.
    let second = s.runtime.process(2).await;
    assert!(!second.args().iter().any(|arg| arg == "--strict-mcp-config"));
    let disallowed = arg_after(&second, "--disallowedTools").expect("disallowed");
    assert!(disallowed.starts_with("CronCreate,"));
    assert!(disallowed.contains("mcp__claude_ai_Gmail"));
    assert!(disallowed.contains("mcp__planted"));
    assert!(!disallowed.contains("Google_Calendar"));
    let rules =
        std::fs::read_to_string(workspace.join(".claude/rules/connectors.md")).expect("rules");
    assert!(rules.contains("mcp__claude_ai_Google_Calendar__*"));

    // Only its own tools, the Botloft's and the connector it was given pass.
    assert!(connectors::server_allowed(&s.daemon, &s.bot, "botloft"));
    assert!(connectors::server_allowed(
        &s.daemon,
        &s.bot,
        "claude_ai_Google_Calendar"
    ));
    assert!(!connectors::server_allowed(
        &s.daemon,
        &s.bot,
        "claude_ai_Gmail"
    ));
    assert!(!connectors::server_allowed(&s.daemon, &s.bot, "planted"));
    let _ = ConnectorState::Connected;
}

fn connectors_workspace(s: &common::supervised::Setup) -> std::path::PathBuf {
    std::path::PathBuf::from(common::context::bot(s).workspace)
}
