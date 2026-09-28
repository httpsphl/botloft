//! Running fake bots: finding their process, reporting their inbox through
//! `/hooks` and a ready-made crew of two.

use std::time::Duration;

use botloftd::hooks::client::post;
use botloftd::runtime::fake::FakeProcess;
use serde_json::{Value, json};

use super::mcp::Mcp;
use super::{Client, TestDaemon};

impl TestDaemon {
    /// The fake process of `bot`, waiting up to 5 s for it to start.
    pub async fn process_of(&self, bot: &Value) -> FakeProcess {
        let id = bot["id"].as_str().expect("bot id");
        for _ in 0..500 {
            let found = self
                .runtime
                .processes()
                .into_iter()
                .rev()
                .find(|process| process.env("BOTLOFT_BOT_ID").as_deref() == Some(id));
            if let Some(process) = found {
                return process;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("bot {id} never started");
    }
}

/// Inbox address fake bot `n` reports.
pub fn pipe(n: u32) -> String {
    format!(r"\\.\pipe\LOCAL\cc-msg-test-{n}")
}

/// Sends `SessionStart` through `/hooks`, as the bot's hook would, with the
/// inbox `pipe(n)` and messaging token `tok-n`.
pub async fn session_start(t: &TestDaemon, process: &FakeProcess, n: u32) {
    let token = process.env("BOTLOFT_BOT_TOKEN").expect("bot token");
    let body = json!({
        "payload": { "hook_event_name": "SessionStart" },
        "messagingSocket": pipe(n),
        "messagingToken": format!("tok-{n}"),
    });
    let body = serde_json::to_vec(&body).expect("json");
    let port = t.addr.port();
    tokio::task::spawn_blocking(move || post(port, "/hooks/session-start", &token, &body))
        .await
        .expect("join")
        .expect("hook accepted");
}

/// Creates a bot, waits for its process and reports inbox `n`. Returns the
/// bot's MCP client.
pub async fn ready_bot(t: &TestDaemon, app: &mut Client, crew: &Value, name: &str, n: u32) -> Mcp {
    let role = format!("{name} role");
    let bot = json!({ "crewId": crew["id"], "name": name, "role": role, "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    let process = t.process_of(&bot).await;
    session_start(t, &process, n).await;
    Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"))
}

/// A running crew "Ops" with @lead (inbox 1) and @writer (inbox 2).
pub struct Crew {
    pub t: TestDaemon,
    pub app: Client,
    pub crew: Value,
    pub lead: Mcp,
    pub writer: Mcp,
}

pub async fn two_bots() -> Crew {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let lead = ready_bot(&t, &mut app, &crew, "Lead", 1).await;
    let writer = ready_bot(&t, &mut app, &crew, "Writer", 2).await;
    Crew {
        t,
        app,
        crew,
        lead,
        writer,
    }
}
