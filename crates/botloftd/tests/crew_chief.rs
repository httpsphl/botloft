//! A crew's chief (spec 10.2): created with the crew, the only bot that can
//! suggest new ones, and the owner's say over each suggestion.

mod common;

use std::path::Path;

use botloft_core::protocol::BotState;
use botloftd::runtime::fake::FakeProcess;
use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

struct Chief {
    t: TestDaemon,
    app: Client,
    crew: Value,
    chief: Value,
    mcp: Mcp,
}

/// A crew "Site" created with its chief, whose process is ready.
async fn crew_with_chief() -> Chief {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call(
            "crews.create",
            json!({
                "name": "Site",
                "lead": { "name": "Chefe", "role": "Leads the crew", "instructions": "Build the bakery's site" },
            }),
        )
        .await
        .expect("crew");
    let bots = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    let chief = bots[0].clone();
    assert_eq!(crew["leadBotId"], chief["id"]);
    let (process, mcp) = ready(&t, &chief).await;
    drop(process);
    Chief {
        t,
        app,
        crew,
        chief,
        mcp,
    }
}

async fn ready(t: &TestDaemon, bot: &Value) -> (FakeProcess, Mcp) {
    let process = t.process_of(bot).await;
    t.until_state(bot, BotState::Idle).await;
    let mcp = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
    (process, mcp)
}

fn designer() -> Value {
    json!({
        "name": "Designer",
        "role": "Draws each page",
        "instructions": "Design the pages in shared/design and tell me when each is ready.",
        "model": "sonnet",
        "reason": "The site needs a look before anyone builds it.",
    })
}

fn suggest(mcp: &Mcp, arguments: Value) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    tokio::spawn(async move { mcp.tool("suggest_bot", arguments).await })
}

/// Waits for the suggestion to show up in the chief's chat.
async fn pending_suggestion(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            assert_eq!(body["toolName"], "mcp__botloft__suggest_bot");
            return body.clone();
        }
    }
}

async fn crew_bots(c: &mut Chief) -> Vec<Value> {
    c.app
        .call("bots.list", json!({ "crewId": c.crew["id"] }))
        .await
        .expect("bots")
        .as_array()
        .expect("list")
        .clone()
}

fn rules(bot: &Value) -> String {
    let workspace = bot["workspace"].as_str().expect("workspace");
    std::fs::read_to_string(Path::new(workspace).join(".claude/rules/botloft.md")).expect("rules")
}

#[tokio::test]
async fn a_new_crew_comes_with_its_chief() {
    let mut c = crew_with_chief().await;
    assert_eq!(c.chief["name"], "Chefe");
    assert!(rules(&c.chief).contains("## You lead this crew"));
    assert!(rules(&c.chief).contains("Build the bakery's site"));

    let roster = c.mcp.tool("crew_roster", json!({})).await.expect("roster");
    assert_eq!(roster["you_lead"], true);
    let (_, _, mut writer) = ready_bot(&c.t, &mut c.app, &c.crew, "Writer").await;
    let seen = writer.tool("crew_roster", json!({})).await.expect("roster");
    assert_eq!(seen["you_lead"], false);
    assert_eq!(seen["bots"][0]["chief"], true);
}

#[tokio::test]
async fn the_owner_approves_a_suggested_bot() {
    let mut c = crew_with_chief().await;
    let call = suggest(&c.mcp, designer());
    let asked = pending_suggestion(&mut c.app).await;
    assert_eq!(asked["summary"], "Designer");
    c.t.until_state(&c.chief, BotState::NeedsApproval).await;

    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    let created = call.await.expect("task").expect("created");
    assert_eq!(created["created"], true);
    assert_eq!(created["handle"], "designer");
    assert!(
        created["note"]
            .as_str()
            .expect("note")
            .contains("send_message")
    );

    let bots = crew_bots(&mut c).await;
    let new = bots
        .iter()
        .find(|bot| bot["handle"] == "designer")
        .expect("designer");
    assert_eq!(new["model"], "sonnet");
    assert!(!rules(new).contains("You lead this crew"));
    c.t.process_of(new).await;
}

#[tokio::test]
async fn the_owner_can_change_a_suggestion_before_approving_it() {
    let mut c = crew_with_chief().await;
    let call = suggest(&c.mcp, designer());
    let asked = pending_suggestion(&mut c.app).await;

    let mut changed = designer();
    changed["name"] = json!("Illustrator");
    changed["model"] = json!("haiku");
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true, "input": changed.to_string() }),
        )
        .await
        .expect("answer");
    let created = call.await.expect("task").expect("created");
    assert_eq!(created["handle"], "illustrator");
    assert_eq!(created["model"], "haiku");
    assert!(created["note"].as_str().expect("note").contains("changed"));
    // The card shows what was created.
    let item = loop {
        let changed = c.app.notification("chat.item").await;
        if changed["item"]["body"]["status"] == "allowed" {
            break changed["item"]["body"].clone();
        }
    };
    assert!(
        item["input"]
            .as_str()
            .expect("input")
            .contains("Illustrator")
    );
}

#[tokio::test]
async fn a_declined_suggestion_creates_nothing() {
    let mut c = crew_with_chief().await;
    let call = suggest(&c.mcp, designer());
    let asked = pending_suggestion(&mut c.app).await;
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": false, "note": "Use the writer" }),
        )
        .await
        .expect("answer");
    let declined = call.await.expect("task").expect("answered");
    assert_eq!(declined["created"], false);
    assert!(
        declined["note"]
            .as_str()
            .expect("note")
            .contains("Use the writer")
    );
    assert_eq!(crew_bots(&mut c).await.len(), 1);
}

#[tokio::test]
async fn a_chief_that_bypasses_permissions_creates_at_once() {
    let mut c = crew_with_chief().await;
    c.app
        .call(
            "bots.setPermissionMode",
            json!({ "botId": c.chief["id"], "mode": "bypass_permissions" }),
        )
        .await
        .expect("mode");
    // The chief restarts into the new mode, with a new token.
    let old = c.mcp.token.clone();
    let mcp = loop {
        let (_, mcp) = ready(&c.t, &c.chief).await;
        if mcp.token != old {
            break mcp;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
    let created = suggest(&mcp, designer())
        .await
        .expect("task")
        .expect("created");
    assert_eq!(created["created"], true);
    assert!(
        created["note"]
            .as_str()
            .expect("note")
            .contains("without asking")
    );
    assert_eq!(crew_bots(&mut c).await.len(), 2);
}

#[tokio::test]
async fn only_the_chief_suggests_and_only_while_the_crew_has_room() {
    let mut c = crew_with_chief().await;
    let (_, _, writer) = ready_bot(&c.t, &mut c.app, &c.crew, "Writer").await;
    let refused = suggest(&writer, designer()).await.expect("task");
    assert!(
        refused
            .expect_err("refused")
            .contains("only the crew's chief")
    );

    // The tests allow three bots per crew.
    ready_bot(&c.t, &mut c.app, &c.crew, "Reviewer").await;
    let full = suggest(&c.mcp, designer()).await.expect("task");
    assert!(full.expect_err("full").contains("already has 3 bots"));

    let mut nameless = designer();
    nameless["reason"] = json!("  ");
    let refused = suggest(&c.mcp, nameless).await.expect("task");
    assert!(refused.expect_err("no reason").contains("reason"));
}

#[tokio::test]
async fn the_owner_moves_the_chief_and_an_archived_chief_leaves_none() {
    let mut c = crew_with_chief().await;
    let (writer, _, _) = ready_bot(&c.t, &mut c.app, &c.crew, "Writer").await;
    let crew = c
        .app
        .call(
            "crews.setLead",
            json!({ "crewId": c.crew["id"], "botId": writer["id"] }),
        )
        .await
        .expect("move");
    assert_eq!(crew["leadBotId"], writer["id"]);
    assert!(rules(&writer).contains("## You lead this crew"));
    assert!(!rules(&c.chief).contains("## You lead this crew"));

    c.app
        .call("bots.archive", json!({ "botId": writer["id"] }))
        .await
        .expect("archive");
    let crews = c.app.call("crews.list", json!({})).await.expect("crews");
    assert_eq!(crews[0]["leadBotId"], Value::Null);
}

#[tokio::test]
async fn a_suggested_bot_starts_with_the_effort_suggested() {
    let mut c = crew_with_chief().await;
    let mut asked = designer();
    asked["effort"] = json!("loud");
    let refused = suggest(&c.mcp, asked.clone()).await.expect("task");
    assert!(refused.expect_err("bad effort").contains("effort must be"));

    asked["effort"] = json!("low");
    let waiting = suggest(&c.mcp, asked);
    let request = pending_suggestion(&mut c.app).await;
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": request["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    let created = waiting.await.expect("task").expect("created");
    assert_eq!(created["effort"], "low");
    let designer = crew_bots(&mut c)
        .await
        .into_iter()
        .find(|bot| bot["name"] == "Designer")
        .expect("designer");
    assert_eq!(designer["effort"], "low");
}
