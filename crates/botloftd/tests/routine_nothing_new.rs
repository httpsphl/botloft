//! A routine run with nothing new to report (spec 20.5): the bot calls
//! `nothing_new` in the run's turn, and the run ends done and marked, with
//! no report for the owner to read.

mod common;

use std::time::Duration;

use botloft_core::protocol::{Missed, Overlap, RunStatus};
use botloftd::routines;
use common::bots::ready_bot;
use common::routines::*;
use common::{TestDaemon, stream};
use serde_json::json;

#[tokio::test(start_paused = true)]
async fn the_run_of_the_turn_takes_the_mark_and_keeps_it_when_done() {
    let s = routines_setup().await;
    let routine = create(&s, every(5), Overlap::Skip, Missed::RunOnce);
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 1).await;
    // The message waits: no turn carries the run yet.
    assert!(!routines::mark_nothing_new(
        &s.daemon,
        &botloftd::seat::Seat::main(&s.bot)
    ));

    let process = s.runtime.process(1).await;
    process.emit(stream::replay(&lines[0])).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(routines::mark_nothing_new(
        &s.daemon,
        &botloftd::seat::Seat::main(&s.bot)
    ));
    process.emit(stream::result(false)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let seen = runs(&s, &routine.id);
    assert_eq!(seen[0].status, RunStatus::Done);
    assert!(seen[0].nothing_new, "it ended quietly");
    // Once the turn is over, nothing is left to mark.
    assert!(!routines::mark_nothing_new(
        &s.daemon,
        &botloftd::seat::Seat::main(&s.bot)
    ));

    // The next run starts without the mark and keeps none if the bot reports.
    tick_after(&s, 5 * MINUTE);
    let lines = written(&s, 2).await;
    finish_turn(&s, &lines[1], false).await;
    let seen = runs(&s, &routine.id);
    assert_eq!(seen[1].status, RunStatus::Done);
    assert!(!seen[1].nothing_new);
}

#[tokio::test]
async fn outside_a_routine_the_tool_says_so() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Office" }))
        .await
        .expect("crew");
    let (_bot, _process, mut mcp) = ready_bot(&t, &mut app, &crew, "Mail").await;
    let refused = mcp
        .tool("nothing_new", json!({}))
        .await
        .expect_err("no routine run");
    assert!(refused.contains("No routine run is open"), "{refused}");
}
