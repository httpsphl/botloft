//! A bot reaching another crew (spec 10.4): closed until the owner allows
//! it, only now, always for one bot or always for the whole crew; then
//! `crew_roster` and `send_message` take `crew`, and the other side can
//! answer.

mod common;

use botloft_core::ids::BotId;
use common::bots::{ready_bot, text_of};
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

const TOOL: &str = "mcp__botloft__ask_crew_access";

struct Crews {
    t: TestDaemon,
    app: Client,
    ops: Value,
    scout: Value,
    scout_mcp: Mcp,
    writer_mcp: Mcp,
    writer_process: botloftd::runtime::fake::FakeProcess,
}

/// "Ops" with @scout; "Blog" with @writer and @editor.
async fn crews() -> Crews {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let ops = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("ops");
    let blog = app
        .call("crews.create", json!({ "name": "Blog" }))
        .await
        .expect("blog");
    let (scout, _, scout_mcp) = ready_bot(&t, &mut app, &ops, "Scout").await;
    let (_, writer_process, writer_mcp) = ready_bot(&t, &mut app, &blog, "Writer").await;
    ready_bot(&t, &mut app, &blog, "Editor").await;
    Crews {
        t,
        app,
        ops,
        scout,
        scout_mcp,
        writer_mcp,
        writer_process,
    }
}

fn ask(mcp: &Mcp, arguments: Value) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    tokio::spawn(async move { mcp.tool("ask_crew_access", arguments).await })
}

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

async fn answer(app: &mut Client, asked: &Value, scope: Option<&str>) {
    let mut params = json!({ "approvalId": asked["approvalId"], "allow": true });
    if let Some(scope) = scope {
        params["input"] = json!(json!({ "scope": scope }).to_string());
    }
    app.call("approvals.answer", params).await.expect("answer");
}

fn note(to: &str, body: &str) -> Value {
    json!({ "to": to, "crew": "Blog", "body": body })
}

#[tokio::test]
async fn another_crew_is_closed_until_the_owner_lets_a_bot_in() {
    let mut c = crews().await;
    let roster = c
        .scout_mcp
        .tool("crew_roster", json!({ "crew": "Blog" }))
        .await
        .expect_err("closed");
    assert!(roster.contains("ask_crew_access"), "{roster}");
    // A bot there and a bot that is not read the same.
    for to in ["writer", "nobody"] {
        let refused = c
            .scout_mcp
            .tool("send_message", note(to, "Hi"))
            .await
            .expect_err("closed");
        assert!(refused.contains("cannot reach"), "{refused}");
    }
    let own = c
        .scout_mcp
        .tool("crew_roster", json!({ "crew": "ops" }))
        .await
        .expect_err("own crew");
    assert!(own.contains("your own crew"), "{own}");

    // Allowed only now, for @writer.
    let waiting = ask(
        &c.scout_mcp,
        json!({ "crew": "blog", "bot": "@writer", "access": ["talk"], "why": "To get the post." }),
    );
    let asked = pending(&mut c.app).await;
    assert_eq!(asked["summary"], "Blog");
    let input: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    assert_eq!(input["handle"], "writer");
    assert_eq!(input["why"], "To get the post.");
    answer(&mut c.app, &asked, None).await;
    let allowed = waiting.await.expect("task").expect("allowed");
    assert_eq!(allowed["allowed"], true);

    let roster = c
        .scout_mcp
        .tool("crew_roster", json!({ "crew": "Blog" }))
        .await
        .expect("roster");
    let handles: Vec<&str> = roster["bots"]
        .as_array()
        .expect("bots")
        .iter()
        .filter_map(|bot| bot["handle"].as_str())
        .collect();
    assert_eq!(handles, ["writer"]);
    c.scout_mcp
        .tool("send_message", note("writer", "Send me the post."))
        .await
        .expect("sent");
    let lines = c.writer_process.wait_lines(1).await;
    let text = text_of(&lines[0]);
    assert!(
        text.starts_with("[botloft] from @scout of crew Ops · crew Blog"),
        "{text}"
    );
    assert!(
        text.contains("send_message(to: \"scout\", crew: \"Ops\")"),
        "{text}"
    );

    // The writer answers without being let in itself.
    c.writer_mcp
        .tool(
            "send_message",
            json!({ "to": "scout", "crew": "Ops", "body": "Here it is." }),
        )
        .await
        .expect("answer");

    // The turn ends: what was allowed only now goes.
    let scout: BotId = c.scout["id"].as_str().expect("id").parse().expect("bot id");
    c.t.daemon.crew_access.end_turn(&scout);
    let refused = c
        .scout_mcp
        .tool("send_message", note("editor", "Hi"))
        .await
        .expect_err("not let in");
    assert!(refused.contains("cannot reach"), "{refused}");

    // Messages between the crews do not keep a crew from being deleted.
    c.app
        .call("crews.delete", json!({ "crewId": c.ops["id"] }))
        .await
        .expect("delete");
}

#[tokio::test]
async fn always_for_the_crew_lasts_and_a_task_comes_back() {
    let mut c = crews().await;
    // "This bot" needs a bot to be about.
    let waiting = ask(
        &c.scout_mcp,
        json!({ "crew": "Blog", "access": ["talk"], "why": "Weekly report." }),
    );
    let asked = pending(&mut c.app).await;
    let wrong = json!({ "approvalId": asked["approvalId"], "allow": true,
        "input": json!({ "scope": "bot" }).to_string() });
    assert!(c.app.call("approvals.answer", wrong).await.is_err());
    answer(&mut c.app, &asked, Some("crew")).await;
    waiting.await.expect("task").expect("allowed");
    let settled = c
        .app
        .call("chat.history", json!({ "botId": c.scout["id"] }))
        .await
        .expect("history");
    let kept = settled
        .as_array()
        .expect("items")
        .iter()
        .find(|item| item["body"]["kind"] == "approval")
        .expect("approval")["body"]["input"]
        .as_str()
        .expect("input")
        .to_owned();
    let kept: Value = serde_json::from_str(&kept).expect("json");
    assert_eq!(kept["crew"], "Blog");
    assert_eq!(kept["scope"], "crew");

    let scout: BotId = c.scout["id"].as_str().expect("id").parse().expect("bot id");
    c.t.daemon.crew_access.end_turn(&scout);
    let roster = c
        .scout_mcp
        .tool("crew_roster", json!({ "crew": "Blog" }))
        .await
        .expect("roster");
    assert_eq!(roster["bots"].as_array().map(Vec::len), Some(2));

    let sent = c
        .scout_mcp
        .tool(
            "send_message",
            json!({ "to": "writer", "crew": "Blog", "body": "Write it.", "kind": "task" }),
        )
        .await
        .expect("task");
    let mine = c
        .writer_mcp
        .tool("my_tasks", json!({ "role": "assigned" }))
        .await
        .expect("tasks");
    assert_eq!(mine["assigned"][0]["from"], "scout of crew Ops");
    c.writer_mcp
        .tool(
            "complete_task",
            json!({ "task_id": sent["task_id"], "result": "Done." }),
        )
        .await
        .expect("done");

    // Asking again for what it has needs no owner.
    let again = ask(
        &c.scout_mcp,
        json!({ "crew": "Blog", "bot": "editor", "access": ["talk"], "why": "Again." }),
    )
    .await
    .expect("task")
    .expect("already");
    assert!(again["note"].as_str().expect("note").contains("already"));

    // The owner sees it in the scout's details and takes it back.
    let listed = c
        .app
        .call("crewAccess.list", json!({ "botId": c.scout["id"] }))
        .await
        .expect("list");
    assert_eq!(listed[0]["crewName"], "Blog");
    assert_eq!(listed[0]["targetBotId"], Value::Null);
    let left = c
        .app
        .call("crewAccess.revoke", json!({ "accessId": listed[0]["id"] }))
        .await
        .expect("revoke");
    assert_eq!(left, json!([]));
    let closed = c
        .scout_mcp
        .tool("send_message", note("editor", "Hi"))
        .await
        .expect_err("taken back");
    assert!(closed.contains("cannot reach"), "{closed}");
}
