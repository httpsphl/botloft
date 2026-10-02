//! "Allow always" (spec 10.1): the owner allows a request for good, the bot
//! stops asking for that one, and removing the rule brings the question
//! back.

mod common;

use std::time::Duration;

use common::bots::{Crew, two_bots};
use common::mcp::Mcp;
use common::stream;
use serde_json::{Value, json};

/// The next request that waits for the owner.
async fn next_request(app: &mut common::Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            return changed;
        }
    }
}

/// A crew whose @lead is in a turn, ready to use tools.
async fn working() -> (Crew, Value) {
    let mut c = two_bots().await;
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": "Go" }),
        )
        .await
        .expect("send");
    let lines = c.lead_process.wait_lines(1).await;
    let line = lines.last().expect("line");
    c.lead_process.emit(stream::replay(line)).await;
    (c, lead)
}

/// @lead calls `tool` with `input` and Claude Code asks for permission; the
/// decision comes out of the handle.
async fn ask(c: &Crew, id: &str, tool: &str, input: &Value) -> tokio::task::JoinHandle<Value> {
    c.lead_process
        .emit(stream::tool_use(id, tool, input.clone()))
        .await;
    let mut mcp = Mcp::new(c.lead.addr, c.lead.token.clone());
    let args = json!({ "tool_name": tool, "input": input, "tool_use_id": id });
    tokio::spawn(async move { mcp.tool("permission_prompt", args).await.expect("decision") })
}

/// The decision, which must come without the owner.
async fn by_itself(decision: tokio::task::JoinHandle<Value>) -> Value {
    tokio::time::timeout(Duration::from_secs(10), decision)
        .await
        .expect("allowed without asking")
        .expect("task")
}

#[tokio::test]
async fn allowed_always_the_bot_stops_asking_until_the_rule_goes() {
    let (mut c, lead) = working().await;
    let status = json!({ "command": "git status", "description": "Shows what changed" });
    let decision = ask(&c, "toolu_1", "Bash", &status).await;
    let pending = next_request(&mut c.app).await;
    let body = &pending["item"]["body"];
    assert_eq!(
        body["always"],
        json!({ "toolName": "Bash", "kind": "command", "value": "git status" })
    );
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": body["approvalId"], "allow": true, "always": true }),
        )
        .await
        .expect("answer");
    assert_eq!(decision.await.expect("task")["behavior"], "allow");
    let rules = c.app.notification("bot.rules").await;
    assert_eq!(rules["botId"], lead["id"]);
    assert_eq!(rules["rules"][0]["scope"]["value"], "git status");

    // The same command again: allowed without a request.
    let again = ask(&c, "toolu_2", "Bash", &status).await;
    assert_eq!(
        by_itself(again).await,
        json!({ "behavior": "allow", "updatedInput": status })
    );

    // Another command still asks.
    let other = json!({ "command": "git push" });
    let _pushing = ask(&c, "toolu_3", "Bash", &other).await;
    let asked = next_request(&mut c.app).await;
    assert_eq!(asked["item"]["body"]["summary"], "git push");

    let listed = c
        .app
        .call("rules.list", json!({ "botId": lead["id"] }))
        .await
        .expect("list");
    assert_eq!(listed.as_array().map(Vec::len), Some(1));
    let left = c
        .app
        .call("rules.delete", json!({ "ruleId": listed[0]["id"] }))
        .await
        .expect("delete");
    assert_eq!(left, json!({ "botId": lead["id"], "rules": [] }));

    // Without the rule, it asks again.
    let _status = ask(&c, "toolu_4", "Bash", &status).await;
    loop {
        let asked = next_request(&mut c.app).await;
        if asked["item"]["body"]["summary"] == "git status" {
            break;
        }
    }
}

#[tokio::test]
async fn a_site_rule_covers_its_pages_and_a_plain_allow_keeps_none() {
    let (mut c, lead) = working().await;
    let page = json!({ "url": "https://docs.example.com/a", "prompt": "Read it" });
    let decision = ask(&c, "toolu_1", "WebFetch", &page).await;
    let pending = next_request(&mut c.app).await;
    let body = &pending["item"]["body"];
    assert_eq!(body["always"]["value"], "docs.example.com");
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": body["approvalId"], "allow": true, "always": true }),
        )
        .await
        .expect("answer");
    decision.await.expect("task");
    let other_page = json!({ "url": "https://docs.example.com/b?x=1", "prompt": "And this" });
    let again = ask(&c, "toolu_2", "WebFetch", &other_page).await;
    assert_eq!(by_itself(again).await["behavior"], "allow");

    // A search can be allowed for good too, but allowed once it is not.
    let search = json!({ "query": "rust" });
    let decision = ask(&c, "toolu_3", "WebSearch", &search).await;
    let pending = next_request(&mut c.app).await;
    let body = &pending["item"]["body"];
    assert_eq!(body["always"]["kind"], "tool");
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": body["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    decision.await.expect("task");
    let listed = c
        .app
        .call("rules.list", json!({ "botId": lead["id"] }))
        .await
        .expect("list");
    assert_eq!(listed.as_array().map(Vec::len), Some(1));

    // A plan has no "Allow always".
    let plan = json!({ "plan": "# Do it" });
    let _planning = ask(&c, "toolu_4", "ExitPlanMode", &plan).await;
    let pending = next_request(&mut c.app).await;
    assert_eq!(pending["item"]["body"]["always"], Value::Null);
}
