//! A crew's bot whose turn failed is reported to the crew's chief (spec
//! 10.6): a Botloft note with what went wrong and what the bot was working
//! on, so the chief can have it try again (`retry_agent`), tell it what to
//! do differently, or tell the owner. What is kept here (the last failure of
//! each bot, when it was reported, its retries) lives only while the daemon
//! runs: after a restart, a failure from before is not retried.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use botloft_core::ids::{BotId, MessageId};
use botloft_core::protocol::{BotState, Message, MessageKind, SenderKind};
use botloft_store::Store;
use tracing::{debug, warn};

use crate::service::{bots, lead, messages};
use crate::state::Daemon;

/// At most one report about the same bot in this long, so a bot that keeps
/// failing does not keep waking its chief.
const REPORT_GAP_MS: i64 = 10 * 60_000;
/// How many times an hour the chief may have the same bot try again.
pub const RETRIES_PER_HOUR: usize = 2;
const HOUR_MS: i64 = 60 * 60_000;
/// How long a failure can still be tried again.
const RETRY_WITHIN_MS: i64 = 24 * HOUR_MS;
/// The most of the request quoted to the chief, in characters.
const QUOTE_MAX: usize = 1_000;

/// Why a turn did not finish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    /// The agent ended it with an error, as it named it.
    Error(String),
    /// It showed nothing for half an hour and Botloft stopped it (7.9).
    Quiet,
}

impl Reason {
    /// In the words the chief and the bot read.
    pub fn describe(&self) -> String {
        match self {
            Self::Error(error) => format!("it ended with an error ({error})"),
            Self::Quiet => "it showed nothing for half an hour, so Botloft stopped it".to_owned(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Failure {
    /// The message the turn was working on; `None` when it had none.
    pub message: Option<MessageId>,
    pub reason: Reason,
    pub at: i64,
}

#[derive(Default)]
struct Kept {
    last: Option<Failure>,
    reported_at: Option<i64>,
    retries: Vec<i64>,
}

#[derive(Default)]
pub struct Failures {
    bots: Mutex<HashMap<BotId, Kept>>,
}

impl Failures {
    fn lock(&self) -> MutexGuard<'_, HashMap<BotId, Kept>> {
        self.bots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The bot's last failure, while it can still be tried again.
    pub fn last(&self, bot: &BotId, now: i64) -> Option<Failure> {
        self.lock()
            .get(bot)
            .and_then(|kept| kept.last.clone())
            .filter(|failure| now - failure.at < RETRY_WITHIN_MS)
    }

    /// Counts a try again of `bot`, unless it had its share this hour.
    pub fn take_retry(&self, bot: &BotId, now: i64) -> bool {
        let mut bots = self.lock();
        let kept = bots.entry(bot.clone()).or_default();
        kept.retries.retain(|at| now - at < HOUR_MS);
        if kept.retries.len() >= RETRIES_PER_HOUR {
            return false;
        }
        kept.retries.push(now);
        true
    }

    /// Keeps the failure; `true` when the chief should hear of it now.
    fn note(&self, bot: &BotId, failure: Failure) -> bool {
        let mut bots = self.lock();
        let kept = bots.entry(bot.clone()).or_default();
        let at = failure.at;
        kept.last = Some(failure);
        let due = kept
            .reported_at
            .is_none_or(|reported| at - reported >= REPORT_GAP_MS);
        if due {
            kept.reported_at = Some(at);
        }
        due
    }

    /// The bot was deleted.
    pub fn forget(&self, bot: &BotId) {
        self.lock().remove(bot);
    }
}

/// A turn ended. With an error, or stopped because it went quiet, it is a
/// failure; a turn cut by the usage limit is not, since it carries on by
/// itself once the limit resets (7.3).
pub(crate) fn turn_ended(daemon: &Daemon, bot: &BotId, error: Option<&str>, quiet: bool) {
    let reason = match (error, quiet) {
        (_, true) => Reason::Quiet,
        (Some(error), false) => Reason::Error(error.to_owned()),
        (None, false) => return,
    };
    if matches!(
        daemon.supervisor.status(bot),
        Some((BotState::RateLimited, _))
    ) {
        return;
    }
    let store = daemon.store();
    let Ok((crew, me)) = bots::active(&store, bot) else {
        return;
    };
    // Only a crew with a chief, and never the chief about itself.
    let Some(chief) = crew.lead_bot_id.clone().filter(|chief| chief != bot) else {
        return;
    };
    if !lead::is_lead(&crew, &chief) || bots::active(&store, &chief).is_err() {
        return;
    }
    let message = store.last_read_message(bot).ok().flatten();
    let failure = Failure {
        message: message.clone(),
        reason,
        at: daemon.clock.now_ms(),
    };
    if !daemon.failures.note(bot, failure.clone()) {
        debug!(bot = %bot, "a failure the chief heard of a moment ago");
        return;
    }
    let request = message.and_then(|id| store.message(&id).ok().flatten());
    let body = report(&store, &me.handle, &failure.reason, request.as_ref());
    let note = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: chief.clone(),
        kind: MessageKind::System,
        body,
        task_id: None,
        routine_id: None,
        question_id: None,
        reply_to: None,
        attachments: Vec::new(),
        created_at: failure.at,
    };
    match messages::post(daemon, &store, note, None) {
        Ok(_) => debug!(bot = %bot, chief = %chief, "a failed turn went to the chief"),
        Err(err) => warn!(bot = %bot, "could not tell the chief of a failed turn: {err}"),
    }
}

/// Who sent `request`, for the chief.
fn sender(store: &Store, request: &Message) -> String {
    match request.from_kind {
        SenderKind::Owner => "the owner".to_owned(),
        SenderKind::Bot => request
            .from_bot_id
            .as_ref()
            .and_then(|id| store.bot(id).ok().flatten())
            .map_or_else(|| "a bot".to_owned(), |bot| format!("@{}", bot.handle)),
        SenderKind::System => request
            .routine_id
            .as_ref()
            .and_then(|id| store.routine(id).ok().flatten())
            .map_or_else(
                || "Botloft".to_owned(),
                |routine| format!("the routine \"{}\"", routine.name),
            ),
    }
}

/// `text` cut to `max` characters, with an ellipsis when cut.
pub fn quote(text: &str, max: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let cut: String = text.chars().take(max).collect();
    format!("{}…", cut.trim_end())
}

/// What the chief reads.
fn report(store: &Store, handle: &str, reason: &Reason, request: Option<&Message>) -> String {
    let what = match reason {
        Reason::Error(_) => "failed",
        Reason::Quiet => "stopped",
    };
    let mut text = format!("@{handle}'s turn {what}: {}.", reason.describe());
    match request {
        Some(request) => {
            let quoted = quote(&request.body, QUOTE_MAX)
                .lines()
                .map(|line| format!("> {line}"))
                .collect::<Vec<_>>()
                .join("\n");
            text.push_str(&format!(
                "\nIt was working on this, from {}:\n{quoted}\n\nAs the crew's chief: if it \
                 matters, have @{handle} try it again with retry_agent (at most \
                 {RETRIES_PER_HOUR} times an hour), send @{handle} what to do differently with \
                 send_message, or tell the owner. If it does not matter, do nothing.",
                sender(store, request)
            ));
        }
        None => text.push_str(&format!(
            "\nIt was not working on a message, so there is nothing to try again. Ask @{handle} \
             what happened with send_message, or tell the owner, if it matters."
        )),
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(at: i64) -> Failure {
        Failure {
            message: None,
            reason: Reason::Quiet,
            at,
        }
    }

    #[test]
    fn the_chief_hears_of_a_bot_at_most_every_ten_minutes() {
        let failures = Failures::default();
        let bot = BotId::generate();
        assert!(failures.note(&bot, failure(0)));
        assert!(!failures.note(&bot, failure(REPORT_GAP_MS - 1)));
        assert!(failures.note(&bot, failure(REPORT_GAP_MS)));
        assert_eq!(
            failures.last(&bot, REPORT_GAP_MS).map(|f| f.at),
            Some(REPORT_GAP_MS)
        );
        assert!(
            failures
                .last(&bot, REPORT_GAP_MS + RETRY_WITHIN_MS)
                .is_none()
        );
    }

    #[test]
    fn a_bot_tries_again_twice_an_hour() {
        let failures = Failures::default();
        let bot = BotId::generate();
        assert!(failures.take_retry(&bot, 0));
        assert!(failures.take_retry(&bot, 1));
        assert!(!failures.take_retry(&bot, 2));
        assert!(
            failures.take_retry(&bot, HOUR_MS),
            "the first one is an hour old"
        );
    }

    #[test]
    fn a_long_request_is_cut_with_an_ellipsis() {
        assert_eq!(quote("  short  ", 10), "short");
        assert_eq!(quote("abcdefghij klm", 10), "abcdefghij…");
    }
}
