//! A new permission mode reaches the bot (spec 7.4). Claude Code only reads
//! `--permission-mode` when it starts, so the process restarts, resuming
//! the conversation, as soon as nothing is in progress: at once when idle,
//! or when the turn that is running (and its approvals) ends.

use botloft_core::ids::BotId;
use tracing::warn;

use super::Supervisor;
use super::slot::{Slot, StopIntent};

impl Supervisor {
    pub fn permission_mode_changed(&self, bot: &BotId) {
        let mut inner = self.lock();
        let Some(slot) = inner.slots.get_mut(bot) else {
            return;
        };
        if slot.running.is_none() || slot.stop.is_some() {
            // The next start reads the new mode anyway.
            return;
        }
        if slot.turns == 0 && slot.approvals == 0 {
            self.restart_for_mode(bot, slot);
        } else {
            slot.restart_when_idle = true;
        }
    }

    /// Whether a new mode waits for the running turn to end.
    pub fn mode_change_pending(&self, bot: &BotId) -> bool {
        self.lock()
            .slots
            .get(bot)
            .is_some_and(|slot| slot.restart_when_idle)
    }

    /// Called when a turn or an approval ends.
    pub(super) fn restart_if_idle_for_mode(&self, bot: &BotId, slot: &mut Slot) {
        if slot.restart_when_idle && slot.turns == 0 && slot.approvals == 0 && slot.stop.is_none() {
            self.restart_for_mode(bot, slot);
        }
    }

    fn restart_for_mode(&self, bot: &BotId, slot: &mut Slot) {
        slot.restart_when_idle = false;
        slot.stop = Some(StopIntent::Restart { fresh: false });
        if let Some(running) = &slot.running
            && let Err(err) = running.control.kill()
        {
            warn!(bot = %bot, "could not restart the bot for its new permission mode: {err}");
        }
        self.wake();
    }
}
