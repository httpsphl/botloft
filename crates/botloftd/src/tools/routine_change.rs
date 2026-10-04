//! `my_routines`, `change_routine` and `delete_routine` (spec 20.12): a bot
//! sees its own routines and asks the owner to change or delete one. The
//! owner answers in the bot's chat, and may adjust a change before allowing
//! it; the call waits, like `schedule_routine`. A bot that bypasses
//! permissions does it at once.

use botloft_core::chat::{CHANGE_ROUTINE_TOOL, DELETE_ROUTINE_TOOL};
use botloft_core::ids::{BotId, RoutineId};
use botloft_core::protocol::{
    Missed, Overlap, PermissionMode, Routine, RoutineIdParams, RoutinesCreateParams,
    RoutinesListParams, RoutinesSetEnabledParams, RoutinesUpdateParams, Schedule,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use super::routine::PROMPT_MAX;
use crate::approvals::{self, Answer};
use crate::service::{ApiError, ApiResult, bots, routines};
use crate::state::Daemon;

pub(super) use super::routine_catalog::{CHANGE_ROUTINE, DELETE_ROUTINE, MY_ROUTINES, REASON_MAX};

/// What `change_routine` takes: the routine and only what changes.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeArgs {
    routine_id: RoutineId,
    name: Option<String>,
    prompt: Option<String>,
    schedule: Option<Schedule>,
    timezone: Option<String>,
    overlap: Option<Overlap>,
    missed: Option<Missed>,
    enabled: Option<bool>,
}

/// The routine as it would be after the change: the request's input, and
/// what the owner sends back when they adjust it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Changed {
    routine_id: RoutineId,
    name: String,
    prompt: String,
    schedule: Schedule,
    timezone: String,
    overlap: Overlap,
    missed: Missed,
    enabled: bool,
}

impl Changed {
    fn of(routine: &Routine) -> Self {
        Self {
            routine_id: routine.id.clone(),
            name: routine.name.clone(),
            prompt: routine.prompt.clone(),
            schedule: routine.schedule.clone(),
            timezone: routine.timezone.clone(),
            overlap: routine.overlap,
            missed: routine.missed,
            enabled: routine.enabled,
        }
    }

    fn checked(&self, daemon: &Daemon, bot: &BotId) -> ApiResult<()> {
        if self.prompt.chars().count() > PROMPT_MAX {
            return Err(ApiError::validation(format!(
                "prompt must be at most {PROMPT_MAX} characters"
            )));
        }
        routines::check(
            daemon,
            &RoutinesCreateParams {
                bot_id: bot.clone(),
                name: self.name.clone(),
                prompt: self.prompt.clone(),
                schedule: self.schedule.clone(),
                timezone: self.timezone.clone(),
                overlap: Some(self.overlap),
                missed: Some(self.missed),
            },
        )
    }

    fn apply(self, daemon: &Daemon) -> ApiResult<Routine> {
        routines::update(
            daemon,
            RoutinesUpdateParams {
                routine_id: self.routine_id.clone(),
                name: Some(self.name),
                prompt: Some(self.prompt),
                schedule: Some(self.schedule),
                timezone: Some(self.timezone),
                overlap: Some(self.overlap),
                missed: Some(self.missed),
            },
        )?;
        routines::set_enabled(
            daemon,
            RoutinesSetEnabledParams {
                routine_id: self.routine_id,
                enabled: self.enabled,
            },
        )
    }
}

/// The caller's routines, oldest first.
fn own(daemon: &Daemon, bot: &BotId) -> ApiResult<Vec<Routine>> {
    routines::list(
        daemon,
        RoutinesListParams {
            bot_id: Some(bot.clone()),
        },
    )
}

/// One of the caller's routines; another bot's is as good as missing.
fn own_one(daemon: &Daemon, bot: &BotId, id: &RoutineId) -> ApiResult<Routine> {
    own(daemon, bot)?
        .into_iter()
        .find(|routine| routine.id == *id)
        .ok_or_else(|| {
            ApiError::NotFound(format!(
                "you have no routine {id}; call my_routines to see yours"
            ))
        })
}

fn bypasses(daemon: &Daemon, bot: &BotId) -> ApiResult<bool> {
    Ok(bots::find(&daemon.store(), bot)?.permission_mode == PermissionMode::BypassPermissions)
}

fn shown(routine: &Routine) -> Value {
    json!({
        "routine_id": routine.id,
        "name": routine.name,
        "prompt": routine.prompt,
        "schedule": routine.schedule,
        "timezone": routine.timezone,
        "enabled": routine.enabled,
        "next_run": routines::next_run_text(routine),
    })
}

pub(super) fn mine(daemon: &Daemon, bot: &BotId) -> Result<Value, String> {
    own(daemon, bot)
        .map(|list| json!({ "routines": list.iter().map(shown).collect::<Vec<_>>() }))
        .map_err(explain)
}

fn declined(note: Option<String>, what: &str) -> Value {
    let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
    json!({
        "done": false,
        "note": format!("The owner declined to {what}.{why} Nothing changed: do not say it did."),
    })
}

fn expired() -> Value {
    json!({
        "done": false,
        "note": "The owner did not answer in time. Nothing changed: tell the owner, and ask \
            again later if it still helps.",
    })
}

pub(super) async fn change(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Value {
    tool_result(run_change(daemon, bot, generation, args).await)
}

async fn run_change(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    args: Value,
) -> Result<Value, String> {
    let args: ChangeArgs = parse(args)?;
    let routine = own_one(daemon, bot, &args.routine_id).map_err(explain)?;
    let before = Changed::of(&routine);
    let mut after = before.clone();
    after.name = args.name.unwrap_or(after.name);
    after.prompt = args.prompt.unwrap_or(after.prompt);
    after.schedule = args.schedule.unwrap_or(after.schedule);
    after.timezone = args.timezone.unwrap_or(after.timezone);
    after.overlap = args.overlap.unwrap_or(after.overlap);
    after.missed = args.missed.unwrap_or(after.missed);
    after.enabled = args.enabled.unwrap_or(after.enabled);
    if after == before {
        return Err("that is how the routine is already: nothing to change".to_owned());
    }
    // Nothing impossible goes to the owner.
    after.checked(daemon, bot).map_err(explain)?;

    if bypasses(daemon, bot).map_err(explain)? {
        let changed = after.apply(daemon).map_err(explain)?;
        return Ok(done(
            &changed,
            "You bypass permissions, so it changed without asking.",
        ));
    }
    let input = json!({ "before": before, "after": after, "name": routine.name });
    match approvals::ask(daemon, bot, generation, CHANGE_ROUTINE_TOOL, &input, "").await {
        Some(Answer::Allowed { input: None }) => {
            let changed = after.apply(daemon).map_err(explain)?;
            Ok(done(&changed, "The owner approved the change."))
        }
        Some(Answer::Allowed {
            input: Some(adjusted),
        }) => {
            let adjusted: Changed = serde_json::from_str(&adjusted)
                .map_err(|err| format!("the owner's changes could not be read: {err}"))?;
            let changed = adjusted.apply(daemon).map_err(explain)?;
            Ok(done(
                &changed,
                "The owner adjusted the change before approving it: read the routine above.",
            ))
        }
        Some(Answer::Denied { note }) => Ok(declined(note, "change this routine")),
        Some(Answer::Expired) => Ok(expired()),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

fn done(routine: &Routine, how: &str) -> Value {
    let mut shown = shown(routine);
    shown["done"] = json!(true);
    shown["note"] = json!(format!("{how} The routine is now as above."));
    shown
}

/// What `delete_routine` takes.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteArgs {
    routine_id: RoutineId,
    reason: Option<String>,
}

pub(super) async fn delete(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Value {
    tool_result(run_delete(daemon, bot, generation, args).await)
}

async fn run_delete(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    args: Value,
) -> Result<Value, String> {
    let args: DeleteArgs = parse(args)?;
    if args
        .reason
        .as_ref()
        .is_some_and(|reason| reason.chars().count() > REASON_MAX)
    {
        return Err(format!("reason must be at most {REASON_MAX} characters"));
    }
    let routine = own_one(daemon, bot, &args.routine_id).map_err(explain)?;
    let gone = |how: &str| {
        routines::archive(
            daemon,
            RoutineIdParams {
                routine_id: routine.id.clone(),
            },
        )
        .map(|_| {
            json!({
                "done": true,
                "note": format!("{how} The routine \"{}\" is deleted and will not run again.", routine.name),
            })
        })
        .map_err(explain)
    };
    if bypasses(daemon, bot).map_err(explain)? {
        return gone("You bypass permissions, so it was deleted without asking.");
    }
    let input = json!({
        "routine_id": routine.id,
        "name": routine.name,
        "schedule": routine.schedule,
        "reason": args.reason,
    });
    match approvals::ask(daemon, bot, generation, DELETE_ROUTINE_TOOL, &input, "").await {
        Some(Answer::Allowed { .. }) => gone("The owner approved it."),
        Some(Answer::Denied { note }) => Ok(declined(note, "delete this routine")),
        Some(Answer::Expired) => Ok(expired()),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

/// The owner's adjustment of a change (`approvals.answer`), checked before
/// the request is settled so the card can say what is wrong. The routine
/// stays the one asked about.
pub(crate) fn check_changed(daemon: &Daemon, asked: &str, adjusted: &str) -> ApiResult<String> {
    let asked: Value = serde_json::from_str(asked)
        .map_err(|_| ApiError::validation("the change asked for could not be read"))?;
    let asked: Changed = serde_json::from_value(asked["after"].clone())
        .map_err(|_| ApiError::validation("the change asked for could not be read"))?;
    let mut adjusted: Changed = serde_json::from_str(adjusted)
        .map_err(|_| ApiError::validation("the adjusted routine is not valid"))?;
    adjusted.routine_id = asked.routine_id;
    // The bot is checked again when the change is made.
    adjusted.checked(daemon, &BotId::generate())?;
    serde_json::to_string(&adjusted).map_err(|err| ApiError::validation(err.to_string()))
}
