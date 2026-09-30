//! State changes driven by what the bot's process prints and by approvals
//! (spec 7.2). Each call names the generation it is about; news from a
//! process that is gone is ignored.

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use tracing::{debug, warn};

use super::slot::{Slot, StopIntent};
use super::{Inner, Supervisor};

impl Supervisor {
    /// Runs `change` on the slot of `bot` if `generation` is still running.
    pub(super) fn with_current(
        &self,
        bot: &BotId,
        generation: u64,
        change: impl FnOnce(&Self, &mut Slot),
    ) {
        let mut inner = self.lock();
        let Inner { slots, .. } = &mut *inner;
        if let Some(slot) = slots.get_mut(bot)
            && slot.generation == Some(generation)
            && slot.running.is_some()
        {
            change(self, slot);
        }
    }

    /// The process lived through its first moments: it is ready.
    pub(crate) fn ready(&self, bot: &BotId, generation: u64) {
        self.with_current(bot, generation, |supervisor, slot| {
            if slot.state == BotState::Launching {
                supervisor.set_state(bot, slot, slot.working_state());
            }
        });
    }

    /// A turn began (`system/init`, spec 8.1). With no message written
    /// for it, it is Claude Code's own: a subagent finished and it went on
    /// by itself. The bot is working all the same.
    pub(crate) fn turn_began(&self, bot: &BotId, generation: u64) {
        self.with_current(bot, generation, |supervisor, slot| {
            slot.in_turn = true;
            if slot.is_working() {
                supervisor.set_state(bot, slot, slot.working_state());
            }
        });
    }

    /// Claude Code gave back a user message (`isReplay`, spec 9.1): the
    /// one written with `uuid`, now in a turn of its own or inside the turn
    /// already running, which then ends with a single `result` for both.
    /// `/compact` also gives back its output, with a uuid nobody wrote.
    pub(crate) fn message_began(&self, bot: &BotId, generation: u64, uuid: Option<&str>) {
        self.with_current(bot, generation, |supervisor, slot| {
            if let Some(at) = slot.waiting.iter().position(|w| Some(w.as_str()) == uuid) {
                slot.waiting.remove(at);
            }
            slot.in_turn = true;
            if slot.is_working() {
                supervisor.set_state(bot, slot, slot.working_state());
            }
        });
    }

    /// Claude Code lists the subagents it runs in the background (`system/
    /// background_tasks_changed`); the whole list comes each time. They work
    /// after the turn that started them has ended.
    pub(crate) fn agents_running(&self, bot: &BotId, generation: u64, agents: u32) {
        self.with_current(bot, generation, |supervisor, slot| {
            slot.agents = agents;
            if slot.is_working() {
                supervisor.set_state(bot, slot, slot.working_state());
            }
            supervisor.relaunch_if_idle(bot, slot);
        });
    }

    /// A `result` closed the turn, with every message taken up in it.
    pub(crate) fn turn_ended(&self, bot: &BotId, generation: u64) {
        self.with_current(bot, generation, |supervisor, slot| {
            slot.in_turn = false;
            if slot.is_working() {
                supervisor.set_state(bot, slot, slot.working_state());
            }
            supervisor.relaunch_if_idle(bot, slot);
        });
    }

    /// A tool waits for the owner (spec 10.1).
    pub fn approval_opened(&self, bot: &BotId, generation: u64) {
        self.with_current(bot, generation, |supervisor, slot| {
            slot.approvals += 1;
            if slot.is_working() {
                supervisor.set_state(bot, slot, slot.working_state());
            }
        });
    }

    pub fn approval_closed(&self, bot: &BotId, generation: u64) {
        self.with_current(bot, generation, |supervisor, slot| {
            slot.approvals = slot.approvals.saturating_sub(1);
            if slot.is_working() {
                supervisor.set_state(bot, slot, slot.working_state());
            }
        });
    }

    /// The account hit its usage limit until `until` (Unix ms). Messages
    /// wait; the bot goes back to work once the limit resets.
    pub(crate) fn rate_limited(&self, bot: &BotId, generation: u64, until: i64) {
        self.with_current(bot, generation, |supervisor, slot| {
            slot.limited_until = Some(until);
            if slot.is_working() || slot.state == BotState::RateLimited {
                debug!(bot = %bot, until, "rate limited");
                supervisor.set_state(bot, slot, BotState::RateLimited);
            }
        });
    }

    /// Claude Code is not signed in: restarting cannot fix that, so the
    /// process stops until the owner restarts the bot or signs in again
    /// (spec 7.3).
    pub(crate) fn signed_out(&self, bot: &BotId, generation: u64) {
        self.recheck_sign_in();
        self.with_current(bot, generation, |supervisor, slot| {
            supervisor.set_state(bot, slot, BotState::AuthError);
            if slot.stop.is_none() {
                slot.stop = Some(StopIntent::Halt(BotState::AuthError));
                if let Some(running) = &slot.running
                    && let Err(err) = running.control.kill()
                {
                    warn!(bot = %bot, "could not stop a signed-out bot: {err}");
                }
            }
        });
    }

    /// Ends every rate limit whose time has come (checked while reconciling).
    pub(super) fn release_limits(&self, now_ms: i64) {
        let mut inner = self.lock();
        let Inner { slots, .. } = &mut *inner;
        for (bot, slot) in slots.iter_mut() {
            let over = slot.limited_until.is_some_and(|until| until <= now_ms);
            if slot.state == BotState::RateLimited && over && slot.running.is_some() {
                slot.limited_until = None;
                let next = slot.working_state();
                self.set_state(bot, slot, next);
            }
        }
    }
}
