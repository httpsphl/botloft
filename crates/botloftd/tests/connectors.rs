//! The owner's claude.ai connectors in an agent (spec 34): how an agent
//! given some starts, and which MCP servers' tools it may use.

mod common;

use botloft_core::protocol::{
    BotState, BotsSetConnectorsParams, ConnectorState, ConnectorsListParams,
};
use botloftd::connectors;
use common::stream;
use common::supervised::{arg_after, session_of, setup};
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

/// A connector the last look found needing a sign-in, and that the agent
/// uses after the owner signed in on claude.ai, shows as connected: each
/// turn's `system/init` says how it stands for the agent.
#[tokio::test(start_paused = true)]
async fn what_an_agent_sees_at_a_turn_settles_the_connectors_state() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let kept = s.daemon.paths.home.join("connectors").join("known.json");
    std::fs::create_dir_all(kept.parent().expect("dir")).expect("dir");
    std::fs::write(
        &kept,
        json!({ "connectors": [
            { "name": "Canva", "state": "needs_auth" },
            { "name": "Gmail", "state": "connected" }
        ], "checkedAt": 1 })
        .to_string(),
    )
    .expect("known");

    let session = session_of(&process);
    s.message("make a poster");
    process
        .emit(json!({
            "type": "system", "subtype": "init", "session_id": session,
            "mcp_servers": [
                { "name": "botloft", "status": "connected", "source": "dynamic" },
                { "name": "claude.ai Canva", "status": "connected", "source": "claudeai" },
                { "name": "claude.ai Gmail", "status": "pending", "source": "claudeai" },
            ]
        }))
        .await;
    process.emit(stream::began(&session)).await;
    process.emit(stream::result(false)).await;
    s.until(BotState::Idle).await;

    let list = connectors::list(&s.daemon, ConnectorsListParams { refresh: false }).expect("list");
    let seen: Vec<_> = list
        .connectors
        .iter()
        .map(|connector| (connector.name.as_str(), connector.state))
        .collect();
    assert_eq!(
        seen,
        [
            ("Canva", ConnectorState::Connected),
            // Still connecting in that turn: what the look found stands.
            ("Gmail", ConnectorState::Connected),
        ]
    );
    // And a new daemon starts from it.
    let on_disk = std::fs::read_to_string(&kept).expect("kept");
    assert!(on_disk.contains(r#"{"name":"Canva","state":"connected"}"#));
}

fn connectors_workspace(s: &common::supervised::Setup) -> std::path::PathBuf {
    std::path::PathBuf::from(common::context::bot(s).workspace)
}
