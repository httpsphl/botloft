//! The Maker (spec 36.2): always there, alone in its crew, and the only bot
//! that builds crews.

mod common;

use std::path::Path;

use common::bots::ready_bot;
use common::maker::{blog_plan, maker};
use serde_json::json;

#[tokio::test]
async fn the_maker_is_made_once_and_works_alone() {
    let mut m = maker().await;
    let again = botloftd::service::maker::ensure(&m.t.daemon).expect("again");
    assert_eq!(again.as_str(), m.bot["id"]);
    assert_eq!(m.bot["handle"], "maker");
    assert_eq!(m.bot["model"], "opus");
    let workspace = m.bot["workspace"].as_str().expect("workspace");
    let rules = std::fs::read_to_string(Path::new(workspace).join(".claude/rules/botloft.md"))
        .expect("rules");
    assert!(rules.contains("## You are the Maker"));
    assert!(!rules.contains("You lead this crew"));

    let bot = json!({ "botId": m.bot["id"] });
    let crew = json!({ "crewId": m.bot["crewId"] });
    for (method, params) in [
        ("bots.archive", bot.clone()),
        (
            "bots.update",
            json!({ "botId": m.bot["id"], "name": "Builder" }),
        ),
        ("bots.delete", bot.clone()),
        ("crews.archive", crew.clone()),
        ("crews.delete", crew.clone()),
        (
            "crews.setLead",
            json!({ "crewId": m.bot["crewId"], "botId": m.bot["id"] }),
        ),
        (
            "crews.setAutoMode",
            json!({ "crewId": m.bot["crewId"], "enabled": true }),
        ),
        (
            "bots.setPermissionMode",
            json!({ "botId": m.bot["id"], "mode": "bypass_permissions" }),
        ),
        (
            "bots.create",
            json!({ "crewId": m.bot["crewId"], "name": "Helper", "role": "Helps", "instructions": "" }),
        ),
        (
            "catalog.add",
            json!({ "crewId": m.bot["crewId"], "templateId": "writer", "name": "Writer", "role": "Writes" }),
        ),
    ] {
        let refused = m.app.call(method, params).await;
        assert!(refused.is_err(), "{method} should be refused");
    }
    let catalog = m.mcp.tool("list_bot_templates", json!({})).await;
    assert!(catalog.is_ok(), "the Maker reads the catalog");

    // Its name follows the app's language; its handle stays.
    m.app
        .call("session.setLocale", json!({ "locale": "pt-BR" }))
        .await
        .expect("locale");
    let bots = m
        .app
        .call("bots.list", json!({ "crewId": m.bot["crewId"] }))
        .await
        .expect("bots");
    assert_eq!(bots[0]["name"], "O Criador");
    assert_eq!(bots[0]["handle"], "maker");
    assert_eq!(bots[0]["color"], "#FF7A59");
    // Asking the owner is fine; bypassing is not.
    m.app
        .call(
            "bots.setPermissionMode",
            json!({ "botId": m.bot["id"], "mode": "accept_edits" }),
        )
        .await
        .expect("a mode that asks");
}

#[tokio::test]
async fn only_the_maker_builds_and_no_one_reaches_its_crew() {
    let mut m = maker().await;
    let crew = m
        .app
        .call(
            "crews.create",
            json!({ "name": "Shop", "lead": { "name": "Chief", "role": "Leads", "instructions": "Sell" } }),
        )
        .await
        .expect("crew");
    let (_, _, mut other) = ready_bot(&m.t, &mut m.app, &crew, "Clerk").await;
    let refused = other
        .tool("build", blog_plan())
        .await
        .expect_err("not the Maker");
    assert!(refused.contains("only the Maker"), "{refused}");

    let reach = other
        .tool(
            "ask_crew_access",
            json!({ "crew": "Maker", "access": ["talk"], "why": "To ask it things" }),
        )
        .await
        .expect_err("hidden");
    assert!(reach.contains("no crew called"), "{reach}");
}

#[tokio::test]
async fn an_impossible_plan_never_reaches_the_owner() {
    let mut m = maker().await;
    let mut plan = blog_plan();
    plan["steps"][1]["crew"] = json!("shop");
    let refused = m.mcp.tool("build", plan).await.expect_err("unknown ref");
    assert!(refused.contains("\"shop\""), "{refused}");

    let mut plan = blog_plan();
    plan["steps"][2]["agent"] = json!("Designer");
    let refused = m.mcp.tool("build", plan).await.expect_err("unknown bot");
    assert!(refused.contains("Designer"), "{refused}");

    let mut plan = blog_plan();
    plan["steps"][1]["template"] = json!("no-such-role");
    assert!(m.mcp.tool("build", plan).await.is_err());

    let pending =
        m.t.daemon
            .store()
            .pending_approvals(&m.bot["id"].as_str().expect("id").parse().expect("bot id"))
            .expect("pending");
    assert!(pending.is_empty(), "no card for an impossible plan");
}
