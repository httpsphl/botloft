//! `ask_crew_access` (spec 10.4): a bot asks the owner to reach another
//! crew, or one bot of it, saying what for. The owner allows it only now
//! (until the bot's turn ends), always for that bot, or always for the
//! whole crew, or declines; the call waits, like a permission request. A
//! bot that bypasses permissions gets it for the turn at once. With
//! access, `crew_roster` and `send_message` take `crew`.

use botloft_core::chat::CREW_ACCESS_TOOL;
use botloft_core::ids::BotId;
use botloft_core::protocol::{BotsListParams, PermissionMode};
use botloft_core::slug;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::{explain, parse, tool_result};
use super::suggest::REASON_MAX;
use crate::approvals::{self, Answer};
use crate::service::crew_access::{self, Reach};
use crate::service::{ApiError, ApiResult, bots, lead};
use crate::state::Daemon;

pub const ASK_CREW_ACCESS: &str = "ask_crew_access";

/// What a bot may ask for in another crew. Only talking, for now.
const KINDS: [&str; 1] = ["talk"];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    crew: String,
    bot: Option<String>,
    access: Vec<String>,
    why: String,
}

/// How long the owner allowed it, as `approvals.answer` sends it back.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Scope {
    /// Until the bot's turn ends.
    Once,
    /// For good, for the bot asked about.
    Bot,
    /// For good, for every bot of the crew.
    Crew,
}

#[derive(Debug, Deserialize)]
struct Answered {
    scope: Scope,
}

pub(super) fn tool() -> Value {
    json!({
        "name": ASK_CREW_ACCESS,
        "title": "Ask to reach another crew",
        "description": "Your crew cannot see or reach other crews. When the owner wants you to \
            work with another crew, or one of its bots, call this with the crew's name as the \
            owner gave it, the bot's handle if it is one bot, what you need (`talk`: see its \
            bots and send them messages and tasks) and why. The owner allows it only for now, \
            always for that bot, or always for the whole crew, or declines; the call waits for \
            that. Then use crew_roster and send_message with `crew`.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "crew": { "type": "string", "description": "The other crew's name." },
                "bot": {
                    "type": "string",
                    "description": "Handle of one bot of that crew, when you need only it.",
                },
                "access": {
                    "type": "array",
                    "items": { "type": "string", "enum": KINDS },
                    "minItems": 1,
                    "description": "What you need: `talk`.",
                },
                "why": {
                    "type": "string",
                    "maxLength": REASON_MAX,
                    "description": "For the owner: what you will do there and why, in a sentence \
                        or two, in the owner's language.",
                },
            },
            "required": ["crew", "access", "why"],
            "additionalProperties": false,
        },
    })
}

pub(super) async fn ask(daemon: &Daemon, bot: &BotId, generation: u64, args: Value) -> Value {
    tool_result(run(daemon, bot, generation, args).await)
}

async fn run(daemon: &Daemon, me: &BotId, generation: u64, args: Value) -> Result<Value, String> {
    let args: Args = parse(args)?;
    if args.access.is_empty()
        || args
            .access
            .iter()
            .any(|kind| !KINDS.contains(&kind.as_str()))
    {
        return Err("access must list what you need: \"talk\"".to_owned());
    }
    let why = args.why.trim();
    if why.is_empty() || why.chars().count() > REASON_MAX {
        return Err(format!(
            "why must say, in at most {REASON_MAX} characters, what you will do and why"
        ));
    }
    let (crew, target, mode) = {
        let store = daemon.store();
        let (own, record) = bots::active(&store, me).map_err(explain)?;
        let crew = crew_access::other_crew(&store, &own.id, &args.crew).map_err(explain)?;
        let target = match args.bot.as_deref() {
            None => None,
            Some(handle) => Some(find(&store, &crew, handle).map_err(explain)?),
        };
        let already =
            crew_access::reaches(daemon, &store, me, &crew.id, target.as_ref().map(|t| &t.id))
                .map_err(explain)?;
        if already {
            return Ok(granted(
                &crew.name,
                target.as_ref().map(|t| t.handle.as_str()),
                "You can already reach it.",
            ));
        }
        (crew, target, record.permission_mode)
    };
    let reach = Reach {
        crew: crew.id.clone(),
        target: target.as_ref().map(|t| t.id.clone()),
    };
    let handle = target.as_ref().map(|t| t.handle.as_str());
    if mode == PermissionMode::BypassPermissions {
        daemon.crew_access.allow(me, reach);
        return Ok(granted(
            &crew.name,
            handle,
            "You bypass permissions, so you have it for this turn without asking.",
        ));
    }
    let input = json!({
        "crew_id": crew.id,
        "crew": crew.name,
        "bot_id": target.as_ref().map(|t| &t.id),
        "bot": target.as_ref().map(|t| &t.name),
        "handle": handle,
        "access": args.access,
        "why": why,
    });
    match approvals::ask(daemon, me, generation, CREW_ACCESS_TOOL, &input, "").await {
        Some(Answer::Allowed { input }) => {
            let scope = match input {
                Some(answer) => {
                    serde_json::from_str::<Answered>(&answer)
                        .map_err(|err| format!("the owner's answer could not be read: {err}"))?
                        .scope
                }
                None => Scope::Once,
            };
            let now = daemon.clock.now_ms();
            let how = match scope {
                Scope::Once => {
                    daemon.crew_access.allow(me, reach);
                    "The owner allowed it for this turn only."
                }
                Scope::Bot | Scope::Crew => {
                    let target = if scope == Scope::Bot {
                        reach.target.as_ref()
                    } else {
                        None
                    };
                    daemon
                        .store()
                        .grant_crew_access(me, &crew.id, target, true, now)
                        .map_err(|err| explain(err.into()))?;
                    "The owner allowed it for good."
                }
            };
            let whole = scope == Scope::Crew;
            Ok(granted(&crew.name, if whole { None } else { handle }, how))
        }
        Some(Answer::Denied { note }) => {
            let why = note.map_or_else(String::new, |note| format!(" They said: {note}"));
            Ok(json!({
                "allowed": false,
                "note": format!(
                    "The owner did not let you reach {}.{why} Do without it.",
                    crew.name
                ),
            }))
        }
        Some(Answer::Expired) => Ok(json!({
            "allowed": false,
            "note": "The owner did not answer in time. Tell the owner, and ask again later if it \
                still helps.",
        })),
        None => Err("Botloft could not ask the owner.".to_owned()),
    }
}

fn find(
    store: &botloft_store::Store,
    crew: &botloft_core::protocol::Crew,
    handle: &str,
) -> ApiResult<botloft_store::BotRecord> {
    let handle = slug::slugify(handle.trim().trim_start_matches('@'), "");
    let found = match handle.as_str() {
        "" => None,
        handle => store.active_bot_by_handle(&crew.id, handle)?,
    };
    let id = found.ok_or_else(|| {
        ApiError::NotFound(format!(
            "no bot of the crew {} answers to @{handle}; ask the owner for its name",
            crew.name
        ))
    })?;
    bots::find(store, &id)
}

fn granted(crew: &str, handle: Option<&str>, how: &str) -> Value {
    let whom = handle.map_or_else(
        || format!("send_message(to: <handle>, crew: \"{crew}\")"),
        |handle| format!("send_message(to: \"{handle}\", crew: \"{crew}\")"),
    );
    json!({
        "allowed": true,
        "crew": crew,
        "bot": handle,
        "note": format!(
            "{how} See who you can reach with crew_roster(crew: \"{crew}\") and write with \
             {whom}. Tell them who you are and what you need: they do not know you."
        ),
    })
}

/// `crew_roster` with `crew`: the bots of another crew the caller may reach.
pub(super) fn roster(daemon: &Daemon, me: &BotId, name: &str) -> ApiResult<Value> {
    let (crew, mine) = {
        let store = daemon.store();
        let (own, mine) = bots::active(&store, me)?;
        (crew_access::other_crew(&store, &own.id, name)?, mine)
    };
    let list = bots::list(
        daemon,
        BotsListParams {
            crew_id: Some(crew.id.clone()),
        },
    )?;
    let store = daemon.store();
    let mut reachable = Vec::new();
    for bot in list {
        let record = bots::find(&store, &bot.id)?;
        if crew_access::may_talk(daemon, &store, &mine, &record)? {
            reachable.push(json!({
                "handle": bot.handle,
                "name": bot.name,
                "role": bot.role,
                "state": bot.state,
                "chief": lead::is_lead(&crew, &bot.id),
            }));
        }
    }
    if reachable.is_empty() {
        return Err(ApiError::Conflict(format!(
            "you cannot reach the crew {}; if the owner wants you to, call ask_crew_access with \
             what you need and why",
            crew.name
        )));
    }
    Ok(json!({
        "crew": crew.name,
        "bots": reachable,
        "note": format!("Write with send_message(to, crew: \"{}\").", crew.name),
    }))
}

/// The owner's answer to a request (`approvals.answer` with `input`): how
/// long, checked before the request is settled. "This bot" needs a bot.
pub(crate) fn check_answer(asked: &str, answer: &str) -> ApiResult<String> {
    let answered: Answered = serde_json::from_str(answer)
        .map_err(|_| ApiError::validation("scope must be \"once\", \"bot\" or \"crew\""))?;
    let asked: Value = serde_json::from_str(asked)
        .map_err(|_| ApiError::validation("the request could not be read"))?;
    if answered.scope == Scope::Bot && asked["bot_id"].is_null() {
        return Err(ApiError::validation(
            "the bot asked for the whole crew: allow it now or for the crew",
        ));
    }
    Ok(answer.to_owned())
}
