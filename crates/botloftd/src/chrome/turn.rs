//! The Chrome's turn (spec 37.4): one agent at a time acts in the owner's
//! Chrome. The owner uses it as always; every agent works in its own tabs.

use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::ChromeTurn;
use tokio::time::Instant;

use crate::state::{Daemon, Event};

/// How long an agent keeps the turn without an action in the Chrome.
pub(super) const IDLE: Duration = Duration::from_secs(120);
/// How long another agent waits for the turn before it hears that the
/// Chrome is busy.
pub(super) const WAIT: Duration = Duration::from_secs(120);
const POLL: Duration = Duration::from_secs(1);

#[derive(Default)]
pub(super) struct Turn {
    holder: Mutex<Option<Holder>>,
    /// Agents the owner stopped in the Chrome, until their turn ends.
    stopped: Mutex<HashSet<BotId>>,
}

struct Holder {
    bot: BotId,
    last: Instant,
}

impl Turn {
    fn lock(&self) -> MutexGuard<'_, Option<Holder>> {
        self.holder
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn stops(&self) -> MutexGuard<'_, HashSet<BotId>> {
        self.stopped
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The agent with the turn; one idle for too long has none.
    pub(super) fn holder(&self) -> Option<BotId> {
        self.lock()
            .as_ref()
            .filter(|holder| holder.last.elapsed() < IDLE)
            .map(|holder| holder.bot.clone())
    }

    /// Takes the turn for `bot`: whether it changed hands, or the agent
    /// that has it.
    fn try_take(&self, bot: &BotId) -> Result<bool, BotId> {
        let mut holder = self.lock();
        match holder.as_mut() {
            Some(have) if have.bot == *bot => {
                have.last = Instant::now();
                Ok(false)
            }
            Some(have) if have.last.elapsed() < IDLE => Err(have.bot.clone()),
            _ => {
                *holder = Some(Holder {
                    bot: bot.clone(),
                    last: Instant::now(),
                });
                Ok(true)
            }
        }
    }

    /// Lets go of the turn when `bot` has it; whether it did.
    fn let_go(&self, bot: &BotId) -> bool {
        let mut holder = self.lock();
        if holder.as_ref().is_some_and(|have| have.bot == *bot) {
            *holder = None;
            return true;
        }
        false
    }

    pub(super) fn stop(&self, bot: &BotId) {
        self.stops().insert(bot.clone());
    }

    pub(super) fn is_stopped(&self, bot: &BotId) -> bool {
        self.stops().contains(bot)
    }

    pub(super) fn forget_stop(&self, bot: &BotId) {
        self.stops().remove(bot);
    }

    pub(super) fn forget_stops(&self) {
        self.stops().clear();
    }
}

/// Waits for the turn: `Ok` once `bot` has it, or the agent that kept it
/// for all of `WAIT`.
pub(super) async fn take(daemon: &Daemon, bot: &BotId) -> Result<(), BotId> {
    let until = Instant::now() + WAIT;
    loop {
        match daemon.chrome.turn.try_take(bot) {
            Ok(changed) => {
                if changed {
                    announce(daemon);
                }
                return Ok(());
            }
            Err(holder) if Instant::now() >= until => return Err(holder),
            Err(_) => tokio::time::sleep(POLL).await,
        }
    }
}

/// `bot` lets go of the turn, when it has it.
pub(super) fn release(daemon: &Daemon, bot: &BotId) {
    if daemon.chrome.turn.let_go(bot) {
        announce(daemon);
    }
}

fn announce(daemon: &Daemon) {
    daemon.emit(Event::ChromeTurn(ChromeTurn {
        bot_id: daemon.chrome.turn.holder(),
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bot(n: u8) -> BotId {
        format!("bot_01J00000000000000000000{n:03}")
            .parse()
            .expect("bot id")
    }

    #[tokio::test(start_paused = true)]
    async fn one_agent_at_a_time_and_an_idle_one_lets_go() {
        let turn = Turn::default();
        assert_eq!(turn.try_take(&bot(1)), Ok(true));
        assert_eq!(turn.try_take(&bot(1)), Ok(false));
        assert_eq!(turn.try_take(&bot(2)), Err(bot(1)));
        tokio::time::advance(IDLE).await;
        assert_eq!(turn.holder(), None);
        assert_eq!(turn.try_take(&bot(2)), Ok(true));
        assert!(!turn.let_go(&bot(1)));
        assert!(turn.let_go(&bot(2)));
        assert_eq!(turn.holder(), None);
    }

    #[test]
    fn a_stop_holds_until_it_is_forgotten() {
        let turn = Turn::default();
        turn.stop(&bot(1));
        assert!(turn.is_stopped(&bot(1)));
        assert!(!turn.is_stopped(&bot(2)));
        turn.forget_stop(&bot(1));
        assert!(!turn.is_stopped(&bot(1)));
    }
}
