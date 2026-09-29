//! Routines (spec 20): the scheduler that makes bots work on their own.
//!
//! It sleeps until the soonest routine comes due (at most a minute, to
//! follow clock changes and the return from sleep) or until woken, then
//! fires what is due: a message on the bot's usual path (spec 9.1). A run
//! ends with the turn that began with its message.

mod cron;
mod fire;
pub mod schedule;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::ids::{BotId, MessageId, RoutineRunId};
use botloft_core::protocol::RunStatus;
use tokio::sync::Notify;
use tracing::warn;

pub use self::fire::{fire_now, next_run_at};
use crate::state::{Daemon, Event};

/// The longest the scheduler sleeps, to follow clock changes and sleep.
const MAX_SLEEP: Duration = Duration::from_secs(60);

#[derive(Default)]
pub struct Routines {
    wake: Notify,
    /// The run whose message began each bot's current turn.
    turns: Mutex<HashMap<BotId, RoutineRunId>>,
}

impl Routines {
    /// Looks at the routines now instead of at the next time: one was
    /// created, changed, turned on or off.
    pub fn wake(&self) {
        self.wake.notify_one();
    }

    fn turns(&self) -> MutexGuard<'_, HashMap<BotId, RoutineRunId>> {
        self.turns
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Runs the scheduler until the daemon stops. The first pass also ends the
/// runs whose turn a previous daemon left open.
pub async fn run(daemon: Arc<Daemon>) {
    fire::fail_stale(&daemon);
    loop {
        tick(&daemon);
        let sleep = until_next(&daemon);
        tokio::select! {
            () = tokio::time::sleep(sleep) => {}
            () = daemon.routines.wake.notified() => {}
        }
    }
}

/// One pass: ends runs whose message died and fires what is due. Tests call
/// it after moving the clock.
pub fn tick(daemon: &Daemon) {
    let now = daemon.clock.now_ms();
    fire::fail_undelivered(daemon, now);
    let due = match daemon.store().due_routines(now) {
        Ok(due) => due,
        Err(err) => {
            warn!("could not read the routines that are due: {err}");
            return;
        }
    };
    for routine in due {
        fire::due(daemon, routine, now);
    }
}

fn until_next(daemon: &Daemon) -> Duration {
    let next = daemon.store().next_routine_at().ok().flatten();
    let Some(next) = next else {
        return MAX_SLEEP;
    };
    let left = u64::try_from(next.saturating_sub(daemon.clock.now_ms())).unwrap_or(0);
    Duration::from_millis(left).min(MAX_SLEEP)
}

/// The bot began the turn for `message`: remember the run it belongs to.
pub fn turn_began(daemon: &Daemon, bot: &BotId, message: &MessageId) {
    let run = daemon.store().run_of_message(message).ok().flatten();
    let mut turns = daemon.routines.turns();
    match run {
        Some(run) if run.status == RunStatus::Queued => {
            turns.insert(bot.clone(), run.id);
        }
        _ => {
            turns.remove(bot);
        }
    }
}

/// The bot's turn ended (`result`): its run is done, or failed on an error.
pub fn turn_ended(daemon: &Daemon, bot: &BotId, failed: bool) {
    let Some(run) = daemon.routines.turns().remove(bot) else {
        return;
    };
    let status = if failed {
        RunStatus::Failed
    } else {
        RunStatus::Done
    };
    finish(daemon, &run, status);
}

/// The bot's process ended mid-turn: that run will not finish.
pub fn process_ended(daemon: &Daemon, bot: &BotId) {
    if let Some(run) = daemon.routines.turns().remove(bot) {
        finish(daemon, &run, RunStatus::Failed);
    }
}

fn finish(daemon: &Daemon, run: &RoutineRunId, status: RunStatus) {
    let now = daemon.clock.now_ms();
    let finished = daemon.store().finish_run(run, status, now);
    match finished {
        Ok(Some(run)) => {
            daemon.emit(Event::RoutineRun(run.clone()));
            fire::announce_routine(daemon, &run.routine_id);
        }
        Ok(None) => {}
        Err(err) => warn!(run = %run, "could not end a routine run: {err}"),
    }
}
