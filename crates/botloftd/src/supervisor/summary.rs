//! The state the app shows for a bot: the summary of its conversations
//! (spec 40.4). It needs the owner if one does, works if one works, sleeps
//! if all sleep and is ready otherwise; its generation is the main one's.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::{BotState, BotStateChanged};

/// What one conversation last announced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Seen {
    pub state: BotState,
    pub generation: Option<u64>,
    pub main: bool,
}

/// What each bot's conversations announced, and the summary the app has.
/// Taken while the supervisor's lock is held, never the other way around.
#[derive(Default)]
pub(super) struct Summaries {
    bots: Mutex<HashMap<BotId, Summary>>,
}

#[derive(Default)]
struct Summary {
    seen: HashMap<ConversationId, Seen>,
    shown: Option<(BotState, Option<u64>)>,
}

impl Summary {
    /// The new summary, when it differs from the one the app has.
    fn update(&mut self, bot: &BotId) -> Option<BotStateChanged> {
        let now = summarize(self.seen.values().copied())?;
        if self.shown == Some(now) {
            return None;
        }
        self.shown = Some(now);
        Some(BotStateChanged {
            bot_id: bot.clone(),
            state: now.0,
            generation: now.1,
        })
    }
}

impl Summaries {
    fn lock(&self) -> MutexGuard<'_, HashMap<BotId, Summary>> {
        self.bots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// One conversation announced `seen`. A new main process changes the
    /// summary even when the state did not, because its generation did.
    pub(super) fn note(
        &self,
        bot: &BotId,
        conversation: &ConversationId,
        seen: Seen,
    ) -> Option<BotStateChanged> {
        let mut bots = self.lock();
        let summary = bots.entry(bot.clone()).or_default();
        summary.seen.insert(conversation.clone(), seen);
        summary.update(bot)
    }

    /// A conversation left the supervisor (archived).
    pub(super) fn forget(
        &self,
        bot: &BotId,
        conversation: &ConversationId,
    ) -> Option<BotStateChanged> {
        let mut bots = self.lock();
        let summary = bots.get_mut(bot)?;
        summary.seen.remove(conversation)?;
        summary.update(bot)
    }

    /// The bot was deleted.
    pub(super) fn forget_bot(&self, bot: &BotId) {
        self.lock().remove(bot);
    }
}

/// The summary of what a bot's conversations are doing; `None` when it has
/// none the supervisor knows.
pub(super) fn summarize(seen: impl Iterator<Item = Seen>) -> Option<(BotState, Option<u64>)> {
    let all: Vec<Seen> = seen.collect();
    if all.is_empty() {
        return None;
    }
    let main = all
        .iter()
        .find(|each| each.main)
        .map_or((BotState::Offline, None), |main| {
            (main.state, main.generation)
        });
    let any = |state: BotState| all.iter().any(|each| each.state == state);
    let other_awake = |state: BotState| all.iter().any(|each| !each.main && each.state == state);
    let state = if any(BotState::NeedsApproval) {
        BotState::NeedsApproval
    } else if any(BotState::Busy) {
        BotState::Busy
    } else if main.0 == BotState::Sleeping && other_awake(BotState::Idle) {
        BotState::Idle
    } else if main.0 == BotState::Sleeping && other_awake(BotState::Launching) {
        BotState::Launching
    } else {
        main.0
    };
    Some((state, main.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen(state: BotState, main: bool) -> Seen {
        Seen {
            state,
            generation: main.then_some(7),
            main,
        }
    }

    #[test]
    fn the_bot_shows_what_matters_most_among_its_conversations() {
        let sum = |list: &[Seen]| summarize(list.iter().copied()).map(|(state, _)| state);
        assert_eq!(sum(&[]), None);
        assert_eq!(sum(&[seen(BotState::Idle, true)]), Some(BotState::Idle));
        assert_eq!(
            sum(&[seen(BotState::Idle, true), seen(BotState::Busy, false)]),
            Some(BotState::Busy)
        );
        assert_eq!(
            sum(&[
                seen(BotState::Busy, true),
                seen(BotState::NeedsApproval, false)
            ]),
            Some(BotState::NeedsApproval)
        );
        assert_eq!(
            sum(&[
                seen(BotState::Sleeping, true),
                seen(BotState::Sleeping, false)
            ]),
            Some(BotState::Sleeping)
        );
        assert_eq!(
            sum(&[seen(BotState::Sleeping, true), seen(BotState::Idle, false)]),
            Some(BotState::Idle)
        );
        assert_eq!(
            sum(&[seen(BotState::AuthError, true), seen(BotState::Idle, false)]),
            Some(BotState::AuthError)
        );
    }

    #[test]
    fn the_app_hears_only_when_the_summary_changes() {
        let summaries = Summaries::default();
        let bot: BotId = "bot_01J9Z3K8000000000000000000".parse().unwrap();
        let main = ConversationId::main_of(&bot);
        let other = ConversationId::generate();
        let first = summaries.note(&bot, &main, seen(BotState::Idle, true));
        assert_eq!(first.map(|change| change.state), Some(BotState::Idle));
        assert!(
            summaries
                .note(&bot, &other, seen(BotState::Sleeping, false))
                .is_none()
        );
        let busy = summaries.note(&bot, &other, seen(BotState::Busy, false));
        assert_eq!(busy.map(|change| change.generation), Some(Some(7)));
        let back = summaries.forget(&bot, &other);
        assert_eq!(back.map(|change| change.state), Some(BotState::Idle));
    }
}
