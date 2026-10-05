//! `change_bot` (spec 10.3): a bot asks to rename or change itself, the
//! chief may ask it for another bot of its crew, and nothing changes until
//! the owner allows it. Bots of other crews cannot be named.

mod common;

use botloft_core::protocol::BotState;
use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

const TOOL: &str = "mcp__botloft__change_bot";

struct Crew {
    _t: TestDaemon,
    app: Client,
    lead: Value,
    lead_mcp: Mcp,
    writer: Value,
    writer_mcp: Mcp,
}

/// "Ops" with @lead as chief and @writer, and "Site" with @scout.
async fn crew() -> Crew {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let ops = app
        .call(
            "crews.create",
            json!({
                "name": "Ops",
                "lead": { "name": "Lead", "role": "Lead role", "instructions": "" },
            }),
        )
        .await
        .expect("crew");
    let site = app
        .call("crews.create", json!({ "name": "Site" }))
        .await
        .expect("crew");
    let lead = app
        .call("bots.list", json!({ "crewId": ops["id"] }))
        .await
        .expect("bots")[0]
        .clone();
    let process = t.process_of(&lead).await;
    t.until_state(&lead, BotState::Idle).await;
    let lead_mcp = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
    let (writer, _, writer_mcp) = ready_bot(&t, &mut app, &ops, "Writer").await;
    ready_bot(&t, &mut app, &site, "Scout").await;
    Crew {
        _t: t,
        app,
        lead,
        lead_mcp,
        writer,
        writer_mcp,
    }
}

fn call(mcp: &Mcp, arguments: Value) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    tokio::spawn(async move { mcp.tool("change_bot", arguments).await })
}

/// The pending request, with the bot whose chat it is in as `botId`.
async fn pending(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let mut body = changed["item"]["body"].clone();
        if body["kind"] == "approval" && body["status"] == "pending" {
            assert_eq!(body["toolName"], TOOL);
            body["botId"] = changed["item"]["botId"].clone();
            return body;
        }
    }
}

async fn bot(app: &mut Client, of: &Value) -> Value {
    let bots = app.call("bots.list", json!({})).await.expect("bots");
    bots.as_array()
        .expect("list")
        .iter()
        .find(|bot| bot["id"] == of["id"])
        .expect("bot")
        .clone()
}

#[tokio::test]
async fn a_bot_renames_itself_once_the_owner_allows_it() {
    let mut c = crew().await;
    let waiting = call(
        &c.writer_mcp,
        json!({ "name": "Editor", "reason": "Asked to." }),
    );
    let asked = pending(&mut c.app).await;
    assert_eq!(asked["botId"], c.writer["id"]);
    assert_eq!(asked["summary"], "Writer");
    let input: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    assert_eq!(input["before"]["name"], "Writer");
    assert_eq!(input["after"]["name"], "Editor");
    assert_eq!(input["after"]["role"], "Writer role");
    assert_eq!(bot(&mut c.app, &c.writer).await["name"], "Writer");

    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    let done = waiting.await.expect("task").expect("changed");
    assert_eq!(done["done"], true);
    assert_eq!(done["handle"], "editor");
    let now = bot(&mut c.app, &c.writer).await;
    assert_eq!(now["name"], "Editor");
    assert_eq!(now["handle"], "editor");
}

#[tokio::test]
async fn the_chief_changes_a_bot_of_its_crew_and_the_owner_may_say_no() {
    let mut c = crew().await;
    let waiting = call(
        &c.lead_mcp,
        json!({ "bot": "@writer", "role": "Writes the newsletter" }),
    );
    let asked = pending(&mut c.app).await;
    // The request is in the chief's chat, about the writer.
    assert_eq!(asked["botId"], c.lead["id"]);
    let input: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    assert_eq!(input["bot_id"], c.writer["id"]);
    assert_eq!(input["after"]["role"], "Writes the newsletter");

    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": false, "note": "Not now" }),
        )
        .await
        .expect("answer");
    let declined = waiting.await.expect("task").expect("declined");
    assert_eq!(declined["done"], false);
    assert!(declined["note"].as_str().expect("note").contains("Not now"));
    assert_eq!(bot(&mut c.app, &c.writer).await["role"], "Writer role");

    let waiting = call(&c.lead_mcp, json!({ "bot": "writer", "role": "Edits" }));
    let asked = pending(&mut c.app).await;
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    waiting.await.expect("task").expect("changed");
    assert_eq!(bot(&mut c.app, &c.writer).await["role"], "Edits");
}

#[tokio::test]
async fn only_the_chief_names_another_bot_and_only_of_its_crew() {
    let c = crew().await;
    let refused = call(&c.writer_mcp, json!({ "bot": "lead", "name": "Boss" }))
        .await
        .expect("task");
    assert!(
        refused
            .expect_err("not chief")
            .contains("only the crew's chief")
    );

    // A bot of another crew is as good as missing.
    let refused = call(&c.lead_mcp, json!({ "bot": "scout", "name": "Spy" }))
        .await
        .expect("task");
    assert!(
        refused
            .expect_err("other crew")
            .contains("no bot in your crew")
    );

    // A taken name and no change at all never reach the owner.
    let refused = call(&c.writer_mcp, json!({ "name": "Lead" }))
        .await
        .expect("task");
    assert!(refused.is_err());
    let refused = call(&c.writer_mcp, json!({ "name": "Writer" }))
        .await
        .expect("task");
    assert!(refused.expect_err("same").contains("already"));
}

#[tokio::test]
async fn the_chief_lowers_a_bots_model_and_effort_once_the_owner_allows_it() {
    let mut c = crew().await;
    let roster = c
        .lead_mcp
        .tool("crew_roster", json!({}))
        .await
        .expect("roster");
    assert_eq!(roster["bots"][0]["model"], "default");
    assert_eq!(roster["bots"][0]["effort"], "default");

    let refused = call(&c.lead_mcp, json!({ "bot": "writer", "effort": "tiny" }))
        .await
        .expect("task");
    assert!(refused.expect_err("bad effort").contains("effort must be"));

    let waiting = call(
        &c.lead_mcp,
        json!({ "bot": "writer", "model": "haiku", "effort": "low", "reason": "Simple work." }),
    );
    let asked = pending(&mut c.app).await;
    let input: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    assert_eq!(input["before"]["model"], "default");
    assert_eq!(input["after"]["model"], "haiku");
    assert_eq!(input["after"]["effort"], "low");
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    let done = waiting.await.expect("task").expect("changed");
    assert_eq!(done["model"], "haiku");
    let now = bot(&mut c.app, &c.writer).await;
    assert_eq!(now["model"], "haiku");
    assert_eq!(now["effort"], "low");
    assert_eq!(now["name"], "Writer");
}
