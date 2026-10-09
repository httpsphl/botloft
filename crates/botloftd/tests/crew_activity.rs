//! `crew_activity` (spec 29.2): the chief reads what the crew's bots did,
//! for the owner's daily summary; nobody else may.

mod common;

use botloft_core::protocol::BotState;
use common::bots::ready_bot;
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

struct Site {
    t: TestDaemon,
    app: Client,
    chief: Mcp,
    writer: Mcp,
}

/// A crew "Site" with its chief (Chefe) and a bot Writer, both ready.
async fn site() -> Site {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call(
            "crews.create",
            json!({
                "name": "Site",
                "lead": { "name": "Chefe", "role": "Leads the crew", "instructions": "Build it" },
            }),
        )
        .await
        .expect("crew");
    let bots = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    let chief_bot = bots[0].clone();
    let process = t.process_of(&chief_bot).await;
    t.until_state(&chief_bot, BotState::Idle).await;
    let chief = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
    let (_, _writer_process, writer) = ready_bot(&t, &mut app, &crew, "Writer").await;
    Site {
        t,
        app,
        chief,
        writer,
    }
}

fn writer_of(report: &Value) -> &Value {
    report["bots"]
        .as_array()
        .expect("bots")
        .iter()
        .find(|bot| bot["handle"] == "writer")
        .expect("the writer is in the report")
}

#[tokio::test]
async fn the_chief_reads_what_each_bot_did() {
    let mut s = site().await;
    let done = s
        .chief
        .tool(
            "send_message",
            json!({ "to": "@writer", "body": "Write the intro.", "kind": "task", "deadline_minutes": 30 }),
        )
        .await
        .expect("task");
    let still_open = s
        .chief
        .tool(
            "send_message",
            json!({ "to": "@writer", "body": "Write the outro.", "kind": "task", "deadline_minutes": 30 }),
        )
        .await
        .expect("second task");
    assert!(still_open["task_id"].is_string());
    s.writer
        .tool(
            "complete_task",
            json!({ "task_id": done["task_id"], "result": "It is in shared/intro.md." }),
        )
        .await
        .expect("complete");

    let report = s
        .chief
        .tool("crew_activity", json!({}))
        .await
        .expect("activity");
    assert_eq!(report["crew"], "Site");
    assert_eq!(report["window_hours"], 24);
    assert_eq!(report["totals"]["done"], 1);
    assert_eq!(report["totals"]["open"], 1);
    assert_eq!(report["totals"]["failed"], 0);
    assert_eq!(report["totals"]["overdue"], 0);

    let writer = writer_of(&report);
    assert_eq!(writer["tasks"]["done"], 1);
    assert_eq!(writer["tasks"]["open"], 1);
    assert_eq!(writer["recent"][0]["status"], "done");
    assert_eq!(writer["recent"][0]["from"], "chefe");
    assert_eq!(writer["recent"][0]["request"], "Write the intro.");
    assert_eq!(writer["recent"][0]["result"], "It is in shared/intro.md.");
    assert_eq!(writer["waiting_for_owner"]["approvals"], 0);
    assert_eq!(writer["waiting_for_owner"]["questions"], 0);
    assert_eq!(writer["routines"], json!([]));
    drop(s.app);
    drop(s.t);
}

#[tokio::test]
async fn what_waits_for_the_owner_is_counted() {
    let mut s = site().await;
    s.writer
        .tool(
            "ask_owner",
            json!({ "question": "Which tone do you want?", "options": ["Warm", "Formal"] }),
        )
        .await
        .expect("question");
    let report = s
        .chief
        .tool("crew_activity", json!({ "hours": 1 }))
        .await
        .expect("activity");
    assert_eq!(report["window_hours"], 1);
    assert_eq!(writer_of(&report)["waiting_for_owner"]["questions"], 1);
    assert_eq!(report["totals"]["waiting_for_owner"], 1);
    drop(s.app);
}

#[tokio::test]
async fn only_the_chief_reads_it_and_the_window_has_limits() {
    let mut s = site().await;
    let refused = s
        .writer
        .tool("crew_activity", json!({}))
        .await
        .expect_err("a bot that is not the chief");
    assert!(refused.contains("only the crew's chief"), "{refused}");

    for hours in [0, 169] {
        let refused = s
            .chief
            .tool("crew_activity", json!({ "hours": hours }))
            .await
            .expect_err("outside the window");
        assert!(refused.contains("between 1 and 168"), "{refused}");
    }
    drop(s.app);
}
