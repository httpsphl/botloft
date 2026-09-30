//! Which conversation a bot's next process resumes (spec 7.3). Claude Code
//! has nothing on disk for a conversation before a turn began in it, so
//! only one that had a turn is remembered; one that Claude Code lost, or
//! that a new one took the place of, is forgotten.

use botloft_core::ids::BotId;
use tracing::warn;

use super::Supervisor;
use crate::state::Daemon;

impl Supervisor {
    /// A turn began in `session`: Claude Code has it on disk from here on,
    /// and the next start resumes it.
    pub(crate) fn remember_session(&self, bot: &BotId, session: &str) {
        self.lock().sessions.insert(bot.clone(), session.to_owned());
    }

    /// Claude Code does not have the conversation this process was told to
    /// resume, and is about to exit. The next process starts a new one,
    /// however long this one takes to go.
    pub(crate) fn session_missing(&self, bot: &BotId, generation: u64) {
        self.with_current(bot, generation, |_, slot| slot.fresh_next = true);
    }

    /// Takes the conversations that new ones replaced out of the database,
    /// so a daemon that restarts does not resume them. Called without the
    /// supervisor lock (lock order, see `mod.rs`).
    pub(super) fn forget_replaced(&self, daemon: &Daemon) {
        let replaced = std::mem::take(&mut self.lock().replaced);
        if replaced.is_empty() {
            return;
        }
        let store = daemon.store();
        for (bot, session) in replaced {
            // A turn of the new conversation may have been saved already.
            let stored = store.session_id(&bot).ok().flatten();
            if stored.as_deref() == Some(session.as_str())
                && let Err(err) = store.set_session_id(&bot, None)
            {
                warn!(bot = %bot, "could not forget the old session id: {err}");
            }
        }
    }
}
