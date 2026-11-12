//! Replies go back to the conversation that asked (spec 40.2): a question
//! to the owner, a task given to another agent and a reminder an agent set
//! for itself, each from a conversation that is not the main one.

mod common;

use std::time::Duration;

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::BotState;
use botloftd::runtime::fake::FakeProcess;
use common::conversations::{holding, open, send, turn};
use common::mcp::Mcp;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

/// A daemon whose bots sleep after ten idle minutes.
async fn start() -> TestDaemon {
    let t = TestDaemon::start_supervised().await;
    t.daemon
        .settings
        .set_sleep_after(Some(Duration::from_secs(600)));
    t
}

/// Sleeping agents named `names` in one new crew.
async fn agents(t: &TestDaemon, app: &mut Client, names: &[&str]) -> Vec<(Value, BotId)> {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let mut made = Vec::new();
    for name in names {
        let bot = json!({ "crewId": crew["id"], "name": name, "role": "", "instructions": "" });
        let bot = app.call("bots.create", bot).await.expect("bot");
        t.until_state(&bot, BotState::Sleeping).await;
        let id = bot["id"].as_str().expect("id").parse().expect("bot id");
        made.push((bot, id));
    }
    made
}

/// Opens another conversation of `bot`, wakes it with `body` and lets the
/// turn end: its process and MCP connection. `before` processes ran first.
async fn other_awake(
    t: &TestDaemon,
    app: &mut Client,
    bot: &(Value, BotId),
    body: &str,
    before: usize,
) -> (ConversationId, FakeProcess, Mcp) {
    let other = open(app, &bot.1).await;
    send(app, &bot.0, body, Some(&other)).await;
    let (process, line) = holding(t, before + 1, body).await;
    turn(&process, &line, "session-other", "On it").await;
    let token = process.env("BOTLOFT_BOT_TOKEN").expect("token");
    (other, process, Mcp::new(t.addr, token))
}

/// The kinds of the items in one conversation's chat.
async fn kinds(app: &mut Client, bot: &Value, conversation: Option<&ConversationId>) -> Vec<Value> {
    let mut params = json!({ "botId": bot["id"] });
    if let Some(conversation) = conversation {
        params["conversationId"] = json!(conversation.as_str());
    }
    let history = app.call("chat.history", params).await.expect("history");
    let items = history.as_array().expect("items");
    items
        .iter()
        .map(|item| item["body"]["kind"].clone())
        .collect()
}

/// The text of the `n`th line the process got, waiting for it.
async fn line_text(process: &FakeProcess, n: usize) -> String {
    process.wait_lines(n + 1).await[n].to_string()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_question_is_answered_in_the_conversation_that_asked() {
    let t = start().await;
    let mut app = t.session().await;
    let scout = agents(&t, &mut app, &["Scout"]).await.remove(0);
    let (other, process, mut mcp) = other_awake(&t, &mut app, &scout, "Plan the trip", 0).await;

    let asked = mcp
        .tool("ask_owner", json!({ "question": "Which city?" }))
        .await
        .expect("asked");
    let id = asked["question_id"].as_str().expect("id").to_owned();
    // Its card is in that conversation's chat, not the main one's.
    assert!(
        kinds(&mut app, &scout.0, Some(&other))
            .await
            .contains(&json!("question"))
    );
    assert!(
        !kinds(&mut app, &scout.0, None)
            .await
            .contains(&json!("question"))
    );

    let answer = json!({ "questionId": id, "answer": "Lisbon" });
    app.call("questions.answer", answer).await.expect("answer");
    assert!(line_text(&process, 1).await.contains("Lisbon"));
    // Its main conversation never woke for it.
    assert_eq!(t.runtime.processes().len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_task_result_comes_back_to_the_conversation_that_gave_it() {
    let t = start().await;
    let mut app = t.session().await;
    let mut both = agents(&t, &mut app, &["Lead", "Writer"]).await;
    let writer = both.remove(1);
    let lead = both.remove(0);
    let (_, lead_process, mut lead_mcp) = other_awake(&t, &mut app, &lead, "Delegate", 0).await;

    let task = json!({ "to": "writer", "body": "Write the intro", "kind": "task" });
    let sent = lead_mcp.tool("send_message", task).await.expect("sent");
    let task_id = sent["task_id"].as_str().expect("task").to_owned();
    // The writer gets it in its main conversation, and finishes it.
    let (writer_process, line) = holding(&t, 2, "Write the intro").await;
    turn(&writer_process, &line, "session-writer", "Writing").await;
    let token = writer_process.env("BOTLOFT_BOT_TOKEN").expect("token");
    let done = json!({ "task_id": task_id, "result": "The intro is done" });
    Mcp::new(t.addr, token)
        .tool("complete_task", done)
        .await
        .expect("done");

    assert!(
        line_text(&lead_process, 1)
            .await
            .contains("The intro is done")
    );
    assert_eq!(t.runtime.processes().len(), 2);
    assert_eq!(writer.0["handle"], "writer");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_reminder_fires_in_the_conversation_that_set_it() {
    let t = start().await;
    t.clock
        .set(botloftd::routines::schedule::at("UTC", 2026, 10, 1, 8, 0));
    let mut app = t.session().await;
    let scout = agents(&t, &mut app, &["Scout"]).await.remove(0);
    let (_, process, mut mcp) = other_awake(&t, &mut app, &scout, "Watch the deploy", 0).await;

    let reminder = json!({
        "name": "Look at the deploy again",
        "prompt": "Check whether the deploy finished.",
        "schedule": { "kind": "once", "at": "2026-10-01T10:00", "reminder": false },
        "timezone": "UTC",
    });
    let set = mcp.tool("schedule_routine", reminder).await.expect("set");
    assert_eq!(set["created"], true, "{set}");

    t.clock
        .set(botloftd::routines::schedule::at("UTC", 2026, 10, 1, 10, 1));
    botloftd::routines::tick(&t.daemon);
    assert!(
        line_text(&process, 1)
            .await
            .contains("Check whether the deploy finished.")
    );
    assert_eq!(t.runtime.processes().len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_request_waits_in_its_conversation_and_stop_refuses_only_it() {
    let t = start().await;
    let mut app = t.session().await;
    let scout = agents(&t, &mut app, &["Scout"]).await.remove(0);
    let (other, process, mcp) = other_awake(&t, &mut app, &scout, "Clean up", 0).await;
    send(&mut app, &scout.0, "Delete the build", Some(&other)).await;
    let line = process.wait_lines(2).await.remove(1);
    process.emit(common::stream::replay(&line)).await;
    let command = json!({ "command": "rm -rf build" });
    process
        .emit(common::stream::tool_use("t1", "Bash", command))
        .await;

    let mut asking = Mcp::new(mcp.addr, mcp.token.clone());
    let args =
        json!({ "tool_name": "Bash", "input": { "command": "rm -rf build" }, "tool_use_id": "t1" });
    let waiting = tokio::spawn(async move {
        asking
            .tool("permission_prompt", args)
            .await
            .expect("decision")
    });
    for _ in 0..200 {
        if kinds(&mut app, &scout.0, Some(&other))
            .await
            .contains(&json!("approval"))
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(
        kinds(&mut app, &scout.0, Some(&other))
            .await
            .contains(&json!("approval"))
    );
    assert!(
        !kinds(&mut app, &scout.0, None)
            .await
            .contains(&json!("approval"))
    );

    let stop = json!({ "botId": scout.1, "conversationId": other });
    app.call("bots.interrupt", stop).await.expect("stop");
    let decision = tokio::time::timeout(Duration::from_secs(5), waiting)
        .await
        .expect("answered")
        .expect("task");
    assert_eq!(decision["behavior"], "deny", "{decision}");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_answer_for_an_archived_conversation_goes_to_the_main_one() {
    let t = start().await;
    let mut app = t.session().await;
    let scout = agents(&t, &mut app, &["Scout"]).await.remove(0);
    let (other, _, mut mcp) = other_awake(&t, &mut app, &scout, "Plan the trip", 0).await;
    let asked = mcp
        .tool("ask_owner", json!({ "question": "Which city?" }))
        .await
        .expect("asked");
    let archive = json!({ "conversationId": other });
    app.call("conversations.archive", archive)
        .await
        .expect("archive");

    let answer = json!({ "questionId": asked["question_id"], "answer": "Porto" });
    app.call("questions.answer", answer).await.expect("answer");
    let (main, _) = holding(&t, 2, "Porto").await;
    // A new process: the main conversation's.
    assert_ne!(main.env("BOTLOFT_BOT_TOKEN"), Some(mcp.token.clone()));
}
