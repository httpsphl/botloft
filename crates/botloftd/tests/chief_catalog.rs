//! The chief and the bot catalog (spec 26.4): reading the roles, and
//! suggesting a bot from one.

mod common;

use botloft_core::protocol::BotState;
use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::task::JoinHandle;

struct Chief {
    t: TestDaemon,
    app: Client,
    crew: Value,
    mcp: Mcp,
}

async fn crew_with_chief() -> Chief {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call(
            "crews.create",
            json!({
                "name": "Site",
                "lead": { "name": "Chefe", "role": "Leads the crew", "instructions": "Build the site" },
            }),
        )
        .await
        .expect("crew");
    let bots = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    let chief = bots[0].clone();
    let process = t.process_of(&chief).await;
    t.until_state(&chief, BotState::Idle).await;
    let mcp = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
    Chief { t, app, crew, mcp }
}

fn suggest(mcp: &Mcp, arguments: Value) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    tokio::spawn(async move { mcp.tool("suggest_bot", arguments).await })
}

async fn pending_suggestion(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            return body.clone();
        }
    }
}

async fn approve(app: &mut Client, asked: &Value) {
    app.call(
        "approvals.answer",
        json!({ "approvalId": asked["approvalId"], "allow": true }),
    )
    .await
    .expect("answer");
}

async fn sheet(app: &mut Client, id: &str) -> Value {
    app.call("catalog.get", json!({ "id": id }))
        .await
        .expect("sheet")
}

#[tokio::test]
async fn the_chief_reads_the_catalog() {
    let mut c = crew_with_chief().await;

    let all = c
        .mcp
        .tool("list_bot_templates", json!({}))
        .await
        .expect("list");
    let all = all["templates"].as_array().expect("templates");
    assert_eq!(all.len(), 58);
    assert!(all[0].get("instructions").is_none(), "the list is light");

    let research = c
        .mcp
        .tool("list_bot_templates", json!({ "category": "research" }))
        .await
        .expect("list");
    assert_eq!(
        research["templates"].as_array().expect("templates").len(),
        4
    );

    let designer = c
        .mcp
        .tool("get_bot_template", json!({ "id": "designer" }))
        .await
        .expect("get");
    assert_eq!(designer, sheet(&mut c.app, "designer").await);

    let unknown = c
        .mcp
        .tool("get_bot_template", json!({ "id": "wizard" }))
        .await;
    assert!(unknown.expect_err("unknown").contains("list_bot_templates"));
}

#[tokio::test]
async fn only_the_chief_reads_the_catalog() {
    let mut c = crew_with_chief().await;
    let (_, _, mut writer) = ready_bot(&c.t, &mut c.app, &c.crew, "Writer").await;
    for (tool, args) in [
        ("list_bot_templates", json!({})),
        ("get_bot_template", json!({ "id": "designer" })),
    ] {
        let refused = writer.tool(tool, args).await;
        assert!(
            refused
                .expect_err("refused")
                .contains("only the crew's chief")
        );
    }
}

#[tokio::test]
async fn a_suggestion_from_a_role_carries_its_instructions_to_the_owner() {
    let mut c = crew_with_chief().await;
    let designer = sheet(&mut c.app, "designer").await;
    let call = suggest(
        &c.mcp,
        json!({ "name": "Designer", "template": "designer", "reason": "The site needs a look." }),
    );

    let asked = pending_suggestion(&mut c.app).await;
    let shown: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    assert_eq!(shown["instructions"], designer["instructions"]);
    assert_eq!(shown["role"], designer["role"]);
    assert_eq!(shown["effort"], designer["effort"]);
    assert!(
        shown.get("template").is_none(),
        "the card is a plain suggestion"
    );

    approve(&mut c.app, &asked).await;
    let created = call.await.expect("task").expect("created");
    assert_eq!(created["created"], true);
    assert_eq!(created["handle"], "designer");

    let bots = c
        .app
        .call("bots.list", json!({ "crewId": c.crew["id"] }))
        .await
        .expect("bots");
    let new = bots
        .as_array()
        .expect("bots")
        .iter()
        .find(|bot| bot["handle"] == "designer")
        .expect("designer");
    assert_eq!(new["instructions"], designer["instructions"]);
    assert_eq!(new["permissionMode"], "default");
}

#[tokio::test]
async fn what_the_chief_writes_goes_after_the_roles_instructions() {
    let mut c = crew_with_chief().await;
    let reviewer = sheet(&mut c.app, "code-reviewer").await;
    let call = suggest(
        &c.mcp,
        json!({
            "name": "Reviewer",
            "template": "code-reviewer",
            "instructions": "Review the bakery site's pull requests.",
            "model": "sonnet",
            "reason": "Someone must check the code.",
        }),
    );

    let asked = pending_suggestion(&mut c.app).await;
    let shown: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    let text = shown["instructions"].as_str().expect("instructions");
    assert!(text.starts_with(reviewer["instructions"].as_str().expect("sheet")));
    assert!(text.ends_with("\n\nFor this crew:\nReview the bakery site's pull requests."));
    assert_eq!(shown["model"], "sonnet", "what the chief chose wins");
    assert_eq!(shown["effort"], "high", "the role's effort fills the gap");

    approve(&mut c.app, &asked).await;
    let created = call.await.expect("task").expect("created");
    assert_eq!(created["model"], "sonnet");
    assert_eq!(created["effort"], "high");
}

#[tokio::test]
async fn a_suggestion_the_owner_could_not_use_never_reaches_them() {
    let c = crew_with_chief().await;
    let reason = "Because.";

    let unknown = suggest(
        &c.mcp,
        json!({ "name": "Wizard", "template": "wizard", "reason": reason }),
    )
    .await
    .expect("task");
    assert!(
        unknown
            .expect_err("unknown")
            .contains("no bot template named wizard")
    );

    let bare = suggest(&c.mcp, json!({ "name": "Wizard", "reason": reason }))
        .await
        .expect("task");
    assert!(
        bare.expect_err("bare")
            .contains("unless you give a template")
    );

    let long = "x".repeat(6_500);
    let too_long = suggest(
        &c.mcp,
        json!({ "name": "Designer", "template": "designer", "instructions": long, "reason": reason }),
    )
    .await
    .expect("task");
    assert!(too_long.expect_err("too long").contains("at most 8000"));
}
