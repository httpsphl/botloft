//! A new launch setting reaches the bot (spec 7.4): its permission mode or
//! its model. Claude Code only reads `--permission-mode` and `--model` when
//! it starts, so the process restarts, resuming the conversation, as soon
//! as nothing is in progress: at once when idle, or when the turn that is
//! running (and its approvals) ends.

use botloft_core::ids::BotId;
use botloft_core::protocol::PermissionMode;
use tracing::warn;

use super::Supervisor;
use super::slot::{Slot, StopIntent};

impl Supervisor {
    pub fn launch_settings_changed(&self, bot: &BotId) {
        let mut inner = self.lock();
        let Some(slot) = inner.slots.get_mut(bot) else {
            return;
        };
        if slot.running.is_none() || slot.stop.is_some() {
            // The next start reads the new settings anyway.
            return;
        }
        if slot.turns == 0 && slot.approvals == 0 {
            self.relaunch(bot, slot);
        } else {
            slot.restart_when_idle = true;
        }
    }

    /// Whether new settings wait for the running turn to end.
    pub fn relaunch_pending(&self, bot: &BotId) -> bool {
        self.lock()
            .slots
            .get(bot)
            .is_some_and(|slot| slot.restart_when_idle)
    }

    /// The `--permission-mode` the running process started with.
    pub fn launched_mode(&self, bot: &BotId) -> Option<PermissionMode> {
        self.lock()
            .slots
            .get(bot)
            .and_then(|slot| slot.running.as_ref())
            .map(|running| running.permission_mode)
    }

    /// Called when a turn or an approval ends.
    pub(super) fn relaunch_if_idle(&self, bot: &BotId, slot: &mut Slot) {
        if slot.restart_when_idle && slot.turns == 0 && slot.approvals == 0 && slot.stop.is_none() {
            self.relaunch(bot, slot);
        }
    }

    fn relaunch(&self, bot: &BotId, slot: &mut Slot) {
        slot.restart_when_idle = false;
        slot.stop = Some(StopIntent::Restart { fresh: false });
        if let Some(running) = &slot.running
            && let Err(err) = running.control.kill()
        {
            warn!(bot = %bot, "could not restart the bot for its new settings: {err}");
        }
        self.wake();
    }
}
