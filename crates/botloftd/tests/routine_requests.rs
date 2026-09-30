//! Bots asking for routines (spec 20.12): `schedule_routine` waits for the
//! owner, who may change the routine before it is created, and a routine
//! can be for another bot of the crew.

mod common;

use botloft_core::protocol::BotState;
use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

struct Crew {
    t: TestDaemon,
    app: Client,
    crew: Value,
    bot: Value,
    mcp: Mcp,
}

async fn crew_with_bot() -> Crew {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Office" }))
        .await
        .expect("crew");
    let (bot, _, mcp) = ready_bot(&t, &mut app, &crew, "Mail").await;
    Crew {
        t,
        app,
        crew,
        bot,
        mcp,
    }
}

fn inbox() -> Value {
    json!({
        "name": "Morning inbox",
        "prompt": "Read the new mail and summarize what is urgent.",
        "schedule": { "kind": "weekly", "days": [1, 2, 3, 4, 5], "time": "08:00" },
        "timezone": "America/Sao_Paulo",
    })
}

fn schedule(mcp: &Mcp, arguments: Value) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    tokio::spawn(async move { mcp.tool("schedule_routine", arguments).await })
}

/// Waits for the request to show up in the bot's chat.
async fn pending_request(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            assert_eq!(body["toolName"], "mcp__botloft__schedule_routine");
            return body.clone();
        }
    }
}

async fn routines_of(app: &mut Client, bot: &Value) -> Vec<Value> {
    app.call("routines.list", json!({ "botId": bot["id"] }))
        .await
        .expect("routines")
        .as_array()
        .expect("list")
        .clone()
}

#[tokio::test]
async fn the_owner_approves_a_routine_and_it_shows_in_the_list() {
    let mut c = crew_with_bot().await;
    let call = schedule(&c.mcp, inbox());
    let asked = pending_request(&mut c.app).await;
    assert_eq!(asked["summary"], "Morning inbox");
    c.t.until_state(&c.bot, BotState::NeedsApproval).await;
    assert!(routines_of(&mut c.app, &c.bot).await.is_empty());

    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    let created = call.await.expect("task").expect("created");
    assert_eq!(created["created"], true);
    assert!(
        created["note"]
            .as_str()
            .expect("note")
            .contains("(America/Sao_Paulo)")
    );

    let list = routines_of(&mut c.app, &c.bot).await;
    assert_eq!(list.len(), 1);
    assert_eq!(list[0]["id"], created["routine_id"]);
    assert_eq!(list[0]["name"], "Morning inbox");
    assert_eq!(list[0]["timezone"], "America/Sao_Paulo");
    assert_eq!(list[0]["schedule"]["time"], "08:00");
    assert!(list[0]["nextRunAt"].is_i64());
}

#[tokio::test]
async fn the_owner_can_change_the_routine_before_approving_it() {
    let mut c = crew_with_bot().await;
    let call = schedule(&c.mcp, inbox());
    let asked = pending_request(&mut c.app).await;

    // Too often: refused, and the request stays open.
    let mut changed = inbox();
    changed["schedule"] = json!({ "kind": "interval", "minutes": 2 });
    let refused = c
        .app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true, "input": changed.to_string() }),
        )
        .await
        .expect_err("too often");
    assert!(refused.message.contains("5 minutes"), "{}", refused.message);

    changed["name"] = json!("Inbox at nine");
    changed["schedule"] = json!({ "kind": "weekly", "days": [1, 3], "time": "09:00" });
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true, "input": changed.to_string() }),
        )
        .await
        .expect("answer");
    let created = call.await.expect("task").expect("created");
    assert!(created["note"].as_str().expect("note").contains("changed"));
    let list = routines_of(&mut c.app, &c.bot).await;
    assert_eq!(list[0]["name"], "Inbox at nine");
    assert_eq!(list[0]["schedule"]["days"], json!([1, 3]));
}

#[tokio::test]
async fn a_declined_routine_is_not_created() {
    let mut c = crew_with_bot().await;
    let call = schedule(&c.mcp, inbox());
    let asked = pending_request(&mut c.app).await;
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": false, "note": "I check it myself" }),
        )
        .await
        .expect("answer");
    let declined = call.await.expect("task").expect("answered");
    assert_eq!(declined["created"], false);
    let note = declined["note"].as_str().expect("note");
    assert!(note.contains("I check it myself"));
    assert!(note.contains("Nothing was scheduled"));
    assert!(routines_of(&mut c.app, &c.bot).await.is_empty());
}

#[tokio::test]
async fn a_routine_can_be_for_another_bot_of_the_crew() {
    let mut c = crew_with_bot().await;
    let (sales, _, _) = ready_bot(&c.t, &mut c.app, &c.crew, "Sales").await;
    let mut arguments = inbox();
    arguments["bot"] = json!("@sales");
    let call = schedule(&c.mcp, arguments);
    // The request waits in the chat of the bot that asked.
    let asked = pending_request(&mut c.app).await;
    assert_eq!(
        asked["input"]
            .as_str()
            .map(|input| input.contains("\"bot\":\"sales\"")),
        Some(true)
    );
    c.t.until_state(&c.bot, BotState::NeedsApproval).await;
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    call.await.expect("task").expect("created");
    assert!(routines_of(&mut c.app, &c.bot).await.is_empty());
    assert_eq!(routines_of(&mut c.app, &sales).await.len(), 1);

    let mut stranger = inbox();
    stranger["bot"] = json!("nobody");
    let refused = schedule(&c.mcp, stranger).await.expect("task");
    assert!(refused.expect_err("unknown").contains("@nobody"));
}

#[tokio::test]
async fn an_impossible_routine_never_reaches_the_owner() {
    let c = crew_with_bot().await;
    let mut often = inbox();
    often["schedule"] = json!({ "kind": "cron", "expr": "* * * * *" });
    let refused = schedule(&c.mcp, often).await.expect("task");
    assert!(refused.expect_err("too often").contains("5 minutes"));

    let mut zone = inbox();
    zone["timezone"] = json!("Mars/Olympus");
    let refused = schedule(&c.mcp, zone).await.expect("task");
    assert!(refused.is_err());

    let mut unknown = inbox();
    unknown["when"] = json!("tomorrow");
    let refused = schedule(&c.mcp, unknown).await.expect("task");
    assert!(
        refused
            .expect_err("unknown field")
            .contains("invalid arguments")
    );
}

#[tokio::test]
async fn a_bot_that_bypasses_permissions_creates_at_once() {
    let mut c = crew_with_bot().await;
    c.app
        .call(
            "bots.setPermissionMode",
            json!({ "botId": c.bot["id"], "mode": "bypass_permissions" }),
        )
        .await
        .expect("mode");
    // The bot restarts into the new mode, with a new token.
    let old = c.mcp.token.clone();
    let mcp = loop {
        let process = c.t.process_of(&c.bot).await;
        c.t.until_state(&c.bot, BotState::Idle).await;
        let token = process.env("BOTLOFT_BOT_TOKEN").expect("token");
        if token != old {
            break Mcp::new(c.t.addr, token);
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
    let mut arguments = inbox();
    arguments
        .as_object_mut()
        .expect("object")
        .remove("timezone");
    let created = schedule(&mcp, arguments)
        .await
        .expect("task")
        .expect("created");
    assert!(
        created["note"]
            .as_str()
            .expect("note")
            .contains("without asking")
    );
    // The computer's zone when the bot names none.
    assert!(
        created["timezone"]
            .as_str()
            .is_some_and(|zone| !zone.is_empty())
    );
    assert_eq!(routines_of(&mut c.app, &c.bot).await.len(), 1);
}
