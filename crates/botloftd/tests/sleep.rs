//! Bots that sleep (spec 7.7), with the fake runtime: a bot with nothing to
//! do starts asleep and wakes for a message; an idle one sleeps after the
//! owner's time and wakes with the same conversation; a background command
//! keeps it awake; a restart wakes it; and with sleep off it never sleeps.

mod common;

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use common::{Client, TestDaemon, stream};
use serde_json::{Value, json};

const NAP: Duration = Duration::from_millis(300);

async fn crew_bot(t: &TestDaemon, app: &mut Client) -> Value {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    t.until_state(&bot, BotState::Sleeping).await;
    bot
}

async fn send(app: &mut Client, bot: &Value, body: &str) {
    let message = json!({ "botId": bot["id"], "body": body });
    app.call("messages.send", message).await.expect("send");
}

/// The `n`th process the runtime started, waiting up to 5 s for it.
async fn nth_process(t: &TestDaemon, n: usize) -> botloftd::runtime::fake::FakeProcess {
    for _ in 0..500 {
        if let Some(process) = t.runtime.processes().into_iter().nth(n) {
            return process;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("process {n} never started");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_bot_sleeps_when_idle_and_wakes_with_its_conversation() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(NAP));
    let mut app = t.session().await;

    // Nothing for it yet: it starts asleep, with no process.
    let bot = crew_bot(&t, &mut app).await;
    assert!(t.runtime.processes().is_empty());

    // A message wakes it, and it answers.
    send(&mut app, &bot, "Hello").await;
    let first = nth_process(&t, 0).await;
    let line = first.wait_lines(1).await.pop().expect("line");
    // The turn opens in a conversation Claude Code keeps on disk.
    let mut opened = stream::replay(&line);
    opened["session_id"] = json!("session-1");
    first.emit(stream::init("session-1")).await;
    first.emit(opened).await;
    first.emit(stream::text("Hi")).await;
    first.emit(stream::result(false)).await;

    // Idle for the owner's time: it sleeps, and its process is gone.
    t.until_state(&bot, BotState::Sleeping).await;
    assert!(first.killed());

    // The next message wakes it, resuming the same conversation.
    send(&mut app, &bot, "Again").await;
    let second = nth_process(&t, 1).await;
    let args = second.args();
    let at = args
        .iter()
        .position(|arg| arg == "--resume")
        .unwrap_or_else(|| panic!("{args:?}"));
    assert_eq!(args.get(at + 1).map(String::as_str), Some("session-1"));
    let line = second.wait_lines(1).await.pop().expect("line");
    assert!(line.to_string().contains("Again"), "{line}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_background_command_keeps_a_bot_awake() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(NAP));
    let mut app = t.session().await;
    let bot = crew_bot(&t, &mut app).await;
    send(&mut app, &bot, "Start the dev server").await;
    let process = nth_process(&t, 0).await;
    let line = process.wait_lines(1).await.pop().expect("line");
    let tasks = |tasks: Value| json!({ "type": "system", "subtype": "background_tasks_changed", "tasks": tasks });
    process
        .emit(tasks(
            json!([{ "task_id": "b1", "task_type": "local_bash", "status": "running" }]),
        ))
        .await;
    stream::answer(&process, &line, "It runs").await;
    t.until_state(&bot, BotState::Idle).await;

    // Well past its time, the command keeps it awake.
    tokio::time::sleep(NAP * 4).await;
    assert!(!process.killed());
    let id: BotId = bot["id"].as_str().expect("id").parse().expect("bot id");
    let state = t.daemon.supervisor.status(&id).map(|(state, _)| state);
    assert_eq!(state, Some(BotState::Idle));

    // Once the command ends, it sleeps.
    process.emit(tasks(json!([]))).await;
    t.until_state(&bot, BotState::Sleeping).await;
    assert!(process.killed());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_restart_wakes_a_sleeping_bot_and_sleep_off_never_sleeps() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(NAP));
    let mut app = t.session().await;
    let bot = crew_bot(&t, &mut app).await;

    // The owner restarts it: it starts, then sleeps again with nothing to do.
    app.call("bots.restart", json!({ "botId": bot["id"] }))
        .await
        .expect("restart");
    let first = nth_process(&t, 0).await;
    t.until_state(&bot, BotState::Idle).await;
    t.until_state(&bot, BotState::Sleeping).await;
    assert!(first.killed());

    // Sleep turned off: it starts like any bot and stays up.
    t.daemon.settings.set_sleep_after(None);
    t.daemon.supervisor.wake();
    let second = nth_process(&t, 1).await;
    t.until_state(&bot, BotState::Idle).await;
    tokio::time::sleep(NAP * 4).await;
    assert!(!second.killed());
}
