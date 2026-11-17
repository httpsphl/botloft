//! The requests of the owner's Chrome (spec 37.3 to 37.5): how each one is
//! decided (who, whose turn, which site) and how an agent asks for it.

mod common;

use std::time::Duration;

use botloftd::runtime::fake::FakeProcess;
use common::bots::{Crew, two_bots};
use common::mcp::Mcp;
use common::stream;
use serde_json::{Value, json};

const NAVIGATE: &str = "mcp__claude-in-chrome__navigate";
const COMPUTER: &str = "mcp__claude-in-chrome__computer";

/// The next request that waits for the owner.
async fn next_request(app: &mut common::Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            return body.clone();
        }
    }
}

/// Starts a turn of the bot named `name`.
async fn turn(c: &mut Crew, at: usize, process: &FakeProcess) -> Value {
    let bot = c.app.call("bots.list", json!({})).await.expect("bots")[at].clone();
    c.app
        .call("messages.send", json!({ "botId": bot["id"], "body": "Go" }))
        .await
        .expect("send");
    let lines = process.wait_lines(1).await;
    process
        .emit(stream::replay(lines.last().expect("line")))
        .await;
    bot
}

/// The bot behind `process` calls `tool` and Claude Code asks for it; the
/// decision comes out of the handle.
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

/// A decision that comes without the owner.
async fn by_itself(decision: tokio::task::JoinHandle<Value>) -> Value {
    tokio::time::timeout(Duration::from_secs(10), decision)
        .await
        .expect("decided without asking")
        .expect("task")
}

async fn give(c: &mut Crew, bot: &Value) {
    c.app
        .call(
            "chrome.set",
            json!({ "botId": bot["id"], "on": true, "acceptedRisks": true }),
        )
        .await
        .expect("give");
}

#[tokio::test]
async fn the_owner_allows_a_site_once_and_the_agent_acts_on_its_tabs_without_asking() {
    let mut c = two_bots().await;
    let lead_process = c.lead_process.clone();
    let lead = turn(&mut c, 0, &lead_process).await;

    // Without the owner's Chrome, its tools never run.
    let suno = json!({ "tabId": 1, "url": "https://suno.com/song/1" });
    let refused =
        by_itself(ask(&lead_process, &c.lead, "toolu_1", NAVIGATE, suno.clone()).await).await;
    assert_eq!(refused["behavior"], "deny");
    assert!(
        refused["message"]
            .as_str()
            .expect("message")
            .contains("ask_chrome")
    );

    // Given while it works, it keeps this process until the turn ends.
    give(&mut c, &lead).await;
    let look = ask(
        &lead_process,
        &c.lead,
        "toolu_2",
        "mcp__claude-in-chrome__tabs_context_mcp",
        json!({}),
    )
    .await;
    assert_eq!(by_itself(look).await["behavior"], "allow");

    let opening = ask(&lead_process, &c.lead, "toolu_3", NAVIGATE, suno).await;
    let asked = next_request(&mut c.app).await;
    assert_eq!(asked["toolName"], NAVIGATE);
    assert_eq!(asked["summary"], "suno.com");
    assert_eq!(
        asked["always"],
        json!({ "toolName": "mcp__claude-in-chrome", "kind": "site", "value": "suno.com" })
    );
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true, "always": true }),
        )
        .await
        .expect("answer");
    assert_eq!(opening.await.expect("task")["behavior"], "allow");

    // On the tab it opened there, clicks need no asking.
    let click = json!({ "tabId": 1, "action": "left_click", "ref": "e3" });
    let clicked = ask(&lead_process, &c.lead, "toolu_4", COMPUTER, click).await;
    assert_eq!(by_itself(clicked).await["behavior"], "allow");
    // A tab it did not open, and a script, still ask, with no "always".
    let other = ask(
        &lead_process,
        &c.lead,
        "toolu_5",
        COMPUTER,
        json!({ "tabId": 2, "action": "screenshot" }),
    )
    .await;
    let asked = next_request(&mut c.app).await;
    assert_eq!(asked["always"], Value::Null);
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": false }),
        )
        .await
        .expect("deny");
    assert_eq!(other.await.expect("task")["behavior"], "deny");
    let script = json!({ "tabId": 1, "text": "document.title" });
    let _running = ask(
        &lead_process,
        &c.lead,
        "toolu_6",
        "mcp__claude-in-chrome__javascript_tool",
        script,
    )
    .await;
    let asked = next_request(&mut c.app).await;
    assert_eq!(asked["always"], Value::Null);
}

#[tokio::test]
async fn one_agent_at_a_time_acts_in_the_chrome() {
    let mut c = two_bots().await;
    let (lead_process, writer_process) = (c.lead_process.clone(), c.writer_process.clone());
    let lead = turn(&mut c, 0, &lead_process).await;
    let writer = turn(&mut c, 1, &writer_process).await;
    give(&mut c, &lead).await;
    give(&mut c, &writer).await;

    let tab = json!({});
    let first = ask(
        &lead_process,
        &c.lead,
        "toolu_1",
        "mcp__claude-in-chrome__tabs_create_mcp",
        tab.clone(),
    )
    .await;
    assert_eq!(by_itself(first).await["behavior"], "allow");
    let status = c
        .app
        .call("chrome.status", json!(null))
        .await
        .expect("status");
    assert_eq!(status["botId"], lead["id"]);

    // The writer waits for the lead's turn to end.
    let waiting = ask(
        &writer_process,
        &c.writer,
        "toolu_2",
        "mcp__claude-in-chrome__tabs_create_mcp",
        tab,
    )
    .await;
    tokio::time::sleep(Duration::from_millis(1500)).await;
    assert!(!waiting.is_finished(), "the writer had to wait");
    lead_process.emit(stream::result(false)).await;
    assert_eq!(by_itself(waiting).await["behavior"], "allow");
    let status = c
        .app
        .call("chrome.status", json!(null))
        .await
        .expect("status");
    assert_eq!(status["botId"], writer["id"]);

    // Stopped by the owner, its next actions there are refused.
    let stopped = c.app.call("chrome.stop", json!(null)).await.expect("stop");
    assert_eq!(stopped["botId"], Value::Null);
    let again = ask(
        &writer_process,
        &c.writer,
        "toolu_3",
        "mcp__claude-in-chrome__tabs_create_mcp",
        json!({}),
    )
    .await;
    let refused = by_itself(again).await;
    assert_eq!(refused["behavior"], "deny");
    assert!(
        refused["message"]
            .as_str()
            .expect("message")
            .contains("stopped")
    );
}

#[tokio::test]
async fn an_agent_asks_for_the_chrome_and_gets_it_when_the_owner_allows() {
    let mut c = two_bots().await;
    let lead_process = c.lead_process.clone();
    let lead = turn(&mut c, 0, &lead_process).await;
    let mut asking = Mcp::new(c.lead.addr, c.lead.token.clone());
    let waiting = tokio::spawn(async move {
        asking
            .tool(
                "ask_chrome",
                json!({ "why": "To extend a song on suno.com." }),
            )
            .await
    });
    let asked = next_request(&mut c.app).await;
    assert_eq!(asked["toolName"], "mcp__botloft__ask_chrome");
    assert_eq!(asked["summary"], "To extend a song on suno.com.");
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    let given = waiting.await.expect("task").expect("given");
    assert_eq!(given["done"], true);
    let all = c
        .app
        .call("permissions.list", json!(null))
        .await
        .expect("permissions");
    assert_eq!(all["chrome"][0]["botId"], lead["id"]);
}
