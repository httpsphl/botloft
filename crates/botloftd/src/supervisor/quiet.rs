//! A turn that went quiet is stopped (spec 7.9). A busy bot whose process
//! printed nothing for half an hour, with no request waiting for the owner,
//! has its turn interrupted; if the process does not answer that either,
//! it starts again and resumes the conversation.

use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotState, ChatBody, NoticeCode, NoticeItem, NoticeLevel};
use tokio::time::Instant;
use tracing::{info, warn};

use super::Supervisor;
use super::slot::Slot;
use crate::chat::{control, items};
use crate::state::Daemon;

/// How long a working process may print nothing.
const QUIET_FOR: Duration = Duration::from_secs(30 * 60);
/// How long a process asked to stop has before it starts again.
const ANSWER_WITHIN: Duration = Duration::from_secs(2 * 60);
/// A pass this late means the computer was asleep, not the bots quiet.
const ASLEEP_AFTER: Duration = Duration::from_secs(60);

/// What a pass does to a quiet bot.
enum Step {
    /// Ask the process to end the turn.
    Stop,
    /// It did not answer: start it again.
    Restart,
}

impl Supervisor {
    /// The process printed something: its turn is not stuck.
    pub(crate) fn heard(&self, bot: &BotId, generation: u64) {
        let mut inner = self.lock();
        if let Some(slot) = inner.slots.get_mut(bot)
            && slot.generation == Some(generation)
        {
            slot.quiet_since = Some(Instant::now());
            slot.quiet_stop = None;
        }
    }

    /// Whether Botloft asked the turn that just ended to stop; asks once.
    pub(crate) fn took_quiet_stop(&self, bot: &BotId, generation: u64) -> bool {
        let mut inner = self.lock();
        inner
            .slots
            .get_mut(bot)
            .filter(|slot| slot.generation == Some(generation))
            .is_some_and(|slot| std::mem::take(&mut slot.quiet_stopped))
    }

    /// Stops the turns that went quiet (checked while reconciling).
    pub(super) fn watch_turns(&self, daemon: &Daemon) {
        let now = Instant::now();
        let steps: Vec<(BotId, Step)> = {
            let mut inner = self.lock();
            let last = inner.watched_at.replace(now);
            // Time the computer spent asleep is nobody's silence.
            if last.is_some_and(|last| now.duration_since(last) > ASLEEP_AFTER) {
                for slot in inner.slots.values_mut() {
                    slot.quiet_since = slot.quiet_since.map(|_| now);
                    slot.quiet_stop = slot.quiet_stop.map(|_| now);
                }
                return;
            }
            inner
                .slots
                .iter_mut()
                .filter_map(|(bot, slot)| step(slot, now).map(|step| (bot.clone(), step)))
                .collect()
        };
        for (bot, step) in steps {
            match step {
                Step::Stop => self.stop_quiet(daemon, &bot),
                Step::Restart => {
                    info!(bot = %bot, "a quiet turn did not stop: starting the bot again");
                    self.restart(&bot, false);
                }
            }
        }
    }

    /// Asks the bot's agent to end the turn, or starts it again when the
    /// agent cannot be asked, and says so in the chat.
    fn stop_quiet(&self, daemon: &Daemon, bot: &BotId) {
        info!(bot = %bot, "a turn printed nothing for {} minutes: stopping it", QUIET_FOR.as_secs() / 60);
        let agent = match daemon.store().bot(bot) {
            Ok(Some(record)) => crate::agent::of(record.agent),
            Ok(None) => return,
            Err(err) => {
                warn!(bot = %bot, "could not read a quiet bot: {err}");
                return;
            }
        };
        let asked = agent.is_some_and(|agent| agent.can_interrupt())
            && self
                .write_control(bot, control::interrupt_request())
                .is_ok();
        if !asked {
            self.restart(bot, false);
        }
        items::add(
            daemon,
            bot,
            ChatBody::Notice(NoticeItem {
                level: NoticeLevel::Warning,
                code: Some(NoticeCode::StuckTurn),
                text: "Nothing happened in this turn for 30 minutes, so Botloft stopped it."
                    .to_owned(),
            }),
        );
    }
}

/// What to do with `slot` now, if its turn went quiet. Marks the stop so
/// the next pass waits for the answer.
fn step(slot: &mut Slot, now: Instant) -> Option<Step> {
    let working = slot.running.is_some()
        && slot.stop.is_none()
        && slot.state == BotState::Busy
        && slot.approvals == 0
        && (slot.in_turn || !slot.waiting.is_empty());
    if !working {
        return None;
    }
    if let Some(asked) = slot.quiet_stop {
        if now.duration_since(asked) < ANSWER_WITHIN {
            return None;
        }
        slot.quiet_stop = None;
        return Some(Step::Restart);
    }
    let quiet = slot
        .quiet_since
        .is_some_and(|since| now.duration_since(since) >= QUIET_FOR);
    if !quiet {
        return None;
    }
    slot.quiet_stop = Some(now);
    slot.quiet_stopped = true;
    Some(Step::Stop)
}
