//! Bots changing and deleting their own routines (spec 20.12): they see
//! theirs with `my_routines`, and `change_routine` and `delete_routine`
//! wait for the owner, who may adjust a change before allowing it.

mod common;

use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

struct Crew {
    _t: TestDaemon,
    app: Client,
    bot: Value,
    mcp: Mcp,
    routine: Value,
    other: Value,
}

async fn crew() -> Crew {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Office" }))
        .await
        .expect("crew");
    let (bot, _, mcp) = ready_bot(&t, &mut app, &crew, "Mail").await;
    let (writer, _, _) = ready_bot(&t, &mut app, &crew, "Writer").await;
    let create = |bot: &Value, name: &str| {
        json!({
            "botId": bot["id"], "name": name, "prompt": "Read the new mail.",
            "schedule": { "kind": "weekly", "days": [1, 2, 3, 4, 5], "time": "08:00" },
            "timezone": "America/Sao_Paulo",
        })
    };
    let routine = app
        .call("routines.create", create(&bot, "Morning inbox"))
        .await
        .expect("routine");
    let other = app
        .call("routines.create", create(&writer, "Weekly report"))
        .await
        .expect("other");
    Crew {
        _t: t,
        app,
        bot,
        mcp,
        routine,
        other,
    }
}

fn call(mcp: &Mcp, tool: &'static str, arguments: Value) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    tokio::spawn(async move { mcp.tool(tool, arguments).await })
}

async fn pending(app: &mut Client, tool: &str) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            assert_eq!(body["toolName"], tool);
            return body.clone();
        }
    }
}

async fn answer(app: &mut Client, asked: &Value, extra: Value) {
    let mut params = json!({ "approvalId": asked["approvalId"] });
    for (key, value) in extra.as_object().expect("object") {
        params[key] = value.clone();
    }
    app.call("approvals.answer", params).await.expect("answer");
}

async fn routines(app: &mut Client, bot: &Value) -> Vec<Value> {
    app.call("routines.list", json!({ "botId": bot["id"] }))
        .await
        .expect("routines")
        .as_array()
        .expect("list")
        .clone()
}

#[tokio::test]
async fn a_bot_sees_its_own_routines_only() {
    let c = crew().await;
    let mine = call(&c.mcp, "my_routines", json!({}))
        .await
        .expect("task")
        .expect("list");
    let listed = mine["routines"].as_array().expect("routines");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["routine_id"], c.routine["id"]);
    assert_eq!(listed[0]["enabled"], true);

    // Another bot's routine is as good as missing; no change is no request.
    let theirs = json!({ "routine_id": c.other["id"], "enabled": false });
    let refused = call(&c.mcp, "change_routine", theirs).await.expect("task");
    assert!(refused.expect_err("not mine").contains("my_routines"));
    let same = json!({ "routine_id": c.routine["id"], "name": "Morning inbox" });
    let refused = call(&c.mcp, "change_routine", same).await.expect("task");
    assert!(refused.expect_err("no change").contains("already"));
}

#[tokio::test]
async fn the_owner_adjusts_a_change_before_allowing_it() {
    let mut c = crew().await;
    let change = json!({ "routine_id": c.routine["id"], "name": "Inbox", "enabled": false });
    let waiting = call(&c.mcp, "change_routine", change);
    let asked = pending(&mut c.app, "mcp__botloft__change_routine").await;
    assert_eq!(asked["summary"], "Morning inbox");
    let input: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    assert_eq!(input["before"]["enabled"], true);
    assert_eq!(input["after"]["name"], "Inbox");

    // The owner keeps it on, under another name.
    let mut adjusted = input["after"].clone();
    adjusted["name"] = json!("Inbox at eight");
    adjusted["enabled"] = json!(true);
    answer(
        &mut c.app,
        &asked,
        json!({ "allow": true, "input": adjusted.to_string() }),
    )
    .await;
    let done = waiting.await.expect("task").expect("changed");
    assert_eq!(done["done"], true);
    assert_eq!(done["name"], "Inbox at eight");
    let list = routines(&mut c.app, &c.bot).await;
    assert_eq!(list[0]["name"], "Inbox at eight");
    assert_eq!(list[0]["enabled"], true);
}

#[tokio::test]
async fn deleting_waits_for_the_owner_who_may_say_no() {
    let mut c = crew().await;
    let delete = json!({ "routine_id": c.routine["id"], "reason": "The client left." });
    let waiting = call(&c.mcp, "delete_routine", delete.clone());
    let asked = pending(&mut c.app, "mcp__botloft__delete_routine").await;
    answer(
        &mut c.app,
        &asked,
        json!({ "allow": false, "note": "Keep it" }),
    )
    .await;
    let kept = waiting.await.expect("task").expect("answer");
    assert_eq!(kept["done"], false);
    assert!(kept["note"].as_str().expect("note").contains("Keep it"));
    assert_eq!(routines(&mut c.app, &c.bot).await.len(), 1);

    let waiting = call(&c.mcp, "delete_routine", delete);
    let asked = pending(&mut c.app, "mcp__botloft__delete_routine").await;
    let input: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    assert_eq!(input["reason"], "The client left.");
    answer(&mut c.app, &asked, json!({ "allow": true })).await;
    let gone = waiting.await.expect("task").expect("deleted");
    assert_eq!(gone["done"], true);
    assert!(routines(&mut c.app, &c.bot).await.is_empty());
}
