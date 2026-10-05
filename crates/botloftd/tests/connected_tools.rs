//! Connected tools (spec 25): the owner registers an MCP server, attaches it
//! to a bot, and the bot starts with it in its `mcp.json` and its secrets in
//! its environment, and with nothing else of the owner's.

mod common;

use std::collections::BTreeMap;

use botloft_core::protocol::{
    BotMcpSetParams, BotState, McpKind, McpSaveParams, McpServerIdParams,
};
use botloftd::service::{ApiError, mcp};
use common::supervised::{Setup, setup};
use serde_json::Value;

const COOKIE: &str = "li_at=super-secret-cookie";

fn linkedin() -> McpSaveParams {
    McpSaveParams {
        server_id: None,
        name: "LinkedIn".to_owned(),
        kind: McpKind::Stdio,
        url: None,
        command: Some("uvx".to_owned()),
        args: vec!["linkedin-mcp-server".to_owned()],
        headers: BTreeMap::new(),
        env: BTreeMap::from([("LI_COOKIE".to_owned(), COOKIE.to_owned())]),
        description: "Reads people on LinkedIn.".to_owned(),
    }
}

fn mcp_json(process: &botloftd::runtime::fake::FakeProcess) -> Value {
    let text = std::fs::read_to_string(process.spec.cwd.join(".botloft").join("mcp.json"))
        .expect("mcp.json");
    serde_json::from_str(&text).expect("valid json")
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
async fn a_bot_has_no_connected_tools_until_the_owner_attaches_one() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;

    let overview = mcp::save(&s.daemon, linkedin()).expect("save");
    assert_eq!(overview.servers.len(), 1);
    assert!(overview.bots.is_empty());
    assert!(!first.killed(), "a registered tool alone changes no bot");
    let servers = mcp_json(&first)["mcpServers"].clone();
    assert_eq!(servers.as_object().expect("object").len(), 1);
    assert!(servers.get("botloft").is_some());
}

#[tokio::test(start_paused = true)]
async fn an_attached_tool_reaches_the_bot_with_its_secrets_only_in_the_environment() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let server = mcp::save(&s.daemon, linkedin())
        .expect("save")
        .servers
        .remove(0);

    attach(&s, &server.id);
    let second = s.runtime.process(2).await;
    assert!(first.killed(), "the bot restarts to read its new mcp.json");

    let json = mcp_json(&second);
    let entry = &json["mcpServers"]["linkedin"];
    assert_eq!(entry["command"], "uvx");
    assert_eq!(entry["args"][0], "linkedin-mcp-server");
    let var = entry["env"]["LI_COOKIE"]
        .as_str()
        .and_then(|value| value.strip_prefix("${"))
        .and_then(|value| value.strip_suffix('}'))
        .expect("an expansion")
        .to_owned();
    assert_eq!(second.env(&var).as_deref(), Some(COOKIE));
    assert!(json["mcpServers"].get("botloft").is_some());

    // Nowhere else on disk the bot can read.
    let workspace = &second.spec.cwd;
    let rules = std::fs::read_to_string(workspace.join(".claude/rules/botloft.md")).expect("rules");
    assert!(rules.contains("`mcp__linkedin__*`"));
    assert!(rules.contains("Reads people on LinkedIn."));
    assert!(!rules.contains(COOKIE));
    assert!(!json.to_string().contains(COOKIE));
    let secrets = s
        .daemon
        .paths
        .secrets()
        .join("mcp")
        .join(format!("{}.json", server.id));
    assert!(
        std::fs::read_to_string(secrets)
            .expect("secrets file")
            .contains(COOKIE)
    );
    assert!(!format!("{:?}", s.daemon.store().mcp_servers().expect("list")).contains(COOKIE));
}

#[tokio::test(start_paused = true)]
async fn detaching_and_deleting_take_the_tool_away_from_the_bot() {
    let s = setup().await;
    s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let server = mcp::save(&s.daemon, linkedin())
        .expect("save")
        .servers
        .remove(0);
    attach(&s, &server.id);
    let second = s.runtime.process(2).await;
    s.until(BotState::Idle).await;

    let overview = mcp::delete(
        &s.daemon,
        McpServerIdParams {
            server_id: server.id.clone(),
        },
    )
    .expect("delete");
    assert!(overview.servers.is_empty() && overview.bots.is_empty());
    let third = s.runtime.process(3).await;
    assert!(second.killed());
    assert!(mcp_json(&third)["mcpServers"].get("linkedin").is_none());
    let secrets = s
        .daemon
        .paths
        .secrets()
        .join("mcp")
        .join(format!("{}.json", server.id));
    assert!(!secrets.exists(), "the secrets go with the tool");
}

#[tokio::test(start_paused = true)]
async fn changing_a_tool_keeps_a_secret_left_empty() {
    let s = setup().await;
    s.runtime.process(1).await;
    let server = mcp::save(&s.daemon, linkedin())
        .expect("save")
        .servers
        .remove(0);

    let mut edit = linkedin();
    edit.server_id = Some(server.id.clone());
    edit.description = "Reads LinkedIn profiles.".to_owned();
    edit.env = BTreeMap::from([("LI_COOKIE".to_owned(), String::new())]);
    let overview = mcp::save(&s.daemon, edit).expect("edit");
    assert_eq!(overview.servers[0].description, "Reads LinkedIn profiles.");

    let secrets = botloftd::mcp_secrets::load(&s.daemon.paths.secrets(), &server.id).expect("load");
    assert_eq!(secrets.env["LI_COOKIE"], COOKIE);
}

#[tokio::test(start_paused = true)]
async fn the_owner_cannot_register_what_would_break_the_bot() {
    let s = setup().await;
    let bad = |edit: fn(&mut McpSaveParams)| {
        let mut params = linkedin();
        edit(&mut params);
        mcp::save(&s.daemon, params)
    };
    // Botloft's own server, and a name that is nothing once made a token.
    assert!(matches!(
        bad(|p| p.name = "Botloft".into()),
        Err(ApiError::Validation(_))
    ));
    assert!(matches!(
        bad(|p| p.name = "!!!".into()),
        Err(ApiError::Validation(_))
    ));
    // A variable that would shadow the bot's own.
    assert!(matches!(
        bad(|p| p.env = BTreeMap::from([("BOTLOFT_BOT_TOKEN".into(), "x".into())])),
        Err(ApiError::Validation(_))
    ));
    // A program with an address, and an address that is not one.
    assert!(matches!(
        bad(|p| p.url = Some("http://x".into())),
        Err(ApiError::Validation(_))
    ));
    assert!(matches!(
        bad(|p| {
            p.kind = McpKind::Http;
            p.command = None;
            p.args.clear();
            p.env.clear();
            p.url = Some("file:///etc/passwd".into());
        }),
        Err(ApiError::Validation(_))
    ));
    assert!(
        mcp::overview(&s.daemon)
            .expect("overview")
            .servers
            .is_empty()
    );

    // The same name twice.
    mcp::save(&s.daemon, linkedin()).expect("first");
    assert!(matches!(
        mcp::save(&s.daemon, linkedin()),
        Err(ApiError::Conflict(_))
    ));
}

#[tokio::test(start_paused = true)]
async fn only_servers_that_exist_can_be_attached() {
    let s = setup().await;
    s.runtime.process(1).await;
    let missing = botloft_core::ids::McpServerId::generate();
    let result = mcp::set_bot(
        &s.daemon,
        BotMcpSetParams {
            bot_id: s.bot.clone(),
            server_ids: vec![missing],
        },
    );
    assert!(matches!(result, Err(ApiError::NotFound(_))));
}
