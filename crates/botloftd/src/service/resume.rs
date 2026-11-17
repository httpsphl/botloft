//! A turn the account's usage limit cut short goes on by itself once the
//! limit resets (spec 7.3): the bot waits in `rate_limited`, and Botloft
//! leaves a note in its queue that the courier sends when the bot takes
//! messages again, as it does with the owner's messages that waited.

use botloft_core::ids::{BotId, MessageId};
use botloft_core::protocol::{Message, MessageKind, SenderKind};
use tracing::warn;

use super::{bots, messages};
use crate::state::Daemon;

/// How long a limit lasts when the agent gives no reset time, or one that
/// already passed.
const DEFAULT_LIMIT_MS: i64 = 5 * 60 * 1000;

/// What the bot reads once the limit resets. The owner sees it waiting in
/// the chat before that, so it reads right at either time.
pub const NOTE: &str = "Your last turn stopped when the account reached its usage limit. This \
                        message comes once the limit resets: carry on with that work from where \
                        you stopped, without redoing what is already done. If it was finished, \
                        there is nothing more to do.";

/// When a limit that resets at `resets_at` (Unix ms) ends. A reset time
/// that is missing or already past would let the bot try again at once.
pub(crate) fn until(daemon: &Daemon, resets_at: Option<i64>) -> i64 {
    let now = daemon.clock.now_ms();
    resets_at
        .filter(|at| *at > now)
        .unwrap_or(now + DEFAULT_LIMIT_MS)
}

/// The usage limit cut the bot's turn short: it waits until the limit
/// resets, then carries on.
pub(crate) fn cut_turn(daemon: &Daemon, bot: &BotId, generation: u64, resets_at: Option<i64>) {
    daemon
        .supervisor
        .rate_limited(bot, generation, until(daemon, resets_at));
    leave_note(daemon, bot);
}

/// One note waiting is enough: a turn the limit cuts again before the bot
/// begins the first adds nothing.
fn leave_note(daemon: &Daemon, bot: &BotId) {
    let store = daemon.store();
    match store.unread_note(bot, NOTE) {
        Ok(false) => {}
        Ok(true) => return,
        Err(err) => {
            warn!(bot = %bot, "could not look for a note waiting after the limit: {err}");
            return;
        }
    }
    let Ok((crew, _)) = bots::active(&store, bot) else {
        return;
    };
    let note = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: bot.clone(),
        kind: MessageKind::System,
        body: NOTE.to_owned(),
        task_id: None,
        routine_id: None,
        question_id: None,
        reply_to: None,
        attachments: Vec::new(),
        created_at: daemon.clock.now_ms(),
    };
    if let Err(err) = messages::post(daemon, &store, note, None) {
        warn!(bot = %bot, "could not leave the note to carry on after the limit: {err}");
    }
}
