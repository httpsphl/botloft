//! `retry_agent` (spec 10.6): the crew's chief has a bot whose turn failed
//! try the same request again, with what to do differently if it knows.
//! The request goes back to the bot from the chief, at most twice an hour
//! for each bot, so a request that keeps failing does not loop.

use botloft_core::ids::BotId;
use serde::Deserialize;
use serde_json::{Value, json};

use super::calls::explain;
use crate::failures::{RETRIES_PER_HOUR, quote};
use crate::service::tasks::{self, BotMessage};
use crate::service::{bots, lead};
use crate::state::Daemon;

pub const RETRY_AGENT: &str = "retry_agent";

/// The longest note the chief adds, in characters.
const NOTE_MAX: usize = 1_000;

pub(super) fn tool() -> Value {
    json!({
        "name": RETRY_AGENT,
        "title": "Have an agent try again",
        "description": "Only the crew's chief: after Botloft tells you a bot's turn failed, have \
            that bot try the same request again. Add a note with what to do differently if you \
            know it. At most twice an hour for each bot; past that, tell the owner.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "bot": { "type": "string", "description": "The bot's handle, like @writer." },
                "note": { "type": "string", "maxLength": NOTE_MAX },
            },
            "required": ["bot"],
            "additionalProperties": false,
        },
    })
}

#[derive(Deserialize)]
pub(super) struct Args {
    bot: String,
    #[serde(default)]
    note: Option<String>,
}

pub(super) fn call(daemon: &Daemon, caller: &BotId, args: Args) -> Result<Value, String> {
    let handle = args.bot.trim().trim_start_matches('@').to_owned();
    let store = daemon.store();
    let (crew, _) = bots::active(&store, caller).map_err(explain)?;
    if !lead::is_lead(&crew, caller) {
        return Err(
            "only the crew's chief can have a bot try again; tell the chief with send_message"
                .to_owned(),
        );
    }
    let target = store
        .bots(Some(&crew.id), false)
        .map_err(|err| explain(err.into()))?
        .into_iter()
        .find(|bot| bot.handle == handle)
        .ok_or_else(|| format!("there is no @{handle} in your crew"))?;
    let now = daemon.clock.now_ms();
    let failure = daemon
        .failures
        .last(&target.id, now)
        .ok_or_else(|| format!("@{handle} has no failed turn to try again"))?;
    let request = failure
        .message
        .as_ref()
        .and_then(|id| store.message(id).ok().flatten())
        .ok_or_else(|| {
            format!(
                "Botloft does not know what @{handle}'s turn was working on; send @{handle} \
                 what to do with send_message"
            )
        })?;
    drop(store);
    if !daemon.failures.take_retry(&target.id, now) {
        return Err(format!(
            "@{handle} already tried again {RETRIES_PER_HOUR} times this hour; tell the owner \
             instead"
        ));
    }
    let note = args
        .note
        .as_deref()
        .map(|note| quote(note, NOTE_MAX))
        .filter(|note| !note.is_empty())
        .map_or_else(String::new, |note| {
            format!("\nWhat to do differently: {note}")
        });
    let body = format!(
        "Please try this again. Your last try did not finish: {}.{note}\n\n{}",
        failure.reason.describe(),
        request.body
    );
    let (message, _) = tasks::send(
        daemon,
        caller,
        BotMessage {
            to: handle.clone(),
            crew: None,
            body,
            task: false,
            deadline_minutes: None,
        },
    )
    .map_err(explain)?;
    Ok(json!({
        "message_id": message.id,
        "note": format!(
            "@{handle} got the request again. Its answer, or another failure, comes later; you \
             can end your turn."
        ),
    }))
}
