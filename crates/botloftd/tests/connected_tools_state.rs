//! Connected tools, the second half (spec 25.4, 25.5): what a bot's process
//! says about each server when it starts, and what the owner allowed for
//! good going away with the server.

mod common;

use std::collections::BTreeMap;
use std::time::Duration;

use botloft_core::protocol::{
    AllowKind, AllowScope, BotMcpSetParams, BotState, McpKind, McpSaveParams, McpServerIdParams,
    McpState,
};
use botloftd::service::mcp;
use common::supervised::{Setup, setup};
use serde_json::json;

fn linkedin() -> McpSaveParams {
    McpSaveParams {
        server_id: None,
        name: "LinkedIn".to_owned(),
        kind: McpKind::Stdio,
        url: None,
        command: Some("uvx".to_owned()),
        args: vec!["linkedin-mcp-server".to_owned()],
        headers: BTreeMap::new(),
        env: BTreeMap::from([("LI_COOKIE".to_owned(), "cookie".to_owned())]),
        description: String::new(),
    }
}

fn attach(s: &Setup, server: &botloft_core::ids::McpServerId) {
    mcp::set_bot(
        &s.daemon,
        BotMcpSetParams {
            bot_id: s.bot.clone(),
            server_ids: vec![server.clone()],
        },
    )
    .expect("attach");
}

#[tokio::test(start_paused = true)]
async fn a_bot_with_connected_tools_asks_its_process_and_shows_the_answer() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    assert!(
        !first.control_requests().contains(&"mcp_status".to_owned()),
        "a bot without connected tools has nothing to ask"
    );

    let server = mcp::save(&s.daemon, linkedin())
        .expect("save")
        .servers
        .remove(0);
    let mut events = s.daemon.subscribe();
    attach(&s, &server.id);
    let second = s.runtime.process(2).await;
    assert!(second.control_requests().contains(&"mcp_status".to_owned()));

    // Claude Code also reports Botloft's own server, and the configuration
    // it expanded: neither may get anywhere.
    second
        .answer_control(
            "mcp_status",
            json!({ "mcpServers": [
                { "name": "botloft", "status": "connected" },
                { "name": "linkedin", "status": "failed", "error": "Connection closed\n",
                  "config": { "env": { "LI_COOKIE": "expanded-secret" } } },
            ] }),
        )
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let overview = mcp::overview(&s.daemon).expect("overview");
    let states = &overview.bots[0].states;
    assert_eq!(states.len(), 1);
    assert_eq!(states[0].server_id, server.id);
    assert_eq!(states[0].state, McpState::Failed);
    assert_eq!(states[0].error.as_deref(), Some("Connection closed"));
    assert!(!format!("{overview:?}").contains("expanded-secret"));
    let mut told = false;
    while let Ok(event) = events.try_recv() {
        told |= format!("{event:?}").contains("Connection closed");
    }
    assert!(told, "the app hears about it");

    // A new process says it again; what the old one said is not kept.
    let mut edit = linkedin();
    edit.server_id = Some(server.id.clone());
    edit.description = "Reads people.".to_owned();
    mcp::save(&s.daemon, edit).expect("edit");
    let third = s.runtime.process(3).await;
    assert!(second.killed());
    assert!(
        mcp::overview(&s.daemon).expect("overview").bots[0]
            .states
            .is_empty()
    );
    third
        .answer_control(
            "mcp_status",
            json!({ "mcpServers": [{ "name": "linkedin", "status": "connected" }] }),
        )
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        mcp::overview(&s.daemon).expect("overview").bots[0].states[0].state,
        McpState::Connected
    );
}

#[tokio::test(start_paused = true)]
async fn what_was_allowed_for_good_goes_with_the_tool() {
    let s = setup().await;
    s.runtime.process(1).await;
    let server = mcp::save(&s.daemon, linkedin())
        .expect("save")
        .servers
        .remove(0);
    attach(&s, &server.id);
    let allow = |tool: &str| AllowScope {
        tool_name: tool.to_owned(),
        kind: AllowKind::Tool,
        value: tool.to_owned(),
    };
    {
        let store = s.daemon.store();
        for tool in ["mcp__linkedin__search_people", "mcp__other__search"] {
            store
                .add_allow_rule(&s.bot, &allow(tool), 1)
                .expect("allow");
        }
    }

    mcp::delete(
        &s.daemon,
        McpServerIdParams {
            server_id: server.id,
        },
    )
    .expect("delete");
    let rules = s.daemon.store().allow_rules(&s.bot).expect("rules");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].scope.tool_name, "mcp__other__search");
}
