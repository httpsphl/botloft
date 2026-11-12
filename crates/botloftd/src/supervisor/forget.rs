//! A deleted bot leaves the supervisor (spec 7.6).

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use tracing::warn;

use super::{Inner, Supervisor};

impl Supervisor {
    /// The bot was deleted: its process is killed at once, without waiting
    /// for its turn, and nothing of it is kept. Whatever the process still
    /// prints is ignored, and its exit finds no slot.
    pub fn forget(&self, bot: &BotId) {
        let mut inner = self.lock();
        let Inner {
            slots,
            tokens,
            sessions,
            gone,
            ..
        } = &mut *inner;
        // A pass that read the bot just before must not start it again.
        gone.insert(bot.clone());
        let conversations: Vec<_> = slots
            .iter()
            .filter(|(_, slot)| slot.bot == *bot)
            .map(|(id, _)| id.clone())
            .collect();
        sessions.retain(|id, _| !id.is_main_of(bot) && !conversations.contains(id));
        for id in conversations {
            let Some(slot) = slots.remove(&id) else {
                continue;
            };
            self.count_busy(slot.state, BotState::Offline);
            if let Some(running) = slot.running {
                tokens.remove(&running.token_hash);
                if let Err(err) = running.control.kill() {
                    warn!(bot = %bot, "could not stop a deleted bot: {err}");
                }
            }
        }
    }
}
