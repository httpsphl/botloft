//! Deleting bots and crews for good (spec 7.6): what stops, what leaves
//! the database and what stays on disk. What the other bots hear is in
//! `delete_messages.rs`.

mod common;

use std::time::Duration;

use botloft_core::ids::BotId;
use common::bots::{ready_bot, two_bots};
use common::mcp::Mcp;
use common::stream;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

const NOT_FOUND: i64 = -32002;

async fn bot_named(app: &mut Client, name: &str) -> Value {
    let bots = app.call("bots.list", json!({})).await.expect("bots");
    bots.as_array()
        .expect("list")
        .iter()
        .find(|bot| bot["name"] == name)
        .cloned()
        .unwrap_or_else(|| panic!("no bot named {name}"))
}

fn id_of(bot: &Value) -> BotId {
    bot["id"].as_str().expect("id").parse().expect("bot id")
}

#[tokio::test]
async fn a_deleted_bot_stops_and_leaves_only_its_folder() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let mut watcher = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (scout, process, _) = ready_bot(&t, &mut app, &crew, "Scout").await;
    app.call(
        "messages.send",
        json!({ "botId": scout["id"], "body": "Hello" }),
    )
    .await
    .expect("send");
    process.wait_lines(1).await;
    let workspace = t.paths.bot_workspace("ops", "scout");
    std::fs::write(workspace.join("CLAUDE.md"), "what the bot learned").expect("memory");

    let deleted = app
        .call("bots.delete", json!({ "botId": scout["id"] }))
        .await
        .expect("delete");
    assert_eq!(
        deleted,
        json!({ "botId": scout["id"], "crewId": crew["id"] })
    );
    assert_eq!(watcher.notification("bot.deleted").await, deleted);
    assert!(process.killed(), "the process does not outlive the bot");
    assert_eq!(t.daemon.supervisor.status(&id_of(&scout)), None);

    assert_eq!(
        app.call("bots.list", json!({})).await.expect("bots"),
        json!([])
    );
    for (method, params) in [
        ("chat.history", json!({ "botId": scout["id"] })),
        ("messages.list", json!({ "botId": scout["id"] })),
        ("bots.delete", json!({ "botId": scout["id"] })),
        (
            "messages.send",
            json!({ "botId": scout["id"], "body": "Still there?" }),
        ),
    ] {
        let gone = app.call(method, params).await.expect_err(method);
        assert_eq!(gone.code, NOT_FOUND, "{method}");
    }
    let timeline = app
        .call("messages.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("timeline");
    assert_eq!(timeline, json!([]));

    // The folder stays as it was, and nothing starts the bot again.
    assert_eq!(
        std::fs::read_to_string(workspace.join("CLAUDE.md")).expect("memory"),
        "what the bot learned"
    );
    t.daemon.supervisor.reconcile().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(t.runtime.processes().len(), 1);
}

#[tokio::test]
async fn deleting_the_chief_leaves_the_crew_without_one() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let lead = json!({ "name": "Chief", "role": "Leads", "instructions": "Ship the site." });
    let crew = app
        .call("crews.create", json!({ "name": "Ops", "lead": lead }))
        .await
        .expect("crew");
    let mut watcher = t.session().await;
    app.call("bots.delete", json!({ "botId": crew["leadBotId"] }))
        .await
        .expect("delete");
    let changed = watcher.notification("crew.changed").await;
    assert_eq!(changed["leadBotId"], Value::Null);
    let crews = app.call("crews.list", json!(null)).await.expect("crews");
    assert_eq!(crews[0]["leadBotId"], Value::Null);
}

#[tokio::test]
async fn archived_bots_and_crews_can_be_deleted_too() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let scout = app.call("bots.create", bot.clone()).await.expect("bot");
    app.call("bots.archive", json!({ "botId": scout["id"] }))
        .await
        .expect("archive");
    app.call("bots.delete", json!({ "botId": scout["id"] }))
        .await
        .expect("delete an archived bot");

    // The folder is still there, so a new bot of the same name gets another.
    let again = app.call("bots.create", bot).await.expect("bot");
    assert_eq!(again["handle"], "scout");
    assert_eq!(again["slug"], "scout-2");

    app.call("crews.archive", json!({ "crewId": crew["id"] }))
        .await
        .expect("archive");
    app.call("crews.delete", json!({ "crewId": crew["id"] }))
        .await
        .expect("delete an archived crew");
    let gone = app
        .call("crews.delete", json!({ "crewId": crew["id"] }))
        .await
        .expect_err("already gone");
    assert_eq!(gone.code, NOT_FOUND);
}

#[tokio::test]
async fn a_deleted_crew_takes_its_bots_and_keeps_its_folders() {
    let mut c = two_bots().await;
    let mut watcher = c.t.session().await;
    let other = c
        .app
        .call("crews.create", json!({ "name": "Docs" }))
        .await
        .expect("other crew");
    let (editor, editor_process, _) = ready_bot(&c.t, &mut c.app, &other, "Editor").await;
    c.lead
        .tool(
            "send_message",
            json!({ "to": "writer", "body": "Please help.", "kind": "task" }),
        )
        .await
        .expect("task");
    c.writer_process.wait_lines(1).await;
    let lead = bot_named(&mut c.app, "Lead").await;
    let shared = c.t.paths.shared_dir("ops");
    std::fs::write(shared.join("report.md"), "the work").expect("work file");

    let deleted = c
        .app
        .call("crews.delete", json!({ "crewId": c.crew["id"] }))
        .await
        .expect("delete");
    assert_eq!(deleted, json!({ "crewId": c.crew["id"] }));
    assert_eq!(watcher.notification("crew.deleted").await, deleted);
    assert!(c.lead_process.killed() && c.writer_process.killed());
    assert!(!editor_process.killed(), "other crews keep running");
    assert_eq!(c.t.daemon.supervisor.status(&id_of(&lead)), None);

    let crews = c.app.call("crews.list", json!(null)).await.expect("crews");
    assert_eq!(crews, json!([other]));
    let bots = c.app.call("bots.list", json!({})).await.expect("bots");
    assert_eq!(bots.as_array().expect("list").len(), 1);
    assert_eq!(bots[0]["id"], editor["id"]);
    assert_eq!(
        c.app.call("tasks.list", json!({})).await.expect("tasks"),
        json!([])
    );
    // A deleted bot's token opens nothing.
    let refused = c.writer.request("ping", json!({})).await;
    assert_eq!(refused.status, 401);

    assert_eq!(
        std::fs::read_to_string(shared.join("report.md")).expect("work file"),
        "the work"
    );
    assert!(c.t.paths.bot_workspace("ops", "lead").is_dir());
}

#[tokio::test]
async fn a_request_waiting_for_the_owner_goes_with_the_bot() {
    let mut c = two_bots().await;
    let lead = bot_named(&mut c.app, "Lead").await;
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": "Build it" }),
        )
        .await
        .expect("send");
    let line = c.lead_process.wait_lines(1).await[0].clone();
    c.lead_process.emit(stream::replay(&line)).await;
    c.lead_process
        .emit(stream::tool_use(
            "toolu_1",
            "Bash",
            json!({ "command": "make" }),
        ))
        .await;
    let mut asking = Mcp::new(c.lead.addr, c.lead.token.clone());
    let args =
        json!({ "tool_name": "Bash", "input": { "command": "make" }, "tool_use_id": "toolu_1" });
    let decision = tokio::spawn(async move {
        asking
            .tool("permission_prompt", args)
            .await
            .expect("answer")
    });
    let request = loop {
        let item = c.app.notification("chat.item").await;
        if item["item"]["body"]["kind"] == "approval" {
            break item["item"]["body"]["approvalId"].clone();
        }
    };

    let started = std::time::Instant::now();
    c.app
        .call("bots.delete", json!({ "botId": lead["id"] }))
        .await
        .expect("delete");
    let outcome = decision.await.expect("task");
    assert_eq!(outcome["behavior"], "deny");
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "the request did not wait for its 3 s timeout"
    );
    let gone = c
        .app
        .call(
            "approvals.answer",
            json!({ "approvalId": request, "allow": true }),
        )
        .await
        .expect_err("the request went with the bot");
    assert_eq!(gone.code, NOT_FOUND);
}
