//! Supervisor lifecycle on a FakeRuntime and a paused clock (spec 7 and 16):
//! starting, restarts, sessions, pausing and archiving.

mod common;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::protocol::{
    BotIdParams, BotState, BotsCreateParams, BotsRestartParams, BotsSetPausedParams,
    CrewsCreateParams,
};
use botloftd::service::{bots, crews};
use botloftd::supervisor;
use common::stream;
use common::supervised::{arg_after, setup};
use common::{new_daemon, test_settings};

#[tokio::test(start_paused = true)]
async fn a_new_bot_runs_headless_in_a_new_session_with_a_clean_environment() {
    let s = setup().await;
    let process = s.runtime.process(1).await;
    s.until(BotState::Launching).await;
    s.until(BotState::Idle).await;

    let args = process.args();
    assert_eq!(args[0], "-p");
    assert_eq!(
        arg_after(&process, "--input-format").as_deref(),
        Some("stream-json")
    );
    assert_eq!(
        arg_after(&process, "--output-format").as_deref(),
        Some("stream-json")
    );
    for flag in [
        "--verbose",
        "--include-partial-messages",
        "--replay-user-messages",
        "--strict-mcp-config",
    ] {
        assert!(
            args.iter().any(|arg| arg == flag),
            "{flag} missing from {args:?}"
        );
    }
    let session = arg_after(&process, "--session-id").expect("a new session");
    assert_eq!(session.len(), 36);
    assert_eq!(arg_after(&process, "--resume"), None);
    assert_eq!(
        arg_after(&process, "--setting-sources").as_deref(),
        Some("project,local")
    );
    assert_eq!(
        arg_after(&process, "--permission-mode").as_deref(),
        Some("default")
    );
    assert_eq!(
        arg_after(&process, "--permission-prompt-tool").as_deref(),
        Some("mcp__botloft__permission_prompt")
    );
    assert_eq!(
        arg_after(&process, "--allowedTools").as_deref(),
        Some("mcp__botloft")
    );
    assert!(arg_after(&process, "--mcp-config").is_some_and(|path| path.ends_with("mcp.json")));
    assert!(process.spec.cwd.ends_with("scout"));
    assert_eq!(
        process.env("BOTLOFT_BOT_ID").as_deref(),
        Some(s.bot.as_str())
    );
    assert_eq!(process.env("BOTLOFT_PORT").as_deref(), Some("45710"));
    assert_eq!(
        process.env("CLAUDECODE"),
        None,
        "no Claude session variables leak in"
    );
    assert_eq!(process.env("CLAUDE_CODE_MESSAGING_SOCKET"), None);
}

#[tokio::test(start_paused = true)]
async fn a_crash_restarts_with_backoff_in_the_same_session_and_a_new_token() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = arg_after(&first, "--session-id").expect("session");
    s.message("hi");
    first.emit(stream::init(&session)).await;
    let old_token = first.env("BOTLOFT_BOT_TOKEN").expect("token");
    let (_, first_generation) = s.daemon.supervisor.status(&s.bot).expect("slot");

    tokio::time::sleep(Duration::from_secs(60)).await;
    first.exit(1).await;
    s.until(BotState::Backoff).await;

    let second = s.runtime.process(2).await;
    s.until(BotState::Launching).await;
    assert_eq!(arg_after(&second, "--resume"), Some(session.clone()));
    assert_eq!(arg_after(&second, "--session-id"), None);
    assert!(
        s.daemon.supervisor.token_owner(&old_token).is_none(),
        "old token is revoked"
    );
    let (_, generation) = s.daemon.supervisor.status(&s.bot).expect("slot");
    assert!(generation > first_generation);
    let stored = s.daemon.store().session_id(&s.bot).expect("read");
    assert_eq!(
        stored,
        Some(session),
        "the session survives a daemon restart too"
    );
}

#[tokio::test(start_paused = true)]
async fn a_resumed_session_that_dies_at_once_comes_back_fresh() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = arg_after(&first, "--session-id").expect("session");
    first.exit(0).await;

    let resumed = s.runtime.process(2).await;
    assert_eq!(arg_after(&resumed, "--resume"), Some(session.clone()));
    resumed.exit(1).await;

    let fresh = s.runtime.process(3).await;
    let new_session = arg_after(&fresh, "--session-id").expect("a new session");
    assert_ne!(new_session, session);
}

#[tokio::test(start_paused = true)]
async fn pausing_stops_the_bot_and_resuming_starts_it() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;

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
    s.until(BotState::Idle).await;
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
async fn a_fresh_restart_starts_a_new_session() {
    let s = setup().await;
    let first = s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    let session = arg_after(&first, "--session-id").expect("session");

    let restart = BotsRestartParams {
        bot_id: s.bot.clone(),
        fresh: Some(true),
    };
    bots::restart(&s.daemon, restart).expect("restart");
    let second = s.runtime.process(2).await;
    assert!(first.killed());
    let new_session = arg_after(&second, "--session-id").expect("a new session");
    assert_ne!(new_session, session);
}

#[tokio::test(start_paused = true)]
async fn a_failed_spawn_backs_off_and_retries() {
    let parts = new_daemon(test_settings());
    let (daemon, runtime) = (parts.daemon, parts.runtime);
    runtime.fail_next_spawn();
    let crew = crews::create(
        &daemon,
        CrewsCreateParams {
            name: "Ops".into(),
            work_folder: None,
        },
    )
    .expect("crew");
    let params = BotsCreateParams {
        crew_id: crew.id,
        name: "Scout".into(),
        role: String::new(),
        instructions: String::new(),
        color: None,
        model: None,
    };
    let bot = bots::create(&daemon, params).expect("bot");
    tokio::spawn(supervisor::run(Arc::clone(&daemon)));
    runtime.process(1).await;
    let state = daemon.supervisor.status(&bot.id).map(|(state, _)| state);
    assert_eq!(state, Some(BotState::Launching));
}
