//! One bot under a supervisor on a FakeRuntime, driven directly through the
//! daemon (no WebSocket), for the supervisor tests.

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotState, BotsCreateParams, CrewsCreateParams};
use botloftd::clock::ManualClock;
use botloftd::runtime::fake::{FakeProcess, FakeRuntime};
use botloftd::service::{bots, crews};
use botloftd::state::Daemon;
use botloftd::supervisor;
use serde_json::json;

use super::{new_daemon, stream, test_settings};

pub struct Setup {
    pub daemon: Arc<Daemon>,
    pub runtime: FakeRuntime,
    pub clock: Arc<ManualClock>,
    pub bot: BotId,
    _dir: tempfile::TempDir,
}

pub async fn setup() -> Setup {
    let parts = new_daemon(test_settings());
    let (daemon, runtime, clock, dir) = (parts.daemon, parts.runtime, parts.clock, parts.dir);
    let crew = crews::create(
        &daemon,
        CrewsCreateParams {
            name: "Ops".into(),
            work_folder: None,
            lead: None,
        },
    )
    .expect("crew");
    let bot = bots::create(
        &daemon,
        BotsCreateParams {
            crew_id: crew.id,
            name: "Scout".into(),
            role: String::new(),
            instructions: String::new(),
            color: None,
            model: None,
        },
    )
    .expect("bot");
    tokio::spawn(supervisor::run(Arc::clone(&daemon)));
    Setup {
        daemon,
        runtime,
        clock,
        bot: bot.id,
        _dir: dir,
    }
}

impl Setup {
    pub fn state(&self) -> BotState {
        self.daemon
            .supervisor
            .status(&self.bot)
            .map(|(state, _)| state)
            .expect("slot")
    }

    /// Waits (on the paused clock) until the bot reaches `state`.
    pub async fn until(&self, state: BotState) {
        for _ in 0..600 {
            if self.daemon.supervisor.status(&self.bot).map(|(s, _)| s) == Some(state) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("bot never reached {state:?}; it is {:?}", self.state());
    }

    /// Writes a message line the way the courier does.
    pub fn message(&self, text: &str) -> u64 {
        let line =
            json!({ "type": "user", "uuid": "u", "message": { "role": "user", "content": text } });
        self.daemon
            .supervisor
            .write_message(&self.bot, "u", format!("{line}\n").into())
            .expect("running")
    }

    /// One whole turn in the conversation `process` is in. Returns its
    /// session id, which Claude Code now has on disk (spec 7.3).
    pub async fn turn(&self, process: &FakeProcess) -> String {
        let session = session_of(process);
        self.message("hi");
        process.emit(stream::init(&session)).await;
        process.emit(stream::began(&session)).await;
        process.emit(stream::result(false)).await;
        self.until(BotState::Idle).await;
        session
    }
}

/// The conversation the process started or resumed.
pub fn session_of(process: &FakeProcess) -> String {
    arg_after(process, "--session-id")
        .or_else(|| arg_after(process, "--resume"))
        .expect("a session")
}

/// The value after `flag` on the process's command line.
pub fn arg_after(process: &FakeProcess, flag: &str) -> Option<String> {
    let args = process.args();
    let at = args.iter().position(|arg| arg == flag)?;
    args.get(at + 1).cloned()
}
