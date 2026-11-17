//! An agent asking for a tool of the owner's claude.ai account (spec 34.6):
//! its rules name what it can ask for, the owner allows it in the chat, and
//! the agent gets the tool and a note to go on.

mod common;

use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

const TOOL: &str = "mcp__botloft__ask_connector";

async fn pending(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            assert_eq!(body["toolName"], TOOL);
            return body.clone();
        }
    }
}

#[tokio::test]
async fn an_agent_asks_for_google_calendar_and_gets_it_when_the_owner_allows() {
    let t = TestDaemon::start_supervised().await;
    // What the last look at the owner's account found.
    let dir = t.paths.home.join("connectors");
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
    let mut app = t.session().await;
    let ops = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("ops");
    let (scout, _, mut scout_mcp) = ready_bot(&t, &mut app, &ops, "Scout").await;
    let workspace = std::path::PathBuf::from(scout["workspace"].as_str().expect("workspace"));
    let rules =
        std::fs::read_to_string(workspace.join(".claude/rules/connectors.md")).expect("rules");
    assert!(rules.contains("ask_connector"), "{rules}");
    assert!(rules.contains("- Google Calendar"), "{rules}");
    assert!(
        rules.contains("Never open one of them in your browser"),
        "{rules}"
    );

    // A service the account does not have points to where to connect it.
    let refused = scout_mcp
        .tool("ask_connector", json!({ "name": "Slack" }))
        .await
        .expect_err("not on the account");
    assert!(refused.contains("Settings > Tools > Discover"), "{refused}");

    let mut asking = Mcp::new(scout_mcp.addr, scout_mcp.token.clone());
    let waiting = tokio::spawn(async move {
        asking
            .tool(
                "ask_connector",
                json!({ "name": "google calendar", "why": "To see tomorrow's meetings." }),
            )
            .await
    });
    let asked = pending(&mut app).await;
    assert_eq!(asked["summary"], "Google Calendar");
    app.call(
        "approvals.answer",
        json!({ "approvalId": asked["approvalId"], "allow": true }),
    )
    .await
    .expect("answer");
    let given = waiting.await.expect("task").expect("given");
    assert_eq!(given["done"], true);
    let bots = app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(bots[0]["connectors"], json!(["Google Calendar"]));
}
