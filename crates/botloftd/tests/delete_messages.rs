//! What the other bots hear when a bot is deleted (spec 7.6): notices
//! about the tasks it leaves, and its messages that still waited.

mod common;

use common::Client;
use common::bots::{text_of, two_bots};
use serde_json::{Value, json};

async fn bot_named(app: &mut Client, name: &str) -> Value {
    let bots = app.call("bots.list", json!({})).await.expect("bots");
    bots.as_array()
        .expect("list")
        .iter()
        .find(|bot| bot["name"] == name)
        .cloned()
        .unwrap_or_else(|| panic!("no bot named {name}"))
}

#[tokio::test]
async fn the_other_bots_hear_about_the_tasks_a_deleted_bot_leaves() {
    let mut c = two_bots().await;
    let ask = |to: &str| json!({ "to": to, "body": "Please help.", "kind": "task" });
    let owed = c
        .writer
        .tool("send_message", ask("lead"))
        .await
        .expect("task for the lead");
    let asked = c
        .lead
        .tool("send_message", ask("writer"))
        .await
        .expect("task for the writer");
    let (owed, asked) = (
        owed["task_id"].as_str().expect("id").to_owned(),
        asked["task_id"].as_str().expect("id").to_owned(),
    );
    c.lead
        .tool(
            "send_message",
            json!({ "to": "writer", "body": "The draft is in shared/." }),
        )
        .await
        .expect("note");
    c.writer_process.wait_lines(2).await;

    let lead = bot_named(&mut c.app, "Lead").await;
    c.app
        .call("bots.delete", json!({ "botId": lead["id"] }))
        .await
        .expect("delete");

    let lines = c.writer_process.wait_lines(4).await;
    assert_eq!(
        text_of(&lines[2]),
        format!(
            "[botloft] from Botloft · crew Ops\n\nTask {owed} for @lead will not be done: the \
             owner deleted @lead. Get it done another way, or tell the owner what is missing."
        )
    );
    assert_eq!(
        text_of(&lines[3]),
        format!(
            "[botloft] from Botloft · crew Ops\n\nTask {asked} from @lead no longer needs a \
             result: the owner deleted @lead. Stop working on it."
        )
    );
    assert_eq!(
        c.app.call("tasks.list", json!({})).await.expect("tasks"),
        json!([])
    );
    let gone = c
        .writer
        .tool(
            "complete_task",
            json!({ "task_id": asked, "result": "Done anyway." }),
        )
        .await
        .expect_err("the task went with the bot");
    assert!(gone.contains("does not exist"), "{gone}");

    // What the lead wrote stays in the writer's chat, without a sender.
    let writer = bot_named(&mut c.app, "Writer").await;
    let kept = c
        .app
        .call("messages.list", json!({ "botId": writer["id"] }))
        .await
        .expect("messages");
    let kept: Vec<_> = kept.as_array().expect("list").iter().rev().collect();
    assert_eq!(kept.len(), 4);
    for message in &kept[..2] {
        assert_eq!(message["fromKind"], "bot");
        assert_eq!(message["fromBotId"], Value::Null);
        assert_eq!(message["taskId"], Value::Null);
    }
    assert_eq!(kept[1]["body"], "The draft is in shared/.");
    assert_eq!(kept[2]["fromKind"], "system");
}

#[tokio::test]
async fn a_message_still_waiting_arrives_from_a_deleted_bot() {
    let mut c = two_bots().await;
    let writer = bot_named(&mut c.app, "Writer").await;
    c.app
        .call(
            "bots.setPaused",
            json!({ "botId": writer["id"], "paused": true }),
        )
        .await
        .expect("pause");
    c.lead
        .tool(
            "send_message",
            json!({ "to": "writer", "body": "The draft is in shared/." }),
        )
        .await
        .expect("note");
    let lead = bot_named(&mut c.app, "Lead").await;
    c.app
        .call("bots.delete", json!({ "botId": lead["id"] }))
        .await
        .expect("delete");

    c.app
        .call(
            "bots.setPaused",
            json!({ "botId": writer["id"], "paused": false }),
        )
        .await
        .expect("resume");
    // The lead's, the writer's first and the one the writer resumes in.
    let process = c.t.runtime.process(3).await;
    let lines = process.wait_lines(1).await;
    assert_eq!(
        text_of(&lines[0]),
        "[botloft] from a deleted bot · crew Ops\n\nThe draft is in shared/."
    );
}
