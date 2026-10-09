//! A bot's files and process are made without the supervisor lock, so a
//! slow start does not stall the other bots, the courier or the app.

mod common;

use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotState, BotsCreateParams, BotsDeleteParams, CrewsCreateParams};
use botloftd::service::{bots, crews, delete};
use botloftd::state::Daemon;
use botloftd::supervisor;

use common::{Parts, new_daemon, test_settings};

fn add_bot(daemon: &Daemon) -> BotId {
    let crew = crews::create(
        daemon,
        CrewsCreateParams {
            name: "Ops".into(),
            work_folder: None,
            lead: None,
        },
    )
    .expect("crew");
    bots::create(
        daemon,
        BotsCreateParams {
            crew_id: crew.id,
            name: "Scout".into(),
            role: String::new(),
            instructions: String::new(),
            color: None,
            model: None,
            agent: None,
        },
    )
    .expect("bot")
    .id
}

async fn wait_for(began: Receiver<()>) {
    tokio::task::spawn_blocking(move || began.recv_timeout(Duration::from_secs(5)))
        .await
        .expect("join")
        .expect("the spawn began");
}

/// Runs `f` on its own thread; `None` when it is still blocked after 2 s.
fn within<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> Option<T> {
    let (sent, got) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sent.send(f());
    });
    got.recv_timeout(Duration::from_secs(2)).ok()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_supervisor_answers_while_a_bot_starts() {
    let Parts {
        daemon,
        runtime,
        dir: _dir,
        ..
    } = new_daemon(test_settings());
    let (began, release) = runtime.hold_next_spawn();
    let bot = add_bot(&daemon);
    tokio::spawn(supervisor::run(Arc::clone(&daemon)));
    wait_for(began).await;

    let status = within({
        let daemon = Arc::clone(&daemon);
        let bot = bot.clone();
        move || daemon.supervisor.status(&bot)
    });
    release.send(()).expect("release");
    let (state, _) = status
        .expect("the supervisor lock is free while the process is made")
        .expect("slot");
    assert_ne!(state, BotState::Launching, "not recorded until it exists");

    runtime.process(1).await;
    for _ in 0..200 {
        if daemon.supervisor.status(&bot).map(|(s, _)| s) == Some(BotState::Launching) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("the started process was never recorded");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_bot_deleted_while_it_starts_does_not_keep_its_process() {
    let Parts {
        daemon,
        runtime,
        dir: _dir,
        ..
    } = new_daemon(test_settings());
    let (began, release) = runtime.hold_next_spawn();
    let bot = add_bot(&daemon);
    tokio::spawn(supervisor::run(Arc::clone(&daemon)));
    wait_for(began).await;

    let deleted = within({
        let daemon = Arc::clone(&daemon);
        let bot = bot.clone();
        move || {
            delete::bot(
                &daemon,
                BotsDeleteParams {
                    bot_id: bot,
                    recycle_folder: None,
                },
            )
        }
    });
    release.send(()).expect("release");
    deleted
        .expect("deleting does not wait for the start")
        .expect("deleted");

    let process = runtime.process(1).await;
    for _ in 0..200 {
        if process.killed() {
            assert_eq!(daemon.supervisor.status(&bot), None);
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("the process of a deleted bot was left running");
}
