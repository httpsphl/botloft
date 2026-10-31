//! Bots that sleep (spec 7.7): a bot idle for the owner's time stops its
//! process, keeping its conversation, and one that never ran here starts
//! so. Whatever comes for it wakes it: the courier, a restart. It starts
//! again resuming the same conversation.

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use tokio::time::Instant;
use tracing::{debug, warn};

use super::Supervisor;
use super::slot::{Slot, StopIntent};

/// What a bot that should run does about sleep at this pass.
pub(super) enum Rest {
    /// It sleeps, or just went to: nothing to start.
    Asleep,
    /// It starts, or keeps running.
    Awake,
}

impl Supervisor {
    /// Something waits for `bot`: a sleeping bot starts again, resuming its
    /// conversation. Any other bot is left as it is.
    pub fn wake_up(&self, bot: &BotId) {
        let mut inner = self.lock();
        let Some(slot) = inner.slots.get_mut(bot) else {
            return;
        };
        if slot.state != BotState::Sleeping || slot.wake_asked {
            return;
        }
        slot.wake_asked = true;
        drop(inner);
        debug!(bot = %bot, "waking a sleeping bot");
        self.wake();
    }

    /// Puts a bot to sleep, or keeps it asleep, when the owner's time says
    /// so (`None` never). Called for bots that should run, under the lock.
    pub(super) fn rest(
        &self,
        bot: &BotId,
        slot: &mut Slot,
        now: Instant,
        after: Option<Duration>,
    ) -> Rest {
        let Some(after) = after else {
            if slot.state == BotState::Sleeping && slot.running.is_none() {
                // Sleep was turned off: it starts like any bot.
                slot.wake_asked = true;
            }
            return Rest::Awake;
        };
        let settled = slot.running.is_none() && !slot.launching && slot.stop.is_none();
        if settled && slot.state == BotState::Sleeping {
            return if slot.wake_asked {
                Rest::Awake
            } else {
                Rest::Asleep
            };
        }
        // Never run here, or back from a pause: nothing for it yet.
        if settled && slot.state == BotState::Offline && !slot.wake_asked {
            self.set_state(bot, slot, BotState::Sleeping);
            return Rest::Asleep;
        }
        if Self::tired(slot, now, after) {
            slot.stop = Some(StopIntent::Halt(BotState::Sleeping));
            if let Some(running) = &slot.running
                && let Err(err) = running.control.kill()
            {
                warn!(bot = %bot, "could not put the bot to sleep: {err}");
            }
            debug!(bot = %bot, "the bot goes to sleep");
            return Rest::Asleep;
        }
        Rest::Awake
    }

    /// Idle for `after` with nothing left running: no turn, no message on
    /// its way, no approval, no subagent and no background command or
    /// monitor, which would die with the process.
    fn tired(slot: &Slot, now: Instant, after: Duration) -> bool {
        slot.running.is_some()
            && slot.stop.is_none()
            && !slot.launching
            && slot.state == BotState::Idle
            && !slot.has_work()
            && slot.tasks == 0
            && !slot.restart_when_idle
            && slot
                .idle_since
                .is_some_and(|since| now.saturating_duration_since(since) >= after)
    }
}
