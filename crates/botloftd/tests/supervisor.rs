//! Supervisor behavior on a FakeRuntime and a paused clock (spec 7 and 16).

mod common;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    BotIdParams, BotState, BotsCreateParams, BotsRestartParams, BotsSetPausedParams,
    CrewsCreateParams,
};
use botloftd::runtime::fake::{FakeProcess, FakeRuntime};
use botloftd::service::{bots, crews};
use botloftd::state::Daemon;
use botloftd::supervisor::{self, Hook};
use common::{new_daemon, test_settings};

struct Setup {
    daemon: Arc<Daemon>,
    runtime: FakeRuntime,
    bot: BotId,
    _dir: tempfile::TempDir,
}

async fn setup() -> Setup {
    let parts = new_daemon(test_settings());
    let (daemon, runtime, dir) = (parts.daemon, parts.runtime, parts.dir);
    let crew = crews::create(&daemon, CrewsCreateParams { name: "Ops".into() }).expect("crew");
    let bot = bots::create(
        &daemon,
        BotsCreateParams {
            crew_id: crew.id,
            name: "Scout".into(),
            role: String::new(),
            instructions: String::new(),
            color: None,
        },
    )
    .expect("bot");
    tokio::spawn(supervisor::run(Arc::clone(&daemon)));
    Setup {
        daemon,
        runtime,
        bot: bot.id,
        _dir: dir,
    }
}

impl Setup {
    fn state(&self) -> BotState {
        self.daemon
            .supervisor
            .status(&self.bot)
            .map(|(state, _)| state)
            .expect("slot")
    }

    /// Waits (on the paused clock) until the bot reaches `state`.
    async fn until(&self, state: BotState) {
        for _ in 0..600 {
            if self.daemon.supervisor.status(&self.bot).map(|(s, _)| s) == Some(state) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("bot never reached {state:?}; it is {:?}", self.state());
    }

    /// Sends a hook the way `/hooks` does: with the process's own token.
    fn hook(&self, process: &FakeProcess, hook: Hook) {
        let token = process.env("BOTLOFT_BOT_TOKEN").expect("token in env");
        let (bot, generation) = self
            .daemon
            .supervisor
            .hook_owner(&token)
            .expect("live token");
        self.daemon.supervisor.on_hook(&bot, generation, hook);
    }
}

#[tokio::test(start_paused = true)]
async fn a_new_bot_starts_with_the_spec_command_line_and_a_clean_environment() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Launching).await;

    let args = process.args();
    assert_eq!(
        args[0], "--settings",
        "first start has nothing to --continue"
    );
    assert!(args[1].ends_with("settings.json") && args[2] == "--mcp-config");
    assert!(process.spec.cwd.ends_with("scout"));
    assert_eq!(
        process.env("BOTLOFT_BOT_ID").as_deref(),
        Some(s.bot.as_str())
    );
    assert_eq!(process.env("BOTLOFT_PORT").as_deref(), Some("45710"));
    assert!(process.env("BOTLOFT_HOME").is_some() && process.env("BOTLOFT_BIN").is_some());
    assert_eq!(
        process.env("CLAUDECODE"),
        None,
        "no Claude session variables leak in"
    );
    assert_eq!(process.env("CLAUDE_CODE_MESSAGING_SOCKET"), None);
}

#[tokio::test(start_paused = true)]
async fn hooks_drive_the_state_and_input_answers_a_permission_prompt() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Launching).await;

    s.hook(&process, Hook::SessionStart { inbox: None });
    assert_eq!(s.state(), BotState::Idle);
    s.hook(&process, Hook::PromptSubmit);
    assert_eq!(s.state(), BotState::Busy);
    s.hook(
        &process,
        Hook::Notification {
            kind: "permission_prompt".into(),
        },
    );
    assert_eq!(s.state(), BotState::NeedsApproval);
    s.daemon
        .supervisor
        .write(&s.bot, "1".into())
        .expect("write");
    assert_eq!(s.state(), BotState::Busy);
    assert_eq!(process.input(), b"1");
    s.hook(
        &process,
        Hook::StopFailure {
            error: "rate_limit".into(),
        },
    );
    assert_eq!(s.state(), BotState::RateLimited);
    s.hook(&process, Hook::Stop);
    assert_eq!(s.state(), BotState::Idle);
}

#[tokio::test(start_paused = true)]
async fn a_crash_restarts_with_backoff_and_continue_and_a_new_token() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Launching).await;
    s.hook(&first, Hook::SessionStart { inbox: None });
    let old_token = first.env("BOTLOFT_BOT_TOKEN").expect("token");
    let (_, first_generation) = s.daemon.supervisor.status(&s.bot).expect("slot");

    tokio::time::sleep(Duration::from_secs(60)).await;
    first.exit(1).await;
    s.until(BotState::Backoff).await;

    let second = s.runtime.process(2).await;
    s.until(BotState::Launching).await;
    assert_eq!(second.args()[0], "--continue");
    assert!(
        s.daemon.supervisor.hook_owner(&old_token).is_none(),
        "old token is revoked"
    );
    let (_, generation) = s.daemon.supervisor.status(&s.bot).expect("slot");
    assert!(generation > first_generation);
}

#[tokio::test(start_paused = true)]
async fn a_resumed_session_that_dies_at_once_comes_back_fresh() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Launching).await;
    s.hook(&first, Hook::SessionStart { inbox: None });
    first.exit(0).await;

    let resumed = s.runtime.process(2).await;
    assert_eq!(resumed.args()[0], "--continue");
    resumed.exit(1).await;

    let fresh = s.runtime.process(3).await;
    assert_eq!(fresh.args()[0], "--settings");
}

#[tokio::test(start_paused = true)]
async fn auth_errors_wait_for_the_owner() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Launching).await;
    s.hook(
        &process,
        Hook::StopFailure {
            error: "authentication_failed".into(),
        },
    );
    assert_eq!(s.state(), BotState::AuthError);
    process.exit(1).await;

    tokio::time::sleep(Duration::from_secs(600)).await;
    assert_eq!(s.runtime.processes().len(), 1, "no restart loop");
    assert_eq!(s.state(), BotState::AuthError);

    let restart = BotsRestartParams {
        bot_id: s.bot.clone(),
        fresh: None,
    };
    bots::restart(&s.daemon, restart).expect("restart");
    s.runtime.process(2).await;
    s.until(BotState::Launching).await;
}

#[tokio::test(start_paused = true)]
async fn pausing_stops_the_bot_and_resuming_starts_it() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Launching).await;

    let pause = |paused| BotsSetPausedParams {
        bot_id: s.bot.clone(),
        paused,
    };
    bots::set_paused(&s.daemon, pause(true)).expect("pause");
    s.until(BotState::Offline).await;
    assert!(first.killed());
    tokio::time::sleep(Duration::from_secs(60)).await;
    assert_eq!(s.runtime.processes().len(), 1);

    bots::set_paused(&s.daemon, pause(false)).expect("resume");
    s.runtime.process(2).await;
    s.until(BotState::Launching).await;
}

#[tokio::test(start_paused = true)]
async fn archiving_stops_the_bot_for_good() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Launching).await;
    bots::archive(
        &s.daemon,
        BotIdParams {
            bot_id: s.bot.clone(),
        },
    )
    .expect("archive");
    s.until(BotState::Archived).await;
    assert!(process.killed());
    tokio::time::sleep(Duration::from_secs(60)).await;
    assert_eq!(s.runtime.processes().len(), 1);
}

#[tokio::test(start_paused = true)]
async fn a_fresh_restart_replaces_the_process_without_continue() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Launching).await;
    s.hook(&first, Hook::SessionStart { inbox: None });

    let restart = BotsRestartParams {
        bot_id: s.bot.clone(),
        fresh: Some(true),
    };
    bots::restart(&s.daemon, restart).expect("restart");
    let second = s.runtime.process(2).await;
    assert!(first.killed());
    assert_eq!(second.args()[0], "--settings");
}

#[tokio::test(start_paused = true)]
async fn output_lands_in_the_terminal_and_the_size_carries_over() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Launching).await;
    // ConPTY's startup cursor query is answered by the daemon itself.
    first.output(b"\x1b[6nhello\r\n").await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(first.input(), b"\x1b[1;1R");

    let terminal = s.daemon.supervisor.terminal(&s.bot);
    let attached = terminal.attach(None, None);
    assert_eq!(&attached.replay[..], b"\x1b[6nhello\r\n");

    let size = botloftd::runtime::TermSize { cols: 80, rows: 24 };
    s.daemon.supervisor.resize(&s.bot, size);
    assert_eq!(first.sizes(), vec![size]);
    first.exit(1).await;
    let second = s.runtime.process(2).await;
    assert_eq!(second.spec.size, size);
}

#[tokio::test(start_paused = true)]
async fn a_failed_spawn_backs_off_and_retries() {
    let parts = new_daemon(test_settings());
    let (daemon, runtime) = (parts.daemon, parts.runtime);
    runtime.fail_next_spawn();
    let crew = crews::create(&daemon, CrewsCreateParams { name: "Ops".into() }).expect("crew");
    let params = BotsCreateParams {
        crew_id: crew.id,
        name: "Scout".into(),
        role: String::new(),
        instructions: String::new(),
        color: None,
    };
    let bot = bots::create(&daemon, params).expect("bot");
    tokio::spawn(supervisor::run(Arc::clone(&daemon)));
    runtime.process(1).await;
    let state = daemon.supervisor.status(&bot.id).map(|(state, _)| state);
    assert_eq!(state, Some(BotState::Launching));
}
