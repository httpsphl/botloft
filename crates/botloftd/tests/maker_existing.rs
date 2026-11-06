//! `build` on crews that already exist and `crews_overview` (spec 36.8):
//! the Maker changes a bot's instructions, adds and archives bots, changes
//! and deletes routines, and the owner can undo all of it.

mod common;

use common::maker::{Maker, blog_plan, build, maker, pending_build};
use serde_json::{Value, json};

/// The Maker with the "Blog" crew already built, building without asking.
async fn with_blog() -> Maker {
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
    m
}

async fn bot_named(m: &mut Maker, name: &str) -> Option<Value> {
    let crews = m.app.call("crews.list", json!({})).await.expect("crews");
    let crew = crews
        .as_array()
        .expect("list")
        .iter()
        .find(|crew| crew["name"] == "Blog")
        .expect("crew")
        .clone();
    let bots = m
        .app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    bots.as_array()
        .expect("bots")
        .iter()
        .find(|bot| bot["name"] == name)
        .cloned()
}

async fn routines_of(m: &mut Maker, bot: &Value) -> Value {
    m.app
        .call("routines.list", json!({ "botId": bot["id"] }))
        .await
        .expect("routines")
}

#[tokio::test]
async fn the_maker_sees_the_crews_in_full() {
    let mut m = with_blog().await;
    let list = m.mcp.tool("crews_overview", json!({})).await.expect("list");
    assert_eq!(list["crews"][0]["crew"], "Blog");
    assert_eq!(list["crews"][0]["routines"][0]["name"], "Weekly post");
    assert!(list["crews"][0]["agents"][0].get("instructions").is_none());

    let full = m
        .mcp
        .tool("crews_overview", json!({ "crew": "blog" }))
        .await
        .expect("crew");
    let writer = full["agents"]
        .as_array()
        .expect("agents")
        .iter()
        .find(|agent| agent["handle"] == "writer")
        .expect("writer");
    assert!(!writer["instructions"].as_str().expect("text").is_empty());
    assert_eq!(full["routines"][0]["prompt"], "Write this week's post.");
    let _ = bot_named(&mut m, "Writer").await;
}

#[tokio::test]
async fn the_maker_changes_a_crew_and_the_owner_undoes_it() {
    let mut m = with_blog().await;
    let plan = json!({
        "title": "A better blog",
        "why": "You said the posts are too long.",
        "steps": [
            {
                "op": "change_agent", "crew": "Blog", "agent": "writer",
                "instructions": "Write short posts, at most 300 words.", "model": "haiku",
            },
            {
                "op": "add_agent", "crew": "Blog", "name": "Editor",
                "role": "Edits the posts", "instructions": "Cut every post in half.",
            },
            {
                "op": "add_routine", "crew": "Blog", "agent": "Editor", "name": "Edit",
                "prompt": "Edit the week's post.", "schedule": { "kind": "manual" },
            },
            { "op": "change_routine", "crew": "Blog", "routine": "weekly post", "enabled": false },
        ],
    });
    let built = m.mcp.tool("build", plan).await.expect("built");
    assert_eq!(built["built"], true, "{built}");
    assert_eq!(built["changes"].as_array().expect("changes").len(), 4);

    let writer = bot_named(&mut m, "Writer").await.expect("writer");
    assert_eq!(writer["model"], "haiku");
    assert_eq!(
        writer["instructions"],
        "Write short posts, at most 300 words."
    );
    let editor = bot_named(&mut m, "Editor").await.expect("editor");
    assert_eq!(routines_of(&mut m, &editor).await[0]["name"], "Edit");
    assert_eq!(routines_of(&mut m, &writer).await[0]["enabled"], false);

    let builds = m.app.call("builds.list", json!({})).await.expect("builds");
    m.app
        .call("builds.undo", json!({ "buildId": builds[0]["id"] }))
        .await
        .expect("undo");
    let writer = bot_named(&mut m, "Writer").await.expect("writer");
    assert_ne!(writer["model"], "haiku");
    assert_ne!(
        writer["instructions"],
        "Write short posts, at most 300 words."
    );
    assert_eq!(routines_of(&mut m, &writer).await[0]["enabled"], true);
    assert!(
        bot_named(&mut m, "Editor").await.is_none(),
        "the editor went to the archive"
    );
}

#[tokio::test]
async fn archiving_always_asks_and_can_be_undone() {
    let mut m = with_blog().await;
    let plan = json!({
        "title": "No writer",
        "why": "You write the posts yourself now.",
        "steps": [
            { "op": "archive_agent", "crew": "Blog", "agent": "writer" },
            { "op": "delete_routine", "crew": "Blog", "routine": "Weekly post" },
        ],
    });
    // Even building without asking, archiving waits for the owner.
    let call = build(&m.mcp, plan);
    let asked = pending_build(&mut m.app).await;
    m.app
        .call(
            "approvals.answer",
            json!({ "approvalId": asked["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    let built = call.await.expect("task").expect("built");
    assert_eq!(built["built"], true, "{built}");
    assert!(bot_named(&mut m, "Writer").await.is_none(), "archived");

    let builds = m.app.call("builds.list", json!({})).await.expect("builds");
    m.app
        .call("builds.undo", json!({ "buildId": builds[0]["id"] }))
        .await
        .expect("undo");
    let writer = bot_named(&mut m, "Writer").await.expect("back");
    assert_eq!(routines_of(&mut m, &writer).await[0]["name"], "Weekly post");
}

#[tokio::test]
async fn impossible_changes_never_reach_the_owner() {
    let mut m = with_blog().await;
    for (step, says) in [
        (
            json!({ "op": "change_agent", "crew": "Nope", "agent": "writer", "role": "x" }),
            "no crew is called",
        ),
        (
            json!({ "op": "archive_agent", "crew": "Blog", "agent": "chief" }),
            "chief",
        ),
        (
            json!({ "op": "change_routine", "crew": "Blog", "routine": "Nope", "enabled": false }),
            "no routine",
        ),
        (
            json!({ "op": "change_agent", "crew": "Blog", "agent": "writer", "instructions": "a", "add_instructions": "b" }),
            "not both",
        ),
    ] {
        let plan = json!({ "title": "Bad", "why": "A test.", "steps": [step] });
        let err = m.mcp.tool("build", plan).await.expect_err("refused");
        assert!(err.contains(says), "{err}");
    }
}
