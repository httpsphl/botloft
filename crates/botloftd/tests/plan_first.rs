//! `show_plan` (spec 10.5): with the option on, a bot shows its plan before
//! long work and the owner has a few seconds to start it now, pick the
//! smaller way, change it or cancel it. With no answer the bot starts.

mod common;

use botloft_core::protocol::BotState;
use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

const TOOL: &str = "mcp__botloft__show_plan";

struct Planner {
    t: TestDaemon,
    app: Client,
    bot: Value,
    mcp: Mcp,
}

/// "Ops" with @writer, who shows plans first when `on`.
async fn planner(on: bool) -> Planner {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (mut bot, first, mut mcp) = ready_bot(&t, &mut app, &crew, "Writer").await;
    if on {
        bot = app
            .call(
                "bots.update",
                json!({ "botId": bot["id"], "planFirst": true }),
            )
            .await
            .expect("update");
        // It starts again to read its new rules.
        let old = first.env("BOTLOFT_BOT_TOKEN");
        let process = loop {
            let process = t.process_of(&bot).await;
            if process.env("BOTLOFT_BOT_TOKEN") != old {
                break process;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        };
        t.until_state(&bot, BotState::Idle).await;
        mcp = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
    }
    Planner { t, app, bot, mcp }
}

fn plan(alternative: Option<&str>) -> Value {
    json!({
        "summary": "Fix the sign-in page",
        "steps": ["Read the form", "Fix the check", "Run the tests"],
        "alternative": alternative,
    })
}

fn show(mcp: &Mcp, arguments: Value) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    tokio::spawn(async move { mcp.tool("show_plan", arguments).await })
}

async fn pending(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = changed["item"]["body"].clone();
        if body["kind"] == "approval" && body["status"] == "pending" {
            assert_eq!(body["toolName"], TOOL);
            return body;
        }
    }
}

async fn answer(app: &mut Client, asked: &Value, mut how: Value) {
    how["approvalId"] = asked["approvalId"].clone();
    app.call("approvals.answer", how).await.expect("answer");
}

#[tokio::test]
async fn the_rules_ask_for_the_plan_only_when_the_owner_wants_it() {
    let p = planner(true).await;
    assert_eq!(p.bot["planFirst"], true);
    let rules = std::fs::read_to_string(
        std::path::Path::new(p.bot["workspace"].as_str().expect("workspace"))
            .join(".claude/rules/botloft.md"),
    )
    .expect("rules");
    assert!(rules.contains("## Show your plan first"));
    drop(p.t);

    let p = planner(false).await;
    assert_eq!(p.bot["planFirst"], false);
    let rules = std::fs::read_to_string(
        std::path::Path::new(p.bot["workspace"].as_str().expect("workspace"))
            .join(".claude/rules/botloft.md"),
    )
    .expect("rules");
    assert!(!rules.contains("show_plan"));
}

#[tokio::test]
async fn with_the_option_off_the_bot_starts_at_once_without_a_card() {
    let mut p = planner(false).await;
    let result = show(&p.mcp, plan(None)).await.expect("task").expect("tool");
    assert_eq!(result["start"], "plan");
    let chat = p
        .app
        .call("chat.history", json!({ "botId": p.bot["id"] }))
        .await
        .expect("chat");
    let cards = chat
        .as_array()
        .expect("items")
        .iter()
        .filter(|item| item["body"]["kind"] == "approval")
        .count();
    assert_eq!(cards, 0);
}

#[tokio::test]
async fn start_now_goes_ahead_and_the_card_holds_the_plan() {
    let mut p = planner(true).await;
    let waiting = show(&p.mcp, plan(None));
    let asked = pending(&mut p.app).await;
    assert_eq!(asked["summary"], "Fix the sign-in page");
    let input: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    assert_eq!(input["steps"][2], "Run the tests");
    assert!(input["starts_at"].as_i64().is_some());
    // A short wait that starts by itself: nothing waits for the owner.
    let id = p.bot["id"].as_str().expect("id").parse().expect("id");
    let state = p.t.daemon.supervisor.status(&id).map(|(state, _)| state);
    assert_ne!(state, Some(BotState::NeedsApproval));

    answer(&mut p.app, &asked, json!({ "allow": true })).await;
    let result = waiting.await.expect("task").expect("tool");
    assert_eq!(result["start"], "plan");
}

#[tokio::test]
async fn the_owner_picks_the_alternative_changes_or_cancels_the_plan() {
    let mut p = planner(true).await;

    let waiting = show(&p.mcp, plan(Some("Only list the problems")));
    let asked = pending(&mut p.app).await;
    let pick = json!({ "allow": true, "input": r#"{"choice":"alternative"}"# });
    answer(&mut p.app, &asked, pick).await;
    let result = waiting.await.expect("task").expect("tool");
    assert_eq!(result["start"], "alternative");

    let waiting = show(&p.mcp, plan(None));
    let asked = pending(&mut p.app).await;
    let wrong = p
        .app
        .call(
            "approvals.answer",
            json!({
                "approvalId": asked["approvalId"],
                "allow": true,
                "input": r#"{"choice":"alternative"}"#,
            }),
        )
        .await;
    assert!(wrong.is_err(), "no alternative to pick");
    let change = json!({ "allow": false, "note": "Start with the tests" });
    answer(&mut p.app, &asked, change).await;
    let result = waiting.await.expect("task").expect("tool");
    assert_eq!(result["start"], false);
    assert_eq!(result["change"], "Start with the tests");

    let waiting = show(&p.mcp, plan(None));
    let asked = pending(&mut p.app).await;
    answer(&mut p.app, &asked, json!({ "allow": false })).await;
    let result = waiting.await.expect("task").expect("tool");
    assert_eq!(result["start"], false);
    assert!(result.get("change").is_none());
}

#[tokio::test]
async fn with_no_answer_the_bot_starts_by_itself() {
    let mut p = planner(true).await;
    let waiting = show(&p.mcp, plan(None));
    let asked = pending(&mut p.app).await;
    let result = waiting.await.expect("task").expect("tool");
    assert_eq!(result["start"], "plan");
    loop {
        let changed = p.app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["approvalId"] == asked["approvalId"] && body["status"] != "pending" {
            assert_eq!(body["status"], "expired");
            break;
        }
    }
}

#[tokio::test]
async fn a_plan_without_steps_is_refused_before_the_owner_sees_it() {
    let p = planner(true).await;
    let err = show(
        &p.mcp,
        json!({ "summary": "Fix it", "steps": ["Only one"] }),
    )
    .await
    .expect("task")
    .expect_err("refused");
    assert!(err.contains("steps"), "{err}");
}
