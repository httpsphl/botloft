//! Routines (spec 20): the scheduler that makes bots work on their own.
//!
//! It sleeps until the soonest routine comes due (at most a minute, to
//! follow clock changes and the return from sleep) or until woken, then
//! fires what is due: a message on the bot's usual path (spec 9.1). A run
//! ends with the turn that began with its message.

mod backoff;
mod cron;
mod fire;
pub mod schedule;
pub mod signal;
mod wait_page;
pub mod waits;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::ids::{BotId, MessageId, RoutineId, RoutineRunId};
use botloft_core::protocol::RunStatus;
use tokio::sync::Notify;
use tracing::warn;

pub use self::fire::{fire_now, next_run_at};
use self::waits::{PageTimes, SystemProbe, WaitProbe};
use crate::seat::Seat;
use crate::state::{Daemon, Event};

/// The longest the scheduler sleeps, to follow clock changes and sleep.
const MAX_SLEEP: Duration = Duration::from_secs(60);

pub struct Routines {
    wake: Notify,
    /// The runs whose messages each conversation's current turn took up.
    /// Messages that arrive during a turn may join it (spec 9.1, 40.4).
    turns: Mutex<HashMap<Seat, Vec<RoutineRunId>>>,
    /// What the waits look at (spec 20.14).
    probe: Mutex<Arc<dyn WaitProbe>>,
    /// When each wait on a page last read it.
    pages: Mutex<PageTimes>,
}

impl Default for Routines {
    fn default() -> Self {
        Self {
            wake: Notify::new(),
            turns: Mutex::default(),
            probe: Mutex::new(Arc::new(SystemProbe)),
            pages: Mutex::default(),
        }
    }
}

impl Routines {
    /// Looks at the routines now instead of at the next time: one was
    /// created, changed, turned on or off.
    pub fn wake(&self) {
        self.wake.notify_one();
    }

    fn turns(&self) -> MutexGuard<'_, HashMap<Seat, Vec<RoutineRunId>>> {
        self.turns
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// What the waits look at now.
    pub fn probe(&self) -> Arc<dyn WaitProbe> {
        Arc::clone(&lock(&self.probe))
    }

    /// Waits look at `probe` from now on; tests give their own.
    pub fn set_probe(&self, probe: Arc<dyn WaitProbe>) {
        *lock(&self.probe) = probe;
    }

    fn page_due(&self, id: &RoutineId, now: i64, every: i64) -> bool {
        lock(&self.pages).due(id, now, every)
    }

    fn forget_page(&self, id: &RoutineId) {
        lock(&self.pages).forget(id);
    }

    /// The bot was deleted, and its runs with it.
    pub(crate) fn forget(&self, bot: &BotId) {
        self.turns().retain(|seat, _| seat.bot != *bot);
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
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

/// The conversation took up `message`: remember the run it belongs to.
pub fn turn_began(daemon: &Daemon, bot: &Seat, message: &MessageId) {
    let run = daemon.store().run_of_message(message).ok().flatten();
    if let Some(run) = run
        && run.status == RunStatus::Queued
    {
        daemon
            .routines
            .turns()
            .entry(bot.clone())
            .or_default()
            .push(run.id);
    }
}

/// The conversation's turn ended (`result`): its runs are done, or failed
/// on an error.
pub fn turn_ended(daemon: &Daemon, bot: &Seat, failed: bool) {
    let status = if failed {
        RunStatus::Failed
    } else {
        RunStatus::Done
    };
    let runs = daemon.routines.turns().remove(bot).unwrap_or_default();
    for run in runs {
        finish(daemon, &run, status);
    }
}

/// The bot said the routine runs of its turn had nothing new to report
/// (`nothing_new`, spec 20.5). `false` when the turn carries no open run.
pub fn mark_nothing_new(daemon: &Daemon, bot: &Seat) -> bool {
    let runs = daemon
        .routines
        .turns()
        .get(bot)
        .cloned()
        .unwrap_or_default();
    let mut marked = false;
    for run in runs {
        match daemon.store().mark_nothing_new(&run) {
            Ok(changed) => marked |= changed,
            Err(err) => warn!(run = %run, "could not mark a run as having nothing new: {err}"),
        }
    }
    marked
}

/// The conversation's process ended mid-turn: those runs will not finish.
pub fn process_ended(daemon: &Daemon, bot: &Seat) {
    let runs = daemon.routines.turns().remove(bot).unwrap_or_default();
    for run in runs {
        finish(daemon, &run, RunStatus::Failed);
    }
}

fn finish(daemon: &Daemon, run: &RoutineRunId, status: RunStatus) {
    let now = daemon.clock.now_ms();
    let finished = daemon.store().finish_run(run, status, now);
    match finished {
        Ok(Some(run)) => {
            daemon.emit(Event::RoutineRun(run.clone()));
            if run.status == RunStatus::Failed {
                backoff::after_failure(daemon, &run.routine_id, now);
            }
            fire::announce_routine(daemon, &run.routine_id);
        }
        Ok(None) => {}
        Err(err) => warn!(run = %run, "could not end a routine run: {err}"),
    }
}
