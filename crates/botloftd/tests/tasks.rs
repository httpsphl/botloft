//! Bots working together through the MCP tools (spec 9.4 and 10): tasks
//! and results between two bots, chains that stop at the hop limit,
//! deadlines, mistakes the model can correct and crew boundaries. The test
//! daemon allows 3 hops.

mod common;

use std::time::Duration;

use common::Client;
use common::bots::{ready_bot, text_of, two_bots};
use serde_json::{Value, json};

fn task_id(out: &Value) -> String {
    out["task_id"].as_str().expect("task id").to_owned()
}

async fn task(app: &mut Client, id: &str) -> Value {
    let tasks = app.call("tasks.list", json!({})).await.expect("tasks");
    tasks
        .as_array()
        .expect("list")
        .iter()
        .find(|task| task["id"] == id)
        .cloned()
        .expect("task listed")
}

#[tokio::test]
async fn a_task_reaches_the_assignee_and_its_result_comes_back() {
    let mut c = two_bots().await;
    let sent = c
        .lead
        .tool(
            "send_message",
            json!({ "to": "@writer", "body": "Write the intro.", "kind": "task", "deadline_minutes": 30 }),
        )
        .await
        .expect("send");
    let id = task_id(&sent);
    assert_eq!(sent["due"], "due in 30 min");

    let asked = text_of(&c.writer_process.wait_lines(1).await[0]);
    assert_eq!(
        asked,
        format!(
            "[botloft] from @lead · crew Ops · task {id} · due in 30 min\n\
             Reply with send_message(to: \"lead\"). When the task is done, call \
             complete_task(task_id: \"{id}\").\n\nWrite the intro."
        )
    );

    let mine = c
        .writer
        .tool("my_tasks", json!({}))
        .await
        .expect("my_tasks");
    assert_eq!(mine["assigned"][0]["task_id"], id.as_str());
    assert_eq!(mine["assigned"][0]["from"], "lead");
    assert_eq!(mine["assigned"][0]["request"], "Write the intro.");
    assert_eq!(mine["requested"], json!([]));

    let done = c
        .writer
        .tool(
            "complete_task",
            json!({ "task_id": id, "result": "It is in shared/intro.md." }),
        )
        .await
        .expect("complete");
    assert_eq!(done["status"], "done");
    let result = text_of(&c.lead_process.wait_lines(1).await[0]);
    assert_eq!(
        result,
        format!(
            "[botloft] from @writer · crew Ops · result of task {id} · done\n\
             Reply with send_message(to: \"writer\").\n\nIt is in shared/intro.md."
        )
    );
    let stored = task(&mut c.app, &id).await;
    assert_eq!(
        (&stored["status"], &stored["result"]),
        (&json!("done"), &json!("It is in shared/intro.md."))
    );
    let again = c
        .writer
        .tool("complete_task", json!({ "task_id": id, "result": "again" }))
        .await
        .expect_err("already done");
    assert!(again.contains("already done"), "{again}");
    let requested = c
        .lead
        .tool("my_tasks", json!({ "role": "requested" }))
        .await
        .expect("mine");
    assert_eq!(requested, json!({ "requested": [] }));
}

#[tokio::test]
async fn chains_of_tasks_stop_at_the_hop_limit() {
    let mut c = two_bots().await;
    let ask = |to: &str| json!({ "to": to, "body": "Please help.", "kind": "task" });
    let first = task_id(
        &c.lead
            .tool("send_message", ask("writer"))
            .await
            .expect("hop 1"),
    );
    let second = task_id(
        &c.writer
            .tool("send_message", ask("lead"))
            .await
            .expect("hop 2"),
    );
    let third = task_id(
        &c.lead
            .tool("send_message", ask("writer"))
            .await
            .expect("hop 3"),
    );
    for (id, hops) in [(&first, 1), (&second, 2), (&third, 3)] {
        let stored = task(&mut c.app, id).await;
        assert_eq!(stored["hops"], hops);
        let origin = if hops == 1 { json!(null) } else { json!(first) };
        assert_eq!(stored["originTaskId"], origin);
    }
    let refused = c
        .writer
        .tool("send_message", ask("lead"))
        .await
        .expect_err("hop 4");
    assert!(
        refused.contains("step 4") && refused.contains("limit is 3"),
        "{refused}"
    );
    assert!(
        refused.contains(&first),
        "names where the chain started: {refused}"
    );
    c.writer
        .tool(
            "send_message",
            json!({ "to": "lead", "body": "FYI: stuck." }),
        )
        .await
        .expect("notes have no hop limit");
}

#[tokio::test]
async fn an_overdue_task_expires_and_can_still_be_completed() {
    let mut c = two_bots().await;
    let sent = c
        .lead
        .tool("send_message", json!({ "to": "writer", "body": "Quick check.", "kind": "task", "deadline_minutes": 1 }))
        .await
        .expect("send");
    let id = task_id(&sent);
    c.writer_process.wait_lines(1).await;
    c.t.clock.advance(Duration::from_secs(61));
    let notice = text_of(&c.lead_process.wait_lines(1).await[0]);
    assert!(
        notice.starts_with(&format!(
            "[botloft] from Botloft · crew Ops · task {id} · expired\n\nTask {id} for @writer passed its deadline"
        )),
        "{notice}"
    );
    assert_eq!(task(&mut c.app, &id).await["status"], "expired");

    c.writer
        .tool(
            "complete_task",
            json!({ "task_id": id, "result": "Late, but done." }),
        )
        .await
        .expect("late result");
    let result = text_of(&c.lead_process.wait_lines(2).await[1]);
    assert!(result.contains("result of task") && result.ends_with("Late, but done."));
}

#[tokio::test]
async fn mistakes_come_back_as_tool_errors() {
    let mut c = two_bots().await;
    let roster = c.lead.tool("crew_roster", json!({})).await.expect("roster");
    assert_eq!(roster["crew"], "Ops");
    assert_eq!(roster["you"], "lead");
    assert_eq!(
        roster["bots"],
        json!([{ "handle": "writer", "name": "Writer", "role": "Writer role", "state": "idle",
            "model": "default", "effort": "default", "chief": false }])
    );

    let err = |out: Result<Value, String>| out.expect_err("refused");

    let cases = [
        (
            json!({ "to": "ghost", "body": "hi" }),
            "no bot in your crew answers to @ghost",
        ),
        (
            json!({ "to": "Lead", "body": "hi" }),
            "cannot send a message to yourself",
        ),
        (
            json!({ "to": "writer", "body": " " }),
            "body must not be empty",
        ),
        (
            json!({ "to": "writer", "body": "hi", "kind": "urgent" }),
            "kind must be",
        ),
        (
            json!({ "to": "writer", "body": "hi", "deadline_minutes": 5 }),
            "only applies to kind",
        ),
        (
            json!({ "to": "writer", "body": "hi", "kind": "task", "deadline_minutes": 0 }),
            "between 1 and",
        ),
        (
            json!({ "to": "writer", "body": "hi", "cc": "x" }),
            "unknown field",
        ),
    ];
    for (args, expected) in cases {
        let message = err(c.lead.tool("send_message", args).await);
        assert!(message.contains(expected), "{message}");
    }

    let not_an_id = err(c
        .writer
        .tool("complete_task", json!({ "task_id": "nope", "result": "x" }))
        .await);
    assert!(not_an_id.contains("not a task id"), "{not_an_id}");
    let sent = c
        .lead
        .tool(
            "send_message",
            json!({ "to": "writer", "body": "Do it.", "kind": "task" }),
        )
        .await
        .expect("send");
    let not_mine = err(c
        .lead
        .tool(
            "complete_task",
            json!({ "task_id": task_id(&sent), "result": "x" }),
        )
        .await);
    assert!(not_mine.contains("not assigned to you"), "{not_mine}");
}

#[tokio::test]
async fn bots_only_reach_their_own_crew() {
    let mut c = two_bots().await;
    let other = c
        .app
        .call("crews.create", json!({ "name": "Other" }))
        .await
        .expect("crew");
    let (_, stranger_process, mut stranger) = ready_bot(&c.t, &mut c.app, &other, "Writer").await;
    assert_ne!(other["id"], c.crew["id"]);

    let sent = c
        .lead
        .tool(
            "send_message",
            json!({ "to": "writer", "body": "Ours.", "kind": "task" }),
        )
        .await
        .expect("send");
    c.writer_process.wait_lines(1).await;
    assert!(
        stranger_process.input_lines().is_empty(),
        "only the writer of Ops"
    );
    let roster = c.lead.tool("crew_roster", json!({})).await.expect("roster");
    assert_eq!(roster["bots"].as_array().map(Vec::len), Some(1));

    let foreign = stranger
        .tool(
            "complete_task",
            json!({ "task_id": task_id(&sent), "result": "mine now" }),
        )
        .await
        .expect_err("another crew's task");
    assert!(foreign.contains("does not exist"), "{foreign}");
    let unknown = stranger
        .tool("send_message", json!({ "to": "lead", "body": "hi" }))
        .await
        .expect_err("another crew's bot");
    assert!(
        unknown.contains("no bot in your crew answers to @lead"),
        "{unknown}"
    );
}
