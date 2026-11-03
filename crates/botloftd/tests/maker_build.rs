//! `build` (spec 36.4): the Maker's plan, the owner's card, building it whole
//! and undoing it.

mod common;

use std::path::Path;

use botloft_core::protocol::BotState;
use common::maker::{blog_plan, build, crew_names, maker, pending_build};
use serde_json::{Value, json};

async fn crew_named(app: &mut common::Client, name: &str) -> Value {
    let crews = app.call("crews.list", json!({})).await.expect("crews");
    crews
        .as_array()
        .expect("list")
        .iter()
        .find(|crew| crew["name"] == name)
        .expect("crew")
        .clone()
}

#[tokio::test]
async fn the_owner_builds_the_plan_and_can_undo_it() {
    let mut m = maker().await;
    let call = build(&m.mcp, blog_plan());
    let asked = pending_build(&mut m.app).await;
    assert_eq!(asked["summary"], "A crew for the blog");
    m.t.until_state(&m.bot, BotState::NeedsApproval).await;
    let input: Value = serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");
    // The catalog's role and the chief's words are filled in for the card.
    assert!(
        !input["steps"][1]["instructions"]
            .as_str()
            .expect("text")
            .is_empty()
    );
    assert_eq!(input["steps"][0]["lead_name"], "Chief");

    m.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    let built = call.await.expect("task").expect("built");
    assert_eq!(built["built"], true);
    assert_eq!(built["crews"][0]["agents"][0]["handle"], "writer");
    assert_eq!(built["crews"][0]["routines"][0], "Weekly post");

    let crew = crew_named(&mut m.app, "Blog").await;
    let bots = m
        .app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    // Both are made in the same millisecond, so the list's order between
    // them is the ids' and changes from run to run.
    let mut names: Vec<&str> = bots
        .as_array()
        .expect("bots")
        .iter()
        .map(|bot| bot["name"].as_str().expect("name"))
        .collect();
    names.sort_unstable();
    assert_eq!(names, ["Chief", "Writer"]);
    let named = |name: &str| {
        bots.as_array()
            .expect("bots")
            .iter()
            .find(|bot| bot["name"] == name)
            .expect("bot")
            .clone()
    };
    let (chief, writer) = (named("Chief"), named("Writer"));
    assert_eq!(crew["leadBotId"], chief["id"]);
    let routines = m
        .app
        .call("routines.list", json!({ "botId": writer["id"] }))
        .await
        .expect("routines");
    assert_eq!(routines[0]["name"], "Weekly post");
    let workspace = chief["workspace"].as_str().expect("workspace");
    let memory = Path::new(workspace)
        .parent()
        .expect("crew")
        .join("CLAUDE.md");
    assert!(
        std::fs::read_to_string(memory)
            .expect("memory")
            .contains("Warm tone")
    );

    let builds = m.app.call("builds.list", json!({})).await.expect("builds");
    assert_eq!(builds[0]["approvalId"], asked["approvalId"]);
    assert_eq!(builds[0]["crewIds"][0], crew["id"]);
    let undone = m
        .app
        .call("builds.undo", json!({ "buildId": builds[0]["id"] }))
        .await
        .expect("undo");
    assert!(undone["undoneAt"].is_number());
    assert!(
        crew_names(&mut m.app).await.is_empty(),
        "the crew is archived"
    );
    let again = m
        .app
        .call("builds.undo", json!({ "buildId": builds[0]["id"] }))
        .await;
    assert!(again.is_err(), "undone once");
}

#[tokio::test]
async fn the_owner_leaves_out_part_of_the_plan() {
    let mut m = maker().await;
    let call = build(&m.mcp, blog_plan());
    let asked = pending_build(&mut m.app).await;
    let mut changed: Value =
        serde_json::from_str(asked["input"].as_str().expect("input")).expect("json");

    // A change that names a bot the plan no longer has keeps the card open.
    let steps = changed["steps"].as_array_mut().expect("steps");
    steps.remove(1);
    let broken = m
        .app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true, "input": changed.to_string() }),
        )
        .await;
    assert!(broken.is_err(), "the routine's bot is gone");

    changed["steps"].as_array_mut().expect("steps").truncate(1);
    changed["steps"][0]["name"] = json!("Posts");
    m.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true, "input": changed.to_string() }),
        )
        .await
        .expect("answer");
    let built = call.await.expect("task").expect("built");
    assert!(
        built["note"]
            .as_str()
            .expect("note")
            .contains("changed the plan")
    );
    assert_eq!(crew_names(&mut m.app).await, ["Posts"]);
}

#[tokio::test]
async fn a_declined_plan_builds_nothing() {
    let mut m = maker().await;
    let call = build(&m.mcp, blog_plan());
    let asked = pending_build(&mut m.app).await;
    m.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": false, "note": "Not now" }),
        )
        .await
        .expect("answer");
    let answer = call.await.expect("task").expect("answer");
    assert_eq!(answer["built"], false);
    assert!(answer["note"].as_str().expect("note").contains("Not now"));
    assert!(crew_names(&mut m.app).await.is_empty());
}

#[tokio::test]
async fn the_owner_can_let_the_maker_build_without_asking() {
    let mut m = maker().await;
    m.app
        .call(
            "settings.update",
            json!({ "makerBuildsWithoutAsking": true }),
        )
        .await
        .expect("settings");
    let built = m.mcp.tool("build", blog_plan()).await.expect("built");
    assert_eq!(built["built"], true);
    assert!(built["note"].as_str().expect("note").contains("undo"));
    assert_eq!(crew_names(&mut m.app).await, ["Blog"]);
    let builds = m.app.call("builds.list", json!({})).await.expect("builds");
    assert!(builds[0]["approvalId"].is_null());
}
