//! What a bot leaves running in the background (spec 33.4): the live list
//! Claude Code sends, and stopping one of its tasks.

mod common;

use botloft_core::protocol::{BackgroundKind, BotState, BotsStopTaskParams};
use botloftd::background;
use botloftd::service::ApiError;
use botloftd::state::Event;
use common::context::{bot, settle};
use common::supervised::setup;
use serde_json::{Value, json};

/// `system/background_tasks_changed` as Claude Code 2.1.309 sends it.
fn live(tasks: Value) -> Value {
    json!({ "type": "system", "subtype": "background_tasks_changed", "tasks": tasks })
}

fn command(id: &str) -> Value {
    json!({ "task_id": id, "run_id": format!("run-{id}"), "task_type": "local_bash",
            "description": "Start the dev server" })
}

#[tokio::test(start_paused = true)]
async fn the_live_list_replaces_the_one_before_without_claude_codes_own() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let mut events = s.daemon.subscribe();
    assert!(bot(&s).background.is_empty());

    let watcher = json!({ "task_id": "w1", "task_type": "local_bash",
                          "description": "Watch for updates", "ambient": true });
    process.emit(live(json!([command("b1"), watcher]))).await;
    settle().await;
    let tasks = bot(&s).background;
    assert_eq!(tasks.len(), 1, "Claude Code's own watcher stays out");
    assert_eq!(tasks[0].task_id, "b1");
    assert_eq!(tasks[0].kind, BackgroundKind::Command);
    assert_eq!(tasks[0].description, "Start the dev server");
    let first_seen = tasks[0].seen_at;

    // A subagent joins later; the command keeps the time it was first seen.
    tokio::time::advance(std::time::Duration::from_secs(30)).await;
    let agent = json!({ "task_id": "a1", "task_type": "local_agent",
                        "description": "Review the report" });
    process.emit(live(json!([command("b1"), agent]))).await;
    settle().await;
    let tasks = bot(&s).background;
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].seen_at, first_seen);
    assert_eq!(tasks[1].kind, BackgroundKind::Agent);

    // The same list again tells the app nothing new.
    process
        .emit(live(json!([command("b1"), {
        "task_id": "a1", "task_type": "local_agent", "description": "Review the report" }])))
        .await;
    settle().await;
    let mut told = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let Event::BotBackground(changed) = event {
            told.push(changed.tasks.len());
        }
    }
    assert_eq!(told, [1, 2]);
}

#[tokio::test(start_paused = true)]
async fn stopping_a_task_asks_claude_code_and_the_list_follows() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    process.emit(live(json!([command("b1")]))).await;
    settle().await;

    let stop = |task: &str| {
        background::stop(
            &s.daemon,
            BotsStopTaskParams {
                bot_id: s.bot.clone(),
                task_id: task.to_owned(),
                conversation_id: None,
            },
        )
    };
    assert!(matches!(stop("nope"), Err(ApiError::NotFound(_))));
    stop("b1").expect("stop");
    let asked: Vec<Value> = process
        .control_lines()
        .into_iter()
        .filter(|line| line["request"]["subtype"] == "stop_task")
        .collect();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0]["type"], "control_request");
    assert_eq!(asked[0]["request"]["task_id"], "b1");

    // Claude Code ends it and sends the list without it.
    process.emit(live(json!([]))).await;
    settle().await;
    assert!(bot(&s).background.is_empty());
}

#[tokio::test(start_paused = true)]
async fn the_list_ends_with_the_process() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    process.emit(live(json!([command("b1")]))).await;
    settle().await;
    assert_eq!(bot(&s).background.len(), 1);

    process.exit(1).await;
    settle().await;
    assert!(bot(&s).background.is_empty());
}
