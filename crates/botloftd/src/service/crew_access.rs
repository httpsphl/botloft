//! A bot reaching another crew (spec 10.4). Crews never see each other,
//! except where the owner allowed it: for good, in the `crew_access`
//! table, or for the bot's current turn only, kept here in memory until
//! the turn ends.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use botloft_core::ids::{BotId, CrewId};
use botloft_core::protocol::Crew;
use botloft_store::{BotRecord, Store};

use super::{ApiError, ApiResult};
use crate::state::Daemon;

/// What a bot may reach in another crew: the whole crew, or one bot of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reach {
    pub crew: CrewId,
    pub target: Option<BotId>,
}

impl Reach {
    fn covers(&self, crew: &CrewId, bot: Option<&BotId>) -> bool {
        self.crew == *crew
            && match (&self.target, bot) {
                (None, _) => true,
                (Some(target), Some(bot)) => target == bot,
                (Some(_), None) => false,
            }
    }
}

/// Access the owner gave "only now": it lasts until the bot's turn ends.
#[derive(Default)]
pub struct TurnAccess {
    turns: Mutex<HashMap<BotId, Vec<Reach>>>,
}

impl TurnAccess {
    pub fn allow(&self, bot: &BotId, reach: Reach) {
        let mut turns = self.turns.lock().unwrap_or_else(PoisonError::into_inner);
        let list = turns.entry(bot.clone()).or_default();
        if !list.contains(&reach) {
            list.push(reach);
        }
    }

    /// The bot's turn ended, or its process did: what it had for the turn
    /// goes.
    pub fn end_turn(&self, bot: &BotId) {
        self.turns
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(bot);
    }

    fn covers(&self, bot: &BotId, crew: &CrewId, target: Option<&BotId>) -> bool {
        self.turns
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(bot)
            .is_some_and(|list| list.iter().any(|reach| reach.covers(crew, target)))
    }
}

/// Whether `bot` may talk with `target` of `crew` (or, with `None`, see
/// the crew's roster), for good or for this turn.
pub(crate) fn reaches(
    daemon: &Daemon,
    store: &Store,
    bot: &BotId,
    crew: &CrewId,
    target: Option<&BotId>,
) -> ApiResult<bool> {
    if daemon.crew_access.covers(bot, crew, target) {
        return Ok(true);
    }
    Ok(store.crew_access(bot)?.iter().any(|access| {
        access.talk
            && Reach {
                crew: access.crew_id.clone(),
                target: access.target_bot_id.clone(),
            }
            .covers(crew, target)
    }))
}

/// How long a bot may answer a bot of another crew that wrote to it.
pub(crate) const ANSWER_WINDOW_MS: i64 = 24 * 60 * 60 * 1000;

/// Whether `from` may send to `to`, a bot of another crew: `from` was let
/// in, `to` was let in to `from`, or `to` wrote to `from` in the last day
/// and this is the answer. Bots talk in turns, so the answer usually comes
/// after the turn that was let in has ended.
pub(crate) fn may_talk(
    daemon: &Daemon,
    store: &Store,
    from: &BotRecord,
    to: &BotRecord,
) -> ApiResult<bool> {
    let since = daemon.clock.now_ms() - ANSWER_WINDOW_MS;
    Ok(reaches(daemon, store, &from.id, &to.crew_id, Some(&to.id))?
        || reaches(daemon, store, &to.id, &from.crew_id, Some(&from.id))?
        || store.wrote_to_since(&to.id, &from.id, since)?)
}

/// Another active crew, by its name or folder name, case aside. The
/// caller's own crew is not "another".
pub(crate) fn other_crew(store: &Store, own: &CrewId, name: &str) -> ApiResult<Crew> {
    let wanted = name.trim().to_lowercase();
    let found = store
        .crews(false)?
        .into_iter()
        .find(|crew| crew.name.to_lowercase() == wanted || crew.slug == wanted);
    match found {
        Some(crew) if crew.id == *own => Err(ApiError::validation(
            "that is your own crew: leave out `crew` to reach its bots",
        )),
        Some(crew) => Ok(crew),
        None => Err(ApiError::NotFound(format!(
            "there is no crew called \"{}\"; ask the owner for its exact name",
            name.trim()
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_crew_reach_covers_its_bots_and_a_bot_reach_only_that_bot() {
        let (crew, other) = (CrewId::generate(), CrewId::generate());
        let (writer, editor) = (BotId::generate(), BotId::generate());
        let whole = Reach {
            crew: crew.clone(),
            target: None,
        };
        assert!(whole.covers(&crew, Some(&writer)));
        assert!(whole.covers(&crew, None));
        assert!(!whole.covers(&other, Some(&writer)));
        let one = Reach {
            crew: crew.clone(),
            target: Some(writer.clone()),
        };
        assert!(one.covers(&crew, Some(&writer)));
        assert!(!one.covers(&crew, Some(&editor)));
        assert!(!one.covers(&crew, None));
    }

    #[test]
    fn access_for_the_turn_goes_when_it_ends() {
        let access = TurnAccess::default();
        let (bot, crew) = (BotId::generate(), CrewId::generate());
        access.allow(
            &bot,
            Reach {
                crew: crew.clone(),
                target: None,
            },
        );
        assert!(access.covers(&bot, &crew, None));
        assert!(!access.covers(&BotId::generate(), &crew, None));
        access.end_turn(&bot);
        assert!(!access.covers(&bot, &crew, None));
    }
}
