//! Room for running agents (spec 7.8). With `[bots] max_running`, no more
//! agent processes than that run at once, so a machine with fixed memory
//! (a server) is never asked for more than it has. An agent a message waits
//! for comes first and, with no room, takes the room of the agent idle the
//! longest, which goes to sleep. An agent with nothing to do waits asleep
//! and never takes anyone's room, so they do not push each other out.

use std::collections::HashSet;

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::BotState;
use tracing::{debug, warn};

use super::slot::{Slot, StopIntent};
use super::start::Start;
use super::{Inner, Supervisor};

impl Supervisor {
    /// Of the starts this pass decided, the ones that fit under `max` and in
    /// the `fits` more agents the free memory takes (spec 42.4); the others
    /// wait asleep. Called under the supervisor lock.
    pub(super) fn admit(
        &self,
        inner: &mut Inner,
        starts: Vec<Start>,
        with_work: &HashSet<BotId>,
        max: Option<usize>,
        fits: Option<usize>,
    ) -> Vec<Start> {
        if max.is_none() && fits.is_none() {
            return starts;
        }
        let starting: HashSet<&ConversationId> = starts.iter().map(Start::conversation).collect();
        let others = |conversation: &&ConversationId| !starting.contains(*conversation);
        // Processes alive (stopping ones too, until they are gone) or being
        // made by an earlier pass.
        let running = inner
            .slots
            .iter()
            .filter(|(conversation, slot)| {
                (slot.running.is_some() || slot.launching) && others(conversation)
            })
            .count();
        // Being made, they do not show in the free memory yet.
        let launching = inner
            .slots
            .iter()
            .filter(|(conversation, slot)| {
                slot.launching && slot.running.is_none() && others(conversation)
            })
            .count();
        let by_count = max.map(|max| max.saturating_sub(running));
        let by_memory = fits.map(|fits| fits.saturating_sub(launching));
        let mut room = match (by_count, by_memory) {
            (Some(count), Some(memory)) => count.min(memory),
            (Some(room), None) | (None, Some(room)) => room,
            (None, None) => usize::MAX,
        };
        let (urgent, rest): (Vec<_>, Vec<_>) = starts
            .into_iter()
            .partition(|start| with_work.contains(start.bot_id()));
        let mut admitted = Vec::new();
        let mut waiting = Vec::new();
        for start in urgent.into_iter().chain(rest) {
            if room > 0 {
                room -= 1;
                admitted.push(start);
            } else {
                waiting.push(start);
            }
        }
        let short = waiting
            .iter()
            .filter(|start| with_work.contains(start.bot_id()))
            .count();
        // Room already on its way: processes asked to stop for good or to
        // sleep, by an earlier pass or another reason, are not replaced.
        let leaving = inner
            .slots
            .values()
            .filter(|slot| slot.running.is_some() && matches!(slot.stop, Some(StopIntent::Halt(_))))
            .count();
        let short = short.saturating_sub(leaving);
        if short > 0 {
            self.make_room(inner, short);
        }
        for start in waiting {
            let work = with_work.contains(start.bot_id());
            self.wait_for_room(inner, start.conversation(), work);
        }
        admitted
    }

    /// Memory is short: every idle agent but the one idle the shortest goes
    /// to sleep, so the next one to work finds room (spec 42.4).
    pub(super) fn keep_one_idle(&self, inner: &mut Inner) {
        let idle = inner.slots.values().filter(|slot| may_sleep(slot)).count();
        if idle > 1 {
            self.make_room(inner, idle - 1);
        }
    }

    /// Puts to sleep up to `count` idle agents, the longest idle first. Their
    /// exit wakes the loop, and the next pass starts who waited.
    fn make_room(&self, inner: &mut Inner, count: usize) {
        let mut idle: Vec<_> = inner
            .slots
            .iter_mut()
            .filter(|(_, slot)| may_sleep(slot))
            .collect();
        idle.sort_by_key(|(_, slot)| slot.idle_since);
        for (_, slot) in idle.into_iter().take(count) {
            let bot = &slot.bot;
            slot.stop = Some(StopIntent::Halt(BotState::Sleeping));
            if let Some(running) = &slot.running
                && let Err(err) = running.control.kill()
            {
                warn!(bot = %bot, "could not put the bot to sleep: {err}");
            }
            debug!(bot = %bot, "the bot sleeps to make room");
        }
    }
}

/// An agent that may go to sleep to give its room: running, idle, with no
/// work and no background command, which would die with the process (7.7).
fn may_sleep(slot: &Slot) -> bool {
    slot.running.is_some()
        && slot.stop.is_none()
        && slot.state == BotState::Idle
        && !slot.has_work()
        && slot.tasks == 0
}

impl Supervisor {
    /// A start that did not fit: the agent waits asleep. One a message waits
    /// for keeps asking, so it starts as soon as there is room; any other
    /// starts when a message wakes it (7.7).
    fn wait_for_room(&self, inner: &mut Inner, conversation: &ConversationId, work: bool) {
        if let Some(slot) = inner.slots.get_mut(conversation) {
            slot.launching = false;
            slot.wake_asked = work;
            let bot = slot.bot.clone();
            self.set_state(&bot, slot, BotState::Sleeping);
        }
    }
}
