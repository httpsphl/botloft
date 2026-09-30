//! `schedule_routine` (spec 20.12): a bot asks for a routine, for itself or,
//! by handle, for another bot of its crew. The owner sees it in the bot's
//! chat, may change it, and allows or declines it; the call waits for that,
//! like `suggest_bot`. A bot that bypasses permissions creates it at once.

use botloft_core::chat::ROUTINE_TOOL;
use botloft_core::ids::BotId;
use botloft_core::protocol::{
    Missed, Overlap, PermissionMode, Routine, RoutinesCreateParams, Schedule,
};
use botloft_core::slug;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use crate::approvals::{self, Answer};
use crate::service::{ApiError, ApiResult, bots, routines};
use crate::state::Daemon;

/// Longest prompt a bot may write for a routine, in characters: the owner
/// reads all of it in the card.
pub(super) const PROMPT_MAX: usize = 8_000;
/// Used when the system's zone has no IANA name.
const FALLBACK_ZONE: &str = "UTC";

/// The tool's input, and what the owner sends back when they change it.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Proposal {
    name: String,
    prompt: String,
    schedule: Schedule,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
    /// Handle of the bot the routine is for, when not the caller.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    bot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    overlap: Option<Overlap>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    missed: Option<Missed>,
}

impl Proposal {
    fn params(&self, bot: BotId) -> RoutinesCreateParams {
        RoutinesCreateParams {
            bot_id: bot,
            name: self.name.clone(),
            prompt: self.prompt.clone(),
            schedule: self.schedule.clone(),
            timezone: self.timezone.clone().unwrap_or_else(system_zone),
            overlap: self.overlap,
            missed: self.missed,
        }
    }
}

pub(super) async fn schedule(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    arguments: Value,
) -> Value {
    tool_result(run(daemon, bot, generation, arguments).await)
}

async fn run(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    arguments: Value,
) -> Result<Value, String> {
    let mut proposal: Proposal = parse(arguments)?;
    if proposal.prompt.chars().count() > PROMPT_MAX {
        return Err(format!("prompt must be at most {PROMPT_MAX} characters"));
    }
    let (target, handle) = target(daemon, bot, proposal.bot.as_deref()).map_err(explain)?;
    proposal.bot = handle;
    proposal.timezone = Some(proposal.timezone.unwrap_or_else(system_zone));
    // Nothing impossible goes to the owner.
    let params = proposal.params(target.clone());
    routines::check(daemon, &params).map_err(explain)?;

    let mode = bots::find(&daemon.store(), bot)
        .map_err(explain)?
        .permission_mode;
    if mode == PermissionMode::BypassPermissions {
        let routine = routines::create(daemon, params).map_err(explain)?;
        return Ok(created_note(
            &routine,
            "You bypass permissions, so it was created without asking the owner.",
        ));
    }

    let input = serde_json::to_value(&proposal).map_err(|err| err.to_string())?;
    match approvals::ask(daemon, bot, generation, ROUTINE_TOOL, &input, "").await {
        Some(Answer::Allowed { input: None }) => {
            let routine = routines::create(daemon, params).map_err(explain)?;
            Ok(created_note(&routine, "The owner approved it."))
        }
        Some(Answer::Allowed {
            input: Some(changed),
        }) => {
            let changed: Proposal = serde_json::from_str(&changed)
                .map_err(|err| format!("the owner's changes could not be read: {err}"))?;
            let routine = routines::create(daemon, changed.params(target)).map_err(explain)?;
            Ok(created_note(
                &routine,
                "The owner changed it before approving it: read the name, prompt and schedule \
                 above.",
            ))
        }
        Some(Answer::Denied { note }) => {
            let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Ok(json!({
                "created": false,
                "note": format!(
                    "The owner declined this routine.{why} Nothing was scheduled: do not say it is."
                ),
            }))
        }
        Some(Answer::Expired) => Ok(json!({
            "created": false,
            "note": "The owner did not answer in time. Nothing was scheduled: tell the owner, and \
                ask again later if it still helps.",
        })),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

/// The owner's changes to a request (`approvals.answer`), checked before
/// the request is settled so the card can say what is wrong. Returns them
/// with the bot and zone of the request, which the owner does not change.
pub(crate) fn check_changed(daemon: &Daemon, asked: &str, changed: &str) -> ApiResult<String> {
    let asked: Proposal = serde_json::from_str(asked)
        .map_err(|_| ApiError::validation("the routine asked for could not be read"))?;
    let mut changed: Proposal = serde_json::from_str(changed)
        .map_err(|_| ApiError::validation("the changed routine is not valid"))?;
    changed.bot = asked.bot;
    if changed.timezone.is_none() {
        changed.timezone = asked.timezone;
    }
    // The bot is checked again when the routine is created.
    let params = changed.params(BotId::generate());
    routines::check(daemon, &params)?;
    serde_json::to_string(&changed).map_err(|err| ApiError::validation(err.to_string()))
}

/// The bot the routine is for, and its handle when it is not the caller.
fn target(
    daemon: &Daemon,
    caller: &BotId,
    handle: Option<&str>,
) -> ApiResult<(BotId, Option<String>)> {
    let store = daemon.store();
    let (crew, me) = bots::active(&store, caller)?;
    let handle = match handle.map(|h| slug::slugify(h.trim().trim_start_matches('@'), "")) {
        None => return Ok((me.id, None)),
        Some(handle) if handle.is_empty() || handle == me.handle => return Ok((me.id, None)),
        Some(handle) => handle,
    };
    let Some(id) = store.active_bot_by_handle(&crew.id, &handle)? else {
        return Err(ApiError::NotFound(format!(
            "no bot in your crew answers to @{handle}; call crew_roster to see who does"
        )));
    };
    Ok((id, Some(handle)))
}

/// The computer's time zone, as the app sends it when the owner creates a
/// routine (spec 20.2).
fn system_zone() -> String {
    jiff::tz::TimeZone::system()
        .iana_name()
        .map_or_else(|| FALLBACK_ZONE.to_owned(), str::to_owned)
}

fn created_note(routine: &Routine, how: &str) -> Value {
    let next = routines::next_run_text(routine).map_or_else(String::new, |at| {
        format!(" It runs next at {at} ({}).", routine.timezone)
    });
    json!({
        "created": true,
        "routine_id": routine.id,
        "name": routine.name,
        "prompt": routine.prompt,
        "schedule": routine.schedule,
        "timezone": routine.timezone,
        "note": format!(
            "{how} The owner finds it in the Botloft app among the routines of the bot it is \
             for, and can change, pause or delete it there.{next} Each time, that bot gets the prompt as a message."
        ),
    })
}
