//! `send_signal` (spec 20.13): the bot tells its crew that something
//! happened, and the routines that wait for that signal run.

use std::collections::HashMap;

use botloft_core::ids::BotId;
use botloft_core::protocol::{RunStatus, Schedule, SkipReason};
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::routines::signal::{self, NOTE_MAX_CHARS};
use crate::service::bots;
use crate::state::Daemon;

pub const SEND_SIGNAL: &str = "send_signal";

pub(super) fn tool() -> Value {
    json!({
        "name": SEND_SIGNAL,
        "title": "Send a signal",
        "description": "Tells your crew that something happened, by the name of a signal: every \
            routine of the crew that waits for that signal runs now, each in its own bot's chat. \
            crew_roster lists the signals the routines wait for. Send one when the thing it names \
            is done (\"report-ready\" once the report is saved), not to talk to a bot: for that, \
            use send_message.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "The signal, as crew_roster lists it (\"report-ready\").",
                },
                "note": {
                    "type": "string",
                    "maxLength": NOTE_MAX_CHARS,
                    "description": "What the bots that run should know, such as where the file is.",
                },
            },
            "required": ["name"],
            "additionalProperties": false,
        },
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SignalArgs {
    name: String,
    #[serde(default)]
    note: Option<String>,
}

pub(super) fn send(daemon: &Daemon, bot: &BotId, args: SignalArgs) -> Result<Value, String> {
    let (name, reached) =
        signal::send(daemon, bot, &args.name, args.note.as_deref()).map_err(explain)?;
    let handles = handles(daemon, bot)?;
    let routines: Vec<Value> = reached
        .iter()
        .map(|each| {
            let mut out = json!({
                "routine": each.routine.name,
                "bot": handles.get(&each.routine.bot_id).map(|h| format!("@{h}")),
                "ran": each.run.status == RunStatus::Queued,
            });
            if let Some(reason) = each.run.reason {
                out["why_not"] = json!(why_not(reason));
            }
            out
        })
        .collect();
    let note = if routines.is_empty() {
        "No routine of your crew waits for this signal, so nothing ran. crew_roster lists the \
         signals the routines wait for."
    } else {
        "Each routine that ran works in its own bot's chat; you don't get its answer."
    };
    Ok(json!({ "signal": name, "routines": routines, "note": note }))
}

fn why_not(reason: SkipReason) -> &'static str {
    match reason {
        SkipReason::TooSoon => "it ran for a signal less than 5 minutes ago",
        SkipReason::BotPaused => "its bot is paused",
        SkipReason::Overlap => "its last run has not finished",
        SkipReason::Missed => "missed",
    }
}

/// The signals the crew's routines wait for, for `crew_roster`.
pub(super) fn waited_for(daemon: &Daemon, bot: &BotId) -> Result<Vec<Value>, String> {
    let handles = handles(daemon, bot)?;
    let store = daemon.store();
    let (crew, _) = bots::active(&store, bot).map_err(explain)?;
    let routines = store
        .signal_routines(&crew.id, None)
        .map_err(|err| explain(err.into()))?;
    Ok(routines
        .into_iter()
        .filter_map(|routine| match routine.schedule {
            Schedule::Signal { name } => Some(json!({
                "signal": name,
                "routine": routine.name,
                "bot": handles.get(&routine.bot_id).map(|h| format!("@{h}")),
            })),
            _ => None,
        })
        .collect())
}

/// The handles of the bot's crew, by id.
fn handles(daemon: &Daemon, bot: &BotId) -> Result<HashMap<BotId, String>, String> {
    let store = daemon.store();
    let (crew, _) = bots::active(&store, bot).map_err(explain)?;
    Ok(store
        .bots(Some(&crew.id), true)
        .map_err(|err| explain(err.into()))?
        .into_iter()
        .map(|record| (record.id, record.handle))
        .collect())
}
