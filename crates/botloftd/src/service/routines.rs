//! `routines.*` operations (spec 20.8).

use botloft_core::ids::{BotId, RoutineId};
use botloft_core::protocol::{
    Missed, Overlap, Routine, RoutineIdParams, RoutineRun, RoutinesCreateParams,
    RoutinesListParams, RoutinesRunsParams, RoutinesSetEnabledParams, RoutinesUpdateParams,
    Schedule,
};
use botloft_core::validate;
use botloft_store::Store;
use tracing::warn;

use super::{ApiError, ApiResult, bots};
use crate::routines;
use crate::routines::schedule::{Plan, Problem, normalized};
use crate::state::{Daemon, Event};

const NAME_MAX_CHARS: usize = 80;
const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;

pub fn list(daemon: &Daemon, params: RoutinesListParams) -> ApiResult<Vec<Routine>> {
    let store = daemon.store();
    if let Some(bot) = &params.bot_id {
        bots::find(&store, bot)?;
    }
    Ok(store.routines(params.bot_id.as_ref(), false)?)
}

pub fn create(daemon: &Daemon, params: RoutinesCreateParams) -> ApiResult<Routine> {
    let name = name(&params.name)?;
    let prompt = validate::message("prompt", &params.prompt)?;
    let now = daemon.clock.now_ms();
    let schedule = normalized(params.schedule);
    let plan = plan(&schedule, &params.timezone, now)?;
    let store = daemon.store();
    bots::active(&store, &params.bot_id)?;
    let routine = Routine {
        id: RoutineId::generate(),
        bot_id: params.bot_id,
        name,
        prompt,
        schedule,
        timezone: params.timezone,
        overlap: params.overlap.unwrap_or(Overlap::Skip),
        missed: params.missed.unwrap_or(Missed::RunOnce),
        enabled: true,
        next_run_at: plan.next_after(now, now),
        last_run: None,
        created_at: now,
        updated_at: now,
        archived_at: None,
    };
    store.insert_routine(&routine)?;
    Ok(changed(daemon, &store, routine))
}

/// Checks what `create` would, without making the routine: a bot's
/// suggestion is checked before the owner sees it (spec 20.12).
pub(crate) fn check(daemon: &Daemon, params: &RoutinesCreateParams) -> ApiResult<()> {
    name(&params.name)?;
    validate::message("prompt", &params.prompt)?;
    let schedule = normalized(params.schedule.clone());
    plan(&schedule, &params.timezone, daemon.clock.now_ms())?;
    Ok(())
}

/// When the routine runs next, as its zone shows it (`2026-10-01 09:00`).
pub(crate) fn next_run_text(routine: &Routine) -> Option<String> {
    let plan = Plan::new(&routine.schedule, &routine.timezone).ok()?;
    Some(plan.local_time(routine.next_run_at?))
}

pub fn update(daemon: &Daemon, params: RoutinesUpdateParams) -> ApiResult<Routine> {
    let store = daemon.store();
    let mut routine = active(&store, &params.routine_id)?;
    if let Some(name_) = params.name {
        routine.name = name(&name_)?;
    }
    if let Some(prompt) = params.prompt {
        routine.prompt = validate::message("prompt", &prompt)?;
    }
    let now = daemon.clock.now_ms();
    let timing = params.schedule.is_some() || params.timezone.is_some();
    if let Some(schedule) = params.schedule {
        routine.schedule = normalized(schedule);
    }
    if let Some(timezone) = params.timezone {
        routine.timezone = timezone;
    }
    if timing {
        // A new schedule starts counting now.
        let plan = plan(&routine.schedule, &routine.timezone, now)?;
        if routine.enabled {
            routine.next_run_at = plan.next_after(now, now);
        }
    }
    if let Some(overlap) = params.overlap {
        routine.overlap = overlap;
    }
    if let Some(missed) = params.missed {
        routine.missed = missed;
    }
    routine.updated_at = now;
    store.update_routine(&routine)?;
    Ok(changed(daemon, &store, routine))
}

/// Turning a routine back on starts from now: times that passed while it
/// was off are not missed.
pub fn set_enabled(daemon: &Daemon, params: RoutinesSetEnabledParams) -> ApiResult<Routine> {
    let store = daemon.store();
    let mut routine = active(&store, &params.routine_id)?;
    if routine.enabled == params.enabled {
        return Ok(routine);
    }
    let now = daemon.clock.now_ms();
    routine.enabled = params.enabled;
    routine.next_run_at = if params.enabled {
        routines::next_run_at(&routine, now, now)
    } else {
        None
    };
    routine.updated_at = now;
    store.update_routine(&routine)?;
    Ok(changed(daemon, &store, routine))
}

pub fn run_now(daemon: &Daemon, params: RoutineIdParams) -> ApiResult<RoutineRun> {
    routines::fire_now(daemon, &params.routine_id)
}

/// Archives the routine. Archiving twice is not an error.
pub fn archive(daemon: &Daemon, params: RoutineIdParams) -> ApiResult<Routine> {
    let store = daemon.store();
    let mut routine = find(&store, &params.routine_id)?;
    if routine.archived_at.is_some() {
        return Ok(routine);
    }
    let now = daemon.clock.now_ms();
    routine.archived_at = Some(now);
    routine.next_run_at = None;
    routine.updated_at = now;
    store.update_routine(&routine)?;
    Ok(changed(daemon, &store, routine))
}

pub fn runs(daemon: &Daemon, params: RoutinesRunsParams) -> ApiResult<Vec<RoutineRun>> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(ApiError::validation(format!(
            "limit must be between 1 and {MAX_LIMIT}"
        )));
    }
    let store = daemon.store();
    find(&store, &params.routine_id)?;
    Ok(store.runs(&params.routine_id, params.before.as_ref(), limit)?)
}

/// Archives the routines of a bot that was archived (spec 20.3).
pub(crate) fn archive_of(daemon: &Daemon, store: &Store, bot: &BotId) {
    let now = daemon.clock.now_ms();
    let archived = match store.archive_routines_of(bot, now) {
        Ok(archived) => archived,
        Err(err) => {
            warn!(bot = %bot, "could not archive the bot's routines: {err}");
            return;
        }
    };
    for id in archived {
        if let Ok(Some(routine)) = store.routine(&id) {
            daemon.emit(Event::RoutineChanged(routine));
        }
    }
}

fn name(input: &str) -> ApiResult<String> {
    let name = input.trim();
    if name.is_empty() {
        return Err(ApiError::validation("name must not be empty"));
    }
    if name.contains(['\n', '\r']) {
        return Err(ApiError::validation("name must be a single line"));
    }
    if name.chars().count() > NAME_MAX_CHARS {
        return Err(ApiError::validation(format!(
            "name must be at most {NAME_MAX_CHARS} characters"
        )));
    }
    Ok(name.to_owned())
}

/// A checked schedule, its runs at least five minutes apart.
fn plan(schedule: &Schedule, timezone: &str, now: i64) -> ApiResult<Plan> {
    let rule = |problem: Problem| ApiError::Rule {
        reason: problem.reason,
        message: problem.message,
    };
    let plan = Plan::new(schedule, timezone).map_err(rule)?;
    plan.check_spacing(now).map_err(rule)?;
    Ok(plan)
}

fn changed(daemon: &Daemon, store: &Store, routine: Routine) -> Routine {
    // With its latest run, as the list shows it.
    let routine = store.routine(&routine.id).ok().flatten().unwrap_or(routine);
    daemon.emit(Event::RoutineChanged(routine.clone()));
    daemon.routines.wake();
    routine
}

fn find(store: &Store, id: &RoutineId) -> ApiResult<Routine> {
    store
        .routine(id)?
        .ok_or_else(|| ApiError::NotFound(format!("routine {id} does not exist")))
}

fn active(store: &Store, id: &RoutineId) -> ApiResult<Routine> {
    let routine = find(store, id)?;
    if routine.archived_at.is_some() {
        return Err(ApiError::Conflict(format!("routine {id} is archived")));
    }
    Ok(routine)
}
