//! Agents and the machine's memory (spec 42.4), on the fake runtime with a
//! fake memory: an agent starts only when the free memory takes it, an idle
//! one sleeps sooner on a small machine, and with memory short only one
//! idle agent stays awake.

mod common;

use std::time::Duration;

use botloft_core::protocol::BotState;
use botloftd::memory::MemoryInfo;
use common::{Client, TestDaemon, stream};
use serde_json::{Value, json};

const GB: u64 = 1024 * 1024 * 1024;
const HOUR: Duration = Duration::from_secs(3600);

/// Two agents in one crew, both asleep with nothing to do.
async fn two_asleep(t: &TestDaemon, app: &mut Client) -> (Value, Value) {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let mut bots = Vec::new();
    for name in ["Scout", "Writer"] {
        let bot = json!({ "crewId": crew["id"], "name": name, "role": "", "instructions": "" });
        let bot = app.call("bots.create", bot).await.expect("bot");
        t.until_state(&bot, BotState::Sleeping).await;
        bots.push(bot);
    }
    let writer = bots.pop().expect("writer");
    (bots.pop().expect("scout"), writer)
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

/// Wakes `bot` with a message and plays its answer; it ends idle.
async fn answers(t: &TestDaemon, app: &mut Client, bot: &Value, n: usize) {
    send(app, bot, "Hello").await;
    let process = nth_process(t, n).await;
    let line = process.wait_lines(1).await.pop().expect("line");
    stream::answer(&process, &line, "Hi").await;
    t.until_state(bot, BotState::Idle).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_starts_only_when_the_free_memory_takes_it() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(HOUR));
    t.memory.set_room(16 * GB, 1);
    let mut app = t.session().await;
    let (scout, writer) = two_asleep(&t, &mut app).await;
    answers(&t, &mut app, &scout, 0).await;

    // The scout now uses the memory there was.
    t.memory.set_room(16 * GB, 0);
    send(&mut app, &writer, "Your turn").await;
    // The idle scout sleeps to free it; the writer waits until it is free.
    t.until_state(&scout, BotState::Sleeping).await;
    let first = nth_process(&t, 0).await;
    assert!(first.killed(), "the idle one made room");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(t.runtime.processes().len(), 1, "no room yet for the writer");

    // Its memory is back: the writer starts with its message.
    t.memory.set_room(16 * GB, 1);
    t.daemon.supervisor.wake();
    let second = nth_process(&t, 1).await;
    let line = second.wait_lines(1).await.pop().expect("line");
    assert!(line.to_string().contains("Your turn"), "{line}");
}

#[tokio::test(flavor = "multi_thread")]
async fn on_a_small_machine_an_idle_agent_sleeps_sooner() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(HOUR));
    t.memory.set_small_machine_nap(Duration::from_millis(300));
    t.memory.set(Some(MemoryInfo {
        total: GB,
        available: 900 * 1024 * 1024,
    }));
    let mut app = t.session().await;
    let (scout, _writer) = two_asleep(&t, &mut app).await;
    answers(&t, &mut app, &scout, 0).await;
    // Well before the owner's hour, the small machine puts it to sleep.
    t.until_state(&scout, BotState::Sleeping).await;
    assert!(nth_process(&t, 0).await.killed());
}

#[tokio::test(flavor = "multi_thread")]
async fn short_of_memory_on_a_server_only_one_idle_agent_stays_awake() {
    let t = TestDaemon::start_on_a_server().await;
    t.daemon.settings.set_sleep_after(Some(HOUR));
    t.memory.set(Some(MemoryInfo {
        total: 16 * GB,
        available: 15 * GB,
    }));
    let mut app = t.session().await;
    let (scout, writer) = two_asleep(&t, &mut app).await;
    answers(&t, &mut app, &scout, 0).await;
    answers(&t, &mut app, &writer, 1).await;

    // Less than half free: the one idle the longest sleeps.
    t.memory.set(Some(MemoryInfo {
        total: 16 * GB,
        available: 4 * GB,
    }));
    t.daemon.supervisor.wake();
    t.until_state(&scout, BotState::Sleeping).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(nth_process(&t, 0).await.killed());
    assert!(
        !nth_process(&t, 1).await.killed(),
        "the last one stays awake"
    );
}
