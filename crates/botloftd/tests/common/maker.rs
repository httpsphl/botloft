//! The Maker (spec 36), made as the daemon makes it at start, with its
//! process ready.

use botloft_core::protocol::BotState;
use serde_json::{Value, json};
use tokio::task::JoinHandle;

use super::mcp::Mcp;
use super::{Client, TestDaemon};

pub struct Maker {
    pub t: TestDaemon,
    pub app: Client,
    pub bot: Value,
    pub mcp: Mcp,
}

pub async fn maker() -> Maker {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let id = botloftd::service::maker::ensure(&t.daemon).expect("maker");
    t.daemon.supervisor.wake();
    let crews = app.call("crews.list", json!({})).await.expect("crews");
    let crew = crews
        .as_array()
        .expect("list")
        .iter()
        .find(|crew| crew["system"] == "maker")
        .expect("the Maker's crew")
        .clone();
    let bots = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    let bot = bots[0].clone();
    assert_eq!(bot["id"], id.as_str());
    let process = t.process_of(&bot).await;
    t.until_state(&bot, BotState::Idle).await;
    let mcp = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
    Maker { t, app, bot, mcp }
}

/// A plan for one crew "Blog" with its chief, a writer from the catalog and
/// a weekly routine for the writer.
pub fn blog_plan() -> Value {
    json!({
        "title": "A crew for the blog",
        "why": "You write a post every week by hand.",
        "steps": [
            {
                "op": "create_crew", "ref": "blog", "name": "Blog",
                "goal": "Publish one good post a week.",
                "memory": "The bakery's blog. Warm tone, no prices.",
            },
            { "op": "add_agent", "crew": "blog", "name": "Writer", "template": "writer" },
            {
                "op": "add_routine", "crew": "blog", "agent": "Writer", "name": "Weekly post",
                "prompt": "Write this week's post.",
                "schedule": { "kind": "weekly", "days": [1], "time": "08:00" },
            },
        ],
    })
}

pub fn build(mcp: &Mcp, plan: Value) -> JoinHandle<Result<Value, String>> {
    let mut mcp = Mcp::new(mcp.addr, mcp.token.clone());
    tokio::spawn(async move { mcp.tool("build", plan).await })
}

/// Waits for the plan to show up in the Maker's chat.
pub async fn pending_build(app: &mut Client) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        let body = &changed["item"]["body"];
        if body["kind"] == "approval" && body["status"] == "pending" {
            assert_eq!(body["toolName"], "mcp__botloft__build");
            return body.clone();
        }
    }
}

/// The owner's crews (the Maker's left out), by name.
pub async fn crew_names(app: &mut Client) -> Vec<String> {
    let crews = app.call("crews.list", json!({})).await.expect("crews");
    crews
        .as_array()
        .expect("list")
        .iter()
        .filter(|crew| crew["system"].is_null())
        .map(|crew| crew["name"].as_str().expect("name").to_owned())
        .collect()
}
