//! Running fake bots: finding their process, waiting for their state and a
//! ready-made crew of two.

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use botloftd::runtime::fake::FakeProcess;
use serde_json::{Value, json};

use super::mcp::Mcp;
use super::{Client, TestDaemon};

impl TestDaemon {
    /// The newest fake process of `bot`, waiting up to 5 s for it to start.
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

    /// Waits up to 5 s for the bot to reach `state`.
    pub async fn until_state(&self, bot: &Value, state: BotState) {
        let id: BotId = bot["id"].as_str().expect("bot id").parse().expect("bot id");
        let mut last = None;
        for _ in 0..500 {
            last = self.daemon.supervisor.status(&id).map(|(s, _)| s);
            if last == Some(state) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("bot never reached {state:?}; it is {last:?}");
    }
}

/// Creates a bot and waits until its process is ready. Returns the bot, its
/// process and its MCP client.
pub async fn ready_bot(
    t: &TestDaemon,
    app: &mut Client,
    crew: &Value,
    name: &str,
) -> (Value, FakeProcess, Mcp) {
    let role = format!("{name} role");
    let bot = json!({ "crewId": crew["id"], "name": name, "role": role, "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    let process = t.process_of(&bot).await;
    t.until_state(&bot, BotState::Idle).await;
    let mcp = Mcp::new(t.addr, process.env("BOTLOFT_BOT_TOKEN").expect("token"));
    (bot, process, mcp)
}

/// A running crew "Ops" with @lead and @writer.
pub struct Crew {
    pub t: TestDaemon,
    pub app: Client,
    pub crew: Value,
    pub lead: Mcp,
    pub lead_process: FakeProcess,
    pub writer: Mcp,
    pub writer_process: FakeProcess,
}

pub async fn two_bots() -> Crew {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (_, lead_process, lead) = ready_bot(&t, &mut app, &crew, "Lead").await;
    let (_, writer_process, writer) = ready_bot(&t, &mut app, &crew, "Writer").await;
    Crew {
        t,
        app,
        crew,
        lead,
        lead_process,
        writer,
        writer_process,
    }
}

/// The text a bot got in the message written as stdin line `line`.
pub fn text_of(line: &Value) -> String {
    line["message"]["content"][0]["text"]
        .as_str()
        .expect("text block")
        .to_owned()
}
