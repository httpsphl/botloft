//! What happens when a routine's time comes (spec 20.3 and 20.4): times
//! missed while the daemon was away, a paused bot, a run still open, and
//! the message a run sends.

use botloft_core::ids::{MessageId, RoutineId, RoutineRunId};
use botloft_core::protocol::{
    Crew, Message, MessageKind, Missed, Overlap, Routine, RoutineRun, RunStatus, SenderKind,
    SkipReason,
};
use botloft_store::{BotRecord, Store};
use tracing::{debug, warn};

use super::schedule::Plan;
use crate::service::{ApiError, ApiResult, bots, messages};
use crate::state::{Daemon, Event};

/// How late a time may fire and still count as on time: the scheduler
/// wakes at least once a minute.
const ON_TIME_MS: i64 = 2 * 60 * 1000;
/// Times counted when catching up after a long absence.
const CATCH_UP_MAX: usize = 10_000;

/// The routine's time came: fire it, skip it, or catch up on missed times,
/// then move it to its next time.
pub(super) fn due(daemon: &Daemon, mut routine: Routine, now: i64) {
    let Some(first) = routine.next_run_at else {
        return;
    };
    let plan = match Plan::new(&routine.schedule, &routine.timezone) {
        Ok(plan) => plan,
        Err(err) => {
            // Checked when saved; a zone gone from a newer tz database.
            warn!(routine = %routine.id, "a routine's schedule no longer works: {err}");
            routine.next_run_at = None;
            save(daemon, &routine);
            return;
        }
    };
    // Every time that came, oldest first.
    let mut times = vec![first];
    while times.len() < CATCH_UP_MAX {
        let last = times[times.len() - 1];
        match plan.next_after(last, last) {
            Some(next) if next <= now => times.push(next),
            _ => break,
        }
    }
    let latest = times[times.len() - 1];
    let on_time = now - latest <= ON_TIME_MS;
    let (fire_at, missed) = match (on_time, routine.missed) {
        (true, _) | (false, Missed::RunOnce) => (Some(latest), times.len() - 1),
        (false, Missed::Skip) => (None, times.len()),
    };

    let store = daemon.store();
    let Ok((crew, bot)) = bots::active(&store, &routine.bot_id) else {
        // Its bot or crew was archived: the routine goes with it.
        routine.archived_at = Some(now);
        routine.next_run_at = None;
        drop(store);
        save(daemon, &routine);
        return;
    };
    if missed > 0 {
        let run = skipped(&routine, times[missed - 1], SkipReason::Missed, missed, now);
        record(daemon, &store, run);
    }
    if let Some(at) = fire_at
        && let Err(err) = attempt(daemon, &store, &routine, &crew, &bot, at, true)
    {
        warn!(routine = %routine.id, "a routine could not run: {err}");
    }
    routine.next_run_at = plan.next_after(latest, now);
    drop(store);
    save(daemon, &routine);
}

/// "Run now" (spec 20.3): a run at this moment, outside the schedule, by
/// the same overlap rule. The bot being paused only delays it.
pub fn fire_now(daemon: &Daemon, id: &RoutineId) -> ApiResult<RoutineRun> {
    let store = daemon.store();
    let routine = store
        .routine(id)?
        .ok_or_else(|| ApiError::NotFound(format!("routine {id} does not exist")))?;
    if routine.archived_at.is_some() {
        return Err(ApiError::Conflict(format!("routine {id} is archived")));
    }
    let (crew, bot) = bots::active(&store, &routine.bot_id)?;
    let now = daemon.clock.now_ms();
    let run = attempt(daemon, &store, &routine, &crew, &bot, now, false)?;
    drop(store);
    announce_routine(daemon, id);
    Ok(run)
}

/// The next time a routine runs, counting from `from`, after `now`.
pub fn next_run_at(routine: &Routine, from: i64, now: i64) -> Option<i64> {
    Plan::new(&routine.schedule, &routine.timezone)
        .ok()?
        .next_after(from, now)
}

/// Sends the routine's message, unless the bot is paused (for a scheduled
/// time) or a run is still open.
fn attempt(
    daemon: &Daemon,
    store: &Store,
    routine: &Routine,
    crew: &Crew,
    bot: &BotRecord,
    at: i64,
    scheduled: bool,
) -> ApiResult<RoutineRun> {
    let now = daemon.clock.now_ms();
    if scheduled && (bot.paused || crew.paused) {
        let run = skipped(routine, at, SkipReason::BotPaused, 1, now);
        record(daemon, store, run.clone());
        return Ok(run);
    }
    let open = store.open_runs(&routine.id)?;
    let room = match routine.overlap {
        Overlap::Skip => 1,
        Overlap::Queue => 2,
    };
    if open >= room {
        let run = skipped(routine, at, SkipReason::Overlap, 1, now);
        record(daemon, store, run.clone());
        return Ok(run);
    }
    let message = Message {
        id: MessageId::generate(),
        crew_id: crew.id.clone(),
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: bot.id.clone(),
        kind: MessageKind::Routine,
        body: routine.prompt.clone(),
        task_id: None,
        routine_id: Some(routine.id.clone()),
        question_id: None,
        attachments: Vec::new(),
        created_at: now,
    };
    let delivery = messages::pending_delivery(&message);
    let run = RoutineRun {
        id: RoutineRunId::generate(),
        routine_id: routine.id.clone(),
        scheduled_for: at,
        status: RunStatus::Queued,
        reason: None,
        skipped_count: 0,
        message_id: Some(message.id.clone()),
        created_at: now,
        finished_at: None,
    };
    let item = store.fire_run(&run, &message, &delivery)?;
    debug!(routine = %routine.id, run = %run.id, "routine fired");
    messages::announce(daemon, None, &message, delivery, item);
    daemon.emit(Event::RoutineRun(run.clone()));
    Ok(run)
}

fn skipped(routine: &Routine, at: i64, reason: SkipReason, count: usize, now: i64) -> RoutineRun {
    RoutineRun {
        id: RoutineRunId::generate(),
        routine_id: routine.id.clone(),
        scheduled_for: at,
        status: RunStatus::Skipped,
        reason: Some(reason),
        skipped_count: u32::try_from(count).unwrap_or(u32::MAX),
        message_id: None,
        created_at: now,
        finished_at: Some(now),
    }
}

fn record(daemon: &Daemon, store: &Store, run: RoutineRun) {
    match store.insert_run(&run) {
        Ok(()) => daemon.emit(Event::RoutineRun(run)),
        Err(err) => warn!(routine = %run.routine_id, "could not record a skipped run: {err}"),
    }
}

fn save(daemon: &Daemon, routine: &Routine) {
    if let Err(err) = daemon.store().update_routine(routine) {
        warn!(routine = %routine.id, "could not save a routine: {err}");
        return;
    }
    announce_routine(daemon, &routine.id);
}

/// Tells the app how the routine stands now, with its latest run.
pub(super) fn announce_routine(daemon: &Daemon, id: &RoutineId) {
    match daemon.store().routine(id) {
        Ok(Some(routine)) => daemon.emit(Event::RoutineChanged(routine)),
        Ok(None) => {}
        Err(err) => warn!(routine = %id, "could not read a routine: {err}"),
    }
}

/// Open runs whose message died (spec 20.3) fail.
pub(super) fn fail_undelivered(daemon: &Daemon, now: i64) {
    let dead = daemon.store().runs_with_dead_delivery();
    for run in dead.unwrap_or_else(|err| {
        warn!("could not read undelivered routine runs: {err}");
        Vec::new()
    }) {
        end(daemon, &run.id, now);
    }
}

/// At start: runs whose turn began under a previous daemon never finished,
/// since its bots died with it.
pub(super) fn fail_stale(daemon: &Daemon) {
    let now = daemon.clock.now_ms();
    let stale = daemon.store().stale_runs();
    for run in stale.unwrap_or_else(|err| {
        warn!("could not read stale routine runs: {err}");
        Vec::new()
    }) {
        end(daemon, &run.id, now);
    }
}

fn end(daemon: &Daemon, run: &RoutineRunId, now: i64) {
    let ended = daemon.store().finish_run(run, RunStatus::Failed, now);
    match ended {
        Ok(Some(run)) => {
            daemon.emit(Event::RoutineRun(run.clone()));
            announce_routine(daemon, &run.routine_id);
        }
        Ok(None) => {}
        Err(err) => warn!(run = %run, "could not end a routine run: {err}"),
    }
}
