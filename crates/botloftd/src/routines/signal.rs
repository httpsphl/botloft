//! Signals between bots (spec 20.13): a bot says something happened, and
//! the routines of its crew that wait for that signal run.

use botloft_core::ids::BotId;
use botloft_core::protocol::{Routine, RoutineRun, RunSignal, SkipReason};
use tracing::{debug, warn};

use super::fire::{self, Trigger};
use super::schedule::signal_name;
use crate::service::{ApiError, ApiResult, bots};
use crate::state::Daemon;

/// Closest two runs of one routine may be when signals cause them, as for
/// schedules: a bot that signals in a loop cannot spend the owner's plan.
const SIGNAL_SPACING_MS: i64 = 5 * 60 * 1000;
/// Longest note a signal carries, in characters.
pub const NOTE_MAX_CHARS: usize = 2_000;

/// A routine the signal reached, and what became of it.
pub struct Reached {
    pub routine: Routine,
    pub run: RoutineRun,
}

/// `from` sends the signal `name` to its crew. Every enabled routine of an
/// active bot there that waits for it runs, unless it ran for a signal less
/// than five minutes ago, its bot is paused or its last run is still open.
pub fn send(
    daemon: &Daemon,
    from: &BotId,
    name: &str,
    note: Option<&str>,
) -> ApiResult<(String, Vec<Reached>)> {
    let name = signal_name(name);
    if name.is_empty() {
        return Err(ApiError::validation(
            "a signal name needs letters or digits, like report-ready",
        ));
    }
    let note = note.map(str::trim).filter(|note| !note.is_empty());
    if note.is_some_and(|note| note.chars().count() > NOTE_MAX_CHARS) {
        return Err(ApiError::validation(format!(
            "note must be at most {NOTE_MAX_CHARS} characters"
        )));
    }
    let store = daemon.store();
    let (crew, _) = bots::active(&store, from)?;
    let signal = RunSignal {
        name: name.clone(),
        from_bot_id: Some(from.clone()),
        note: note.map(str::to_owned),
    };
    let now = daemon.clock.now_ms();
    let mut reached = Vec::new();
    for routine in store.signal_routines(&crew.id, Some(&name))? {
        let Ok((crew, bot)) = bots::active(&store, &routine.bot_id) else {
            continue;
        };
        let last = store.last_signal_run_at(&routine.id)?;
        let run = if last.is_some_and(|at| now - at < SIGNAL_SPACING_MS) {
            let mut run = fire::skipped(&routine, now, SkipReason::TooSoon, 1, now);
            run.signal = Some(signal.clone());
            fire::record(daemon, &store, run.clone());
            run
        } else {
            match fire::attempt(
                daemon,
                &store,
                &routine,
                &crew,
                &bot,
                now,
                Trigger::Signal(signal.clone()),
            ) {
                Ok(run) => run,
                Err(err) => {
                    warn!(routine = %routine.id, "a signal could not run a routine: {err}");
                    continue;
                }
            }
        };
        debug!(routine = %routine.id, status = %run.status, "signal reached a routine");
        reached.push(Reached { routine, run });
    }
    drop(store);
    for each in &reached {
        fire::announce_routine(daemon, &each.routine.id);
    }
    Ok((name, reached))
}
