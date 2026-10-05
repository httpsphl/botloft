//! `change_bot` (spec 10.3): a bot asks the owner to rename itself or change
//! its role or instructions, and the crew's chief may ask the same for any
//! bot of its crew. Only bots of the caller's crew can be named. The owner
//! approves or declines in the caller's chat and the call waits, like
//! `change_routine`; a bot that bypasses permissions does it at once.

use botloft_core::chat::CHANGE_BOT_TOOL;
use botloft_core::ids::BotId;
use botloft_core::protocol::{Bot, BotsUpdateParams, PermissionMode};
use botloft_core::{slug, validate};
use botloft_store::BotRecord;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use super::suggest::{INSTRUCTIONS_MAX, REASON_MAX};
use crate::approvals::{self, Answer};
use crate::service::{ApiError, ApiResult, bots, lead};
use crate::state::Daemon;

pub const CHANGE_BOT: &str = "change_bot";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    bot: Option<String>,
    name: Option<String>,
    role: Option<String>,
    instructions: Option<String>,
    reason: Option<String>,
}

/// What the owner sees changing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct Profile {
    name: String,
    role: String,
    instructions: String,
}

impl Profile {
    fn of(record: &BotRecord) -> Self {
        Self {
            name: record.name.clone(),
            role: record.role.clone(),
            instructions: record.instructions.clone(),
        }
    }
}

pub(super) fn tool() -> Value {
    json!({
        "name": CHANGE_BOT,
        "title": "Change a bot",
        "description": "Asks the owner to rename you or change your role or instructions, \
            when the owner wants that. The crew's chief may name another bot of its crew with \
            `bot`. Give only what changes. The owner approves or declines it in your chat; the \
            call waits for that. Say it changed only after the tool says it did.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "bot": {
                    "type": "string",
                    "description": "Only for the chief: the handle of the bot to change, as \
                        crew_roster gives it. Leave it out to change yourself.",
                },
                "name": {
                    "type": "string",
                    "maxLength": validate::NAME_MAX_CHARS,
                    "description": "The new name. The handle changes with it.",
                },
                "role": {
                    "type": "string",
                    "maxLength": validate::ROLE_MAX_CHARS,
                    "description": "One line: what the bot does in the crew.",
                },
                "instructions": {
                    "type": "string",
                    "maxLength": INSTRUCTIONS_MAX,
                    "description": "The whole new instructions, written to the bot. They \
                        replace the old ones.",
                },
                "reason": {
                    "type": "string",
                    "maxLength": REASON_MAX,
                    "description": "For the owner: why, in a sentence.",
                },
            },
            "additionalProperties": false,
        },
    })
}

pub(super) async fn change(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Value {
    tool_result(run(daemon, bot, generation, args).await)
}

/// The bot to change: the caller, or for the chief one of its crew.
fn target(daemon: &Daemon, caller: &BotId, handle: Option<&str>) -> ApiResult<BotRecord> {
    let store = daemon.store();
    let (crew, me) = bots::active(&store, caller)?;
    let Some(handle) = handle.map(|to| slug::slugify(to.trim().trim_start_matches('@'), "")) else {
        return Ok(me);
    };
    if handle == me.handle {
        return Ok(me);
    }
    if !lead::is_lead(&crew, caller) {
        return Err(ApiError::Conflict(
            "only the crew's chief can change another bot; ask the chief with send_message, \
             or leave out `bot` to change yourself"
                .into(),
        ));
    }
    let found = match handle.as_str() {
        "" => None,
        handle => store.active_bot_by_handle(&crew.id, handle)?,
    };
    let id = found.ok_or_else(|| {
        ApiError::NotFound(format!(
            "no bot in your crew answers to @{handle}; call crew_roster to see who does"
        ))
    })?;
    bots::find(&store, &id)
}

/// The profile after the change, checked as `bots.update` would.
fn after(daemon: &Daemon, record: &BotRecord, args: &Args) -> ApiResult<Profile> {
    let mut next = Profile::of(record);
    if let Some(name) = &args.name {
        next.name = validate::name("name", name)?;
        let store = daemon.store();
        let handle = slug::slugify(&next.name, "bot");
        bots::ensure_handle_free(&store, &record.crew_id, &handle, Some(&record.id))?;
    }
    if let Some(role) = &args.role {
        next.role = validate::role(role)?;
    }
    if let Some(instructions) = &args.instructions {
        if instructions.chars().count() > INSTRUCTIONS_MAX {
            return Err(ApiError::validation(format!(
                "instructions must be at most {INSTRUCTIONS_MAX} characters"
            )));
        }
        next.instructions = validate::instructions(instructions)?;
    }
    Ok(next)
}

fn apply(daemon: &Daemon, id: &BotId, next: &Profile) -> ApiResult<Bot> {
    bots::update(
        daemon,
        BotsUpdateParams {
            bot_id: id.clone(),
            name: Some(next.name.clone()),
            role: Some(next.role.clone()),
            instructions: Some(next.instructions.clone()),
            color: None,
        },
    )
}

async fn run(
    daemon: &Daemon,
    caller: &BotId,
    generation: u64,
    args: Value,
) -> Result<Value, String> {
    let args: Args = parse(args)?;
    if args
        .reason
        .as_ref()
        .is_some_and(|reason| reason.chars().count() > REASON_MAX)
    {
        return Err(format!("reason must be at most {REASON_MAX} characters"));
    }
    let record = target(daemon, caller, args.bot.as_deref()).map_err(explain)?;
    let before = Profile::of(&record);
    // Nothing impossible goes to the owner.
    let next = after(daemon, &record, &args).map_err(explain)?;
    if next == before {
        return Err("that is how the bot is already: nothing to change".to_owned());
    }
    let mode = bots::find(&daemon.store(), caller)
        .map_err(explain)?
        .permission_mode;
    if mode == PermissionMode::BypassPermissions {
        let bot = apply(daemon, &record.id, &next).map_err(explain)?;
        return Ok(done(
            &bot,
            caller,
            "You bypass permissions, so it changed without asking.",
        ));
    }
    let input = json!({
        "bot_id": record.id,
        "handle": record.handle,
        "name": record.name,
        "before": before,
        "after": next,
        "reason": args.reason,
    });
    match approvals::ask(daemon, caller, generation, CHANGE_BOT_TOOL, &input, "").await {
        Some(Answer::Allowed { .. }) => {
            // Checked again: the crew may have changed while the owner read it.
            let next = after(daemon, &record, &args).map_err(explain)?;
            let bot = apply(daemon, &record.id, &next).map_err(explain)?;
            Ok(done(&bot, caller, "The owner approved the change."))
        }
        Some(Answer::Denied { note }) => {
            let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Ok(json!({
                "done": false,
                "note": format!("The owner declined the change.{why} Nothing changed: do not say it did."),
            }))
        }
        Some(Answer::Expired) => Ok(json!({
            "done": false,
            "note": "The owner did not answer in time. Nothing changed: tell the owner, and ask \
                again later if it still helps.",
        })),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

fn done(bot: &Bot, caller: &BotId, how: &str) -> Value {
    let when = if bot.id == *caller {
        "Your new instructions and role take effect when you next start; your name and \
         handle already changed."
    } else {
        "The bot reads its new instructions and role when it next starts."
    };
    json!({
        "done": true,
        "handle": bot.handle,
        "name": bot.name,
        "role": bot.role,
        "note": format!("{how} The bot is now @{} ({}). {when}", bot.handle, bot.name),
    })
}
