//! Looking after the browsers nobody needs (spec 21.2): one whose bot ended
//! its turn rests at once, idle ones close, and so do those of bots that
//! are paused or archived.

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::BotState;
use tokio::sync::broadcast::error::RecvError;

use super::{Browsers, lock};
use crate::service::{bots, crews};
use crate::state::{Daemon, Event};

const SWEEP_EVERY: Duration = Duration::from_secs(30);
/// How long after its last use an awake browser goes back to rest, when
/// its bot is not in a turn.
const REST_AFTER: Duration = Duration::from_secs(20);

/// What should happen to a bot's browser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Want {
    /// The bot is in a turn: the browser is its to use.
    Keep,
    /// The bot is not working: the browser may rest.
    Rest,
    /// The bot or its crew is paused.
    Close,
    /// The bot or its crew is archived: the profile goes too.
    Forget,
}

pub async fn run(daemon: Arc<Daemon>) {
    let mut tick = tokio::time::interval(SWEEP_EVERY);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    let mut events = daemon.subscribe();
    loop {
        tokio::select! {
            _ = tick.tick() => daemon.browsers.sweep(|bot| want(&daemon, bot)),
            event = events.recv() => match event {
                // The turn ended: the browser rests right away.
                Ok(Event::BotState(change)) if !working(change.state) => {
                    daemon.browsers.rest(&change.bot_id);
                }
                // What was missed, the next sweep sees.
                Ok(_) | Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => return,
            },
        }
    }
}

/// Whether a bot in `state` is in a turn, its browser in use or about to be.
fn working(state: BotState) -> bool {
    matches!(state, BotState::Busy | BotState::NeedsApproval)
}

fn want(daemon: &Daemon, bot: &BotId) -> Want {
    let store = daemon.store();
    let Ok(record) = bots::find(&store, bot) else {
        return Want::Forget;
    };
    let Ok(crew) = crews::find(&store, &record.crew_id) else {
        return Want::Forget;
    };
    if record.archived_at.is_some() || crew.archived_at.is_some() {
        Want::Forget
    } else if record.paused || crew.paused {
        Want::Close
    } else if daemon
        .supervisor
        .status(bot)
        .is_some_and(|(state, _)| working(state))
    {
        Want::Keep
    } else {
        Want::Rest
    }
}

/// What the sweep does with an open browser that is in no tool call.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Stay,
    Rest,
    Close,
    Forget,
}

/// `held` by the owner, and unused by the bot and by them for `quiet`.
/// Watching is not using: the panel opens by itself and would keep the
/// browser open for good.
fn step(want: Want, held: bool, quiet: Duration, idle: Duration) -> Step {
    match want {
        Want::Forget => Step::Forget,
        Want::Close => Step::Close,
        _ if held => Step::Stay,
        _ if quiet >= idle => Step::Close,
        Want::Rest if quiet >= REST_AFTER => Step::Rest,
        Want::Keep | Want::Rest => Step::Stay,
    }
}

impl Browsers {
    /// Closes the browsers nobody used for a while and those of bots that
    /// should not run, and rests the ones whose bot is not working (spec
    /// 21.2). Browsers in a call stay as they are.
    pub fn sweep(&self, want: impl Fn(&BotId) -> Want) {
        let open: Vec<(BotId, bool, Duration)> = lock(&self.slots)
            .iter()
            .filter(|(_, slot)| slot.session.is_some() && slot.calls.try_lock().is_ok())
            .map(|(bot, slot)| {
                let held = slot.held.borrow().is_some();
                (bot.clone(), held, slot.used.elapsed())
            })
            .collect();
        for (bot, held, quiet) in open {
            match step(want(&bot), held, quiet, self.settings.idle) {
                Step::Forget => self.forget(&bot),
                Step::Close => self.close(&bot),
                Step::Rest => self.rest(&bot),
                Step::Stay => {}
            }
        }
    }

    /// Makes room for one more browser by closing the one idle longest;
    /// never one in the owner's hands.
    pub(super) fn make_room(&self, starting: &BotId) {
        let victim = {
            let slots = lock(&self.slots);
            let open = slots.values().filter(|slot| slot.session.is_some()).count();
            if open < self.settings.max_open {
                return;
            }
            slots
                .iter()
                .filter(|(bot, slot)| {
                    *bot != starting
                        && slot.session.is_some()
                        && slot.calls.try_lock().is_ok()
                        && slot.held.borrow().is_none()
                })
                .min_by_key(|(_, slot)| slot.used)
                .map(|(bot, _)| bot.clone())
        };
        if let Some(bot) = victim {
            self.close(&bot);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDLE: Duration = Duration::from_secs(600);

    #[test]
    fn a_browser_rests_when_its_bot_is_not_working_and_closes_when_unused() {
        let secs = Duration::from_secs;
        // In a turn, it stays; out of one, it rests once it is quiet.
        assert_eq!(step(Want::Keep, false, secs(120), IDLE), Step::Stay);
        assert_eq!(step(Want::Rest, false, secs(5), IDLE), Step::Stay);
        assert_eq!(step(Want::Rest, false, secs(20), IDLE), Step::Rest);
        // Unused for long enough, it closes, in a turn or not.
        assert_eq!(step(Want::Rest, false, IDLE, IDLE), Step::Close);
        assert_eq!(step(Want::Keep, false, IDLE, IDLE), Step::Close);
        // In the owner's hands it neither rests nor closes by itself.
        assert_eq!(step(Want::Rest, true, IDLE, IDLE), Step::Stay);
        // A paused or archived bot's browser goes, whoever holds it.
        assert_eq!(step(Want::Close, true, secs(0), IDLE), Step::Close);
        assert_eq!(step(Want::Forget, true, secs(0), IDLE), Step::Forget);
    }

    #[test]
    fn only_a_turn_keeps_a_browser_awake() {
        assert!(working(BotState::Busy));
        assert!(working(BotState::NeedsApproval));
        for state in [
            BotState::Idle,
            BotState::RateLimited,
            BotState::AuthError,
            BotState::Backoff,
            BotState::Offline,
        ] {
            assert!(!working(state), "{state:?}");
        }
    }
}
