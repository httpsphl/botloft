//! The MCP endpoint over HTTP (spec 10): discovery on 2026-07-28, the
//! `initialize` fallback of earlier revisions, and what gets refused.

mod common;

use common::TestDaemon;
use common::mcp::{self, MODERN, Mcp, Reply};
use serde_json::{Value, json};

const HEADER_MISMATCH: i64 = -32020;
const UNSUPPORTED_VERSION: i64 = -32022;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;

/// A daemon with one running bot and that bot's MCP client.
async fn one_bot() -> (TestDaemon, Mcp) {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "Finds things", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    let process = t.process_of(&bot).await;
    let token = process.env("BOTLOFT_BOT_TOKEN").expect("token");
    let client = Mcp::new(t.addr, token);
    (t, client)
}

fn code(reply: &Reply) -> i64 {
    reply.body["error"]["code"].as_i64().expect("error code")
}

#[tokio::test]
async fn claude_code_discovers_the_server_and_lists_the_tools() {
    let (_t, mut client) = one_bot().await;
    let discovered = client.request("server/discover", json!({})).await;
    assert_eq!(discovered.status, 200);
    let result = &discovered.body["result"];
    assert_eq!(result["resultType"], "complete");
    assert_eq!(result["supportedVersions"][0], MODERN);
    assert!(
        result["supportedVersions"]
            .as_array()
            .expect("versions")
            .contains(&json!("2025-11-25"))
    );
    assert_eq!(result["capabilities"], json!({ "tools": {} }));
    assert_eq!(
        result["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
        "botloft"
    );
    // Caching hints are required; Claude Code rejects the result without them.
    assert_eq!(result["cacheScope"], "public");
    assert!(result["ttlMs"].as_u64().is_some_and(|ttl| ttl > 0));

    let listed = client.request("tools/list", json!({})).await;
    assert_eq!(listed.body["result"]["cacheScope"], "public");
    assert!(listed.body["result"]["ttlMs"].is_u64());
    let names: Vec<_> = listed.body["result"]["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .map(|tool| tool["name"].as_str().expect("name").to_owned())
        .collect();
    assert_eq!(
        names,
        [
            "crew_roster",
            "send_message",
            "complete_task",
            "my_tasks",
            "suggest_bot",
            "schedule_routine",
            "share_file",
            "ask_owner",
            "send_signal",
            "my_routines",
            "change_routine",
            "delete_routine",
            "browser_open",
            "browser_look",
            "browser_click",
            "browser_type",
            "browser_select",
            "browser_press",
            "browser_scroll",
            "browser_back",
            "browser_screenshot",
            "browser_ask_owner",
            "browser_close",
            "desktop_windows",
            "desktop_look",
            "desktop_screenshot",
            "desktop_click",
            "desktop_type",
            "desktop_select",
            "desktop_scroll",
            "desktop_press",
            "desktop_click_at",
            "permission_prompt"
        ]
    );

    let unknown = client.request("prompts/list", json!({})).await;
    assert_eq!((unknown.status, code(&unknown)), (404, METHOD_NOT_FOUND));
}

#[tokio::test]
async fn earlier_revisions_start_with_initialize() {
    let (t, client) = one_bot().await;
    let auth = format!("Bearer {}", client.token);
    let initialize = |version: &str| {
        json!({ "jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": version, "capabilities": {},
            "clientInfo": { "name": "claude-code", "version": "2.1.284" } } })
        .to_string()
    };
    let headers = [("Authorization", auth.as_str())];
    let reply = mcp::post(t.addr, &headers, &initialize("2025-06-18")).await;
    assert_eq!(reply.status, 200);
    let result = &reply.body["result"];
    assert_eq!(result["protocolVersion"], "2025-06-18");
    assert_eq!(result["serverInfo"]["name"], "botloft");
    assert_eq!(
        result["resultType"],
        Value::Null,
        "only 2026-07-28 results carry it"
    );
    let newer = mcp::post(t.addr, &headers, &initialize("2030-01-01")).await;
    assert_eq!(newer.body["result"]["protocolVersion"], "2025-11-25");

    let initialized = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    let accepted = mcp::post(t.addr, &headers, &initialized.to_string()).await;
    assert_eq!((accepted.status, accepted.body.clone()), (202, Value::Null));

    let legacy = [
        ("Authorization", auth.as_str()),
        ("MCP-Protocol-Version", "2025-11-25"),
    ];
    let list = json!({ "jsonrpc": "2.0", "id": 1, "method": "tools/list" }).to_string();
    let listed = mcp::post(t.addr, &legacy, &list).await;
    assert_eq!(
        listed.body["result"]["tools"].as_array().map(Vec::len),
        Some(33)
    );
    let unknown = json!({ "jsonrpc": "2.0", "id": 2, "method": "prompts/list" }).to_string();
    let unknown = mcp::post(t.addr, &legacy, &unknown).await;
    assert_eq!((unknown.status, code(&unknown)), (200, METHOD_NOT_FOUND));
}

#[tokio::test]
async fn requests_must_be_authenticated_and_consistent() {
    let (t, mut client) = one_bot().await;
    let ping = json!({ "jsonrpc": "2.0", "id": 1, "method": "ping" }).to_string();
    assert_eq!(mcp::post(t.addr, &[], &ping).await.status, 401);
    let wrong = [("Authorization", "Bearer not-a-token")];
    assert_eq!(mcp::post(t.addr, &wrong, &ping).await.status, 401);
    let auth = format!("Bearer {}", client.token);
    let browser = [
        ("Authorization", auth.as_str()),
        ("Origin", "http://evil.test"),
    ];
    assert_eq!(mcp::post(t.addr, &browser, &ping).await.status, 403);
    assert_eq!(
        mcp::raw(t.addr, "GET", &[("Authorization", auth.as_str())], "")
            .await
            .status,
        405
    );

    let future = [
        ("Authorization", auth.as_str()),
        ("MCP-Protocol-Version", "2099-01-01"),
    ];
    let refused = mcp::post(t.addr, &future, &ping).await;
    assert_eq!((refused.status, code(&refused)), (400, UNSUPPORTED_VERSION));
    assert_eq!(refused.body["error"]["data"]["requested"], "2099-01-01");
    assert_eq!(refused.body["error"]["data"]["supported"][0], MODERN);

    let no_meta = [
        ("Authorization", auth.as_str()),
        ("MCP-Protocol-Version", MODERN),
        ("Mcp-Method", "ping"),
    ];
    let missing = mcp::post(t.addr, &no_meta, &ping).await;
    assert_eq!((missing.status, code(&missing)), (400, INVALID_PARAMS));

    let call = |name: &str| {
        json!({ "jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
            "name": name, "arguments": {},
            "_meta": {
                "io.modelcontextprotocol/protocolVersion": MODERN,
                "io.modelcontextprotocol/clientCapabilities": {},
            } } })
        .to_string()
    };
    let headers = |method: &'static str, name: &'static str| {
        vec![
            ("Authorization", auth.as_str()),
            ("MCP-Protocol-Version", MODERN),
            ("Mcp-Method", method),
            ("Mcp-Name", name),
        ]
    };
    let lying = mcp::post(
        t.addr,
        &headers("tools/call", "my_tasks"),
        &call("crew_roster"),
    )
    .await;
    assert_eq!((lying.status, code(&lying)), (400, HEADER_MISMATCH));
    let wrong_method = mcp::post(
        t.addr,
        &headers("tools/list", "crew_roster"),
        &call("crew_roster"),
    )
    .await;
    assert_eq!(
        (wrong_method.status, code(&wrong_method)),
        (400, HEADER_MISMATCH)
    );
    // "crew_roster" in the base64 form clients use for unsafe values.
    let encoded = headers("tools/call", "=?base64?Y3Jld19yb3N0ZXI=?=");
    let accepted = mcp::post(t.addr, &encoded, &call("crew_roster")).await;
    assert_eq!(accepted.status, 200, "{:?}", accepted.body);

    let unknown_tool = client
        .request("tools/call", json!({ "name": "rm_rf" }))
        .await;
    assert_eq!(
        (unknown_tool.status, code(&unknown_tool)),
        (200, INVALID_PARAMS)
    );
}

#[tokio::test]
async fn a_restart_revokes_the_old_token() {
    let (t, mut client) = one_bot().await;
    assert_eq!(client.request("ping", json!({})).await.status, 200);
    t.runtime.process(1).await.exit(1).await;
    t.runtime.process(2).await;
    assert_eq!(client.request("ping", json!({})).await.status, 401);
}
