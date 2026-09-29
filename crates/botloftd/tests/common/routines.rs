//! Routines on a FakeRuntime with a manual clock: a running bot with the
//! courier delivering to it, and helpers to create routines and move time.

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::RoutineId;
use botloft_core::protocol::{
    BotState, Missed, Overlap, Routine, RoutineRun, RoutinesCreateParams, RoutinesRunsParams,
    Schedule,
};
use botloftd::courier;
use botloftd::routines::{self, schedule::at};
use botloftd::service::routines as service;
use serde_json::Value;

use super::stream;
use super::supervised::{Setup, setup};

pub const MINUTE: Duration = Duration::from_secs(60);

/// A running bot, the courier delivering to it, and the clock on
/// 2026-10-01 08:00 UTC.
pub async fn routines_setup() -> Setup {
    let s = setup().await;
    s.clock.set(at("UTC", 2026, 10, 1, 8, 0));
    tokio::spawn(courier::run(Arc::clone(&s.daemon)));
    s.runtime.process(1).await;
    s.until(BotState::Idle).await;
    s
}

pub fn create(s: &Setup, schedule: Schedule, overlap: Overlap, missed: Missed) -> Routine {
    service::create(
        &s.daemon,
        RoutinesCreateParams {
            bot_id: s.bot.clone(),
            name: "Morning".into(),
            prompt: "Summarize what arrived in shared/inbox.".into(),
            schedule,
            timezone: "UTC".into(),
            overlap: Some(overlap),
            missed: Some(missed),
        },
    )
    .expect("create")
}

pub fn every(minutes: u32) -> Schedule {
    Schedule::Interval { minutes }
}

pub fn runs(s: &Setup, routine: &RoutineId) -> Vec<RoutineRun> {
    let mut runs = service::runs(
        &s.daemon,
        RoutinesRunsParams {
            routine_id: routine.clone(),
            before: None,
            limit: None,
        },
    )
    .expect("runs");
    runs.reverse();
    runs
}

pub fn routine(s: &Setup, id: &RoutineId) -> Routine {
    s.daemon
        .store()
        .routine(id)
        .expect("read")
        .expect("routine")
}

/// Moves the clock and runs the scheduler once.
pub fn tick_after(s: &Setup, by: Duration) {
    s.clock.advance(by);
    routines::tick(&s.daemon);
}

/// The routine messages written to the bot so far, waiting for `n`.
pub async fn written(s: &Setup, n: usize) -> Vec<Value> {
    s.runtime.process(1).await.wait_lines(n).await
}

/// Plays the bot starting and ending the turn of stdin line `line`.
pub async fn finish_turn(s: &Setup, line: &Value, failed: bool) {
    let process = s.runtime.process(1).await;
    process.emit(stream::replay(line)).await;
    process.emit(stream::result(failed)).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
}
