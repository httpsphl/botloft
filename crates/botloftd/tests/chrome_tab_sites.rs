//! The daemon learns where the agent's tabs are from the Chrome's answers
//! (spec 37.3): a tab `navigate` opened without naming it, a tab a click
//! took to another site, and a tab of the group opened before.

mod common;

use std::time::Duration;

use botloftd::runtime::fake::FakeProcess;
use common::bots::two_bots;
use common::mcp::Mcp;
use common::{Client, stream};
use serde_json::{Value, json};

const BATCH: &str = "mcp__claude-in-chrome__browser_batch";

/// Claude Code asks for `tool`; the decision comes out of the handle.
async fn ask(
    process: &FakeProcess,
    mcp: &Mcp,
    id: &str,
    tool: &str,
    input: Value,
) -> tokio::task::JoinHandle<Value> {
    process
        .emit(stream::tool_use(id, tool, input.clone()))
        .await;
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    let args = json!({ "tool_name": tool, "input": input, "tool_use_id": id });
    tokio::spawn(async move { mcp.tool("permission_prompt", args).await.expect("decision") })
}

/// The tool `id` answered `output`, and the chat has it.
async fn answered(process: &FakeProcess, app: &mut Client, id: &str, output: &str) {
    process.emit(stream::tool_result(id, output, false)).await;
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["toolUseId"] == id && body["status"] == "done" {
            return;
        }
    }
}

/// The next request that waits for the owner.
async fn next_request(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            return body.clone();
        }
    }
}

async fn by_itself(decision: tokio::task::JoinHandle<Value>) -> Value {
    tokio::time::timeout(Duration::from_secs(10), decision)
        .await
        .expect("decided without asking")
        .expect("task")
}

fn clicks(tab: i64) -> Value {
    json!({ "actions": [
        { "name": "computer", "input": { "tabId": tab, "action": "left_click", "coordinate": [10, 10] } },
        { "name": "computer", "input": { "tabId": tab, "action": "screenshot" } },
    ]})
}

fn context(tab: i64, url: &str) -> String {
    format!(
        "Done.\n\nTab Context:\n- Executed on tabId: {tab}\n- Available tabs:\n  • tabId {tab}: \"Page\" (\"{url}\")"
    )
}

#[tokio::test]
async fn the_agent_s_tabs_are_known_from_what_the_chrome_answers() {
    let mut c = two_bots().await;
    let process = c.lead_process.clone();
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": "Go" }),
        )
        .await
        .expect("send");
    let lines = process.wait_lines(1).await;
    process
        .emit(stream::replay(lines.last().expect("line")))
        .await;

    // Given while it works, it keeps this process until the turn ends.
    c.app
        .call(
            "chrome.set",
            json!({ "botId": lead["id"], "on": true, "acceptedRisks": true }),
        )
        .await
        .expect("give");

    // `navigate` without a tab, to an address without a protocol: the owner
    // allows the site for good, and the answer says which tab it was.
    let suno = json!({ "url": "suno.com/song/1" });
    let opening = ask(
        &process,
        &c.lead,
        "toolu_1",
        "mcp__claude-in-chrome__navigate",
        suno,
    )
    .await;
    let asked = next_request(&mut c.app).await;
    assert_eq!(asked["summary"], "suno.com");
    let allow = json!({ "approvalId": asked["approvalId"], "allow": true, "always": true });
    c.app.call("approvals.answer", allow).await.expect("answer");
    assert_eq!(opening.await.expect("task")["behavior"], "allow");
    answered(
        &process,
        &mut c.app,
        "toolu_1",
        &context(42, "https://suno.com/song/1"),
    )
    .await;

    // Steps on that tab need no asking.
    let steps = ask(&process, &c.lead, "toolu_2", BATCH, clicks(42)).await;
    assert_eq!(by_itself(steps).await["behavior"], "allow");
    // A click took it to another site: the next steps ask, for that site.
    answered(
        &process,
        &mut c.app,
        "toolu_2",
        &context(42, "https://shop.example/cart"),
    )
    .await;
    let _asking = ask(&process, &c.lead, "toolu_3", BATCH, clicks(42)).await;
    let asked = next_request(&mut c.app).await;
    assert_eq!(asked["summary"], "shop.example");

    // A tab of the group the process did not open: the context tells.
    let listed = "{\"availableTabs\":[{\"tabId\":77,\"title\":\"Create\",\
                  \"url\":\"https://suno.com/create\"}],\"tabGroupId\":3}";
    let look = ask(
        &process,
        &c.lead,
        "toolu_4",
        "mcp__claude-in-chrome__tabs_context_mcp",
        json!({}),
    )
    .await;
    assert_eq!(by_itself(look).await["behavior"], "allow");
    answered(&process, &mut c.app, "toolu_4", listed).await;
    let steps = ask(&process, &c.lead, "toolu_5", BATCH, clicks(77)).await;
    assert_eq!(by_itself(steps).await["behavior"], "allow");
}
