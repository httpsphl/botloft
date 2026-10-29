//! `archive.list`, `crews.unarchive` and `bots.unarchive`: what the owner
//! archived, for the app to show, bring back or delete (spec 7.6). The
//! other lists leave archived crews and bots out.

use std::cmp::Reverse;
use std::collections::HashMap;

use botloft_core::protocol::{Archive, Bot, BotIdParams, Crew, CrewIdParams};

use super::{ApiError, ApiResult, bots, crews, routines};
use crate::state::{Daemon, Event};

/// Archived crews and bots, the most recently archived first. A bot of an
/// archived crew is listed too, so the app can count them.
pub fn list(daemon: &Daemon) -> ApiResult<Archive> {
    let store = daemon.store();
    let crews: HashMap<_, _> = store
        .crews(true)?
        .into_iter()
        .map(|crew| (crew.id.clone(), crews::present(daemon, crew)))
        .collect();
    let mut bots: Vec<_> = store
        .bots(None, true)?
        .into_iter()
        .filter(|bot| bot.archived_at.is_some())
        .filter_map(|bot| {
            let crew = crews.get(&bot.crew_id)?;
            Some(bots::to_protocol(daemon, &store, crew, bot))
        })
        .collect();
    // Ids grow with time, so they settle what was archived together.
    bots.sort_by_key(|bot| Reverse((bot.archived_at, bot.id.clone())));
    let mut crews: Vec<_> = crews
        .into_values()
        .filter(|crew| crew.archived_at.is_some())
        .collect();
    crews.sort_by_key(|crew| Reverse((crew.archived_at, crew.id.clone())));
    Ok(Archive { crews, bots })
}

/// Brings back the crew with the bots and routines archived with it. Its
/// bots start again, each with a new token. Unarchiving twice is not an
/// error.
pub fn unarchive_crew(daemon: &Daemon, params: CrewIdParams) -> ApiResult<Crew> {
    let store = daemon.store();
    let crew = crews::find(&store, &params.crew_id)?;
    let Some(at) = crew.archived_at else {
        return Ok(crews::present(daemon, crew));
    };
    let back: Vec<_> = store
        .bots(Some(&crew.id), true)?
        .into_iter()
        .filter(|bot| bot.archived_at == Some(at))
        .collect();
    store.unarchive_crew(&crew.id, at)?;

    let crew = crews::find(&store, &crew.id)?;
    for bot in back {
        routines::unarchive_of(daemon, &store, &bot.id, at);
        if let Some(bot) = store.bot(&bot.id)? {
            daemon.emit(Event::BotChanged(bots::to_protocol(
                daemon, &store, &crew, bot,
            )));
        }
    }
    let crew = crews::changed(daemon, crew);
    daemon.supervisor.wake();
    tracing::info!(crew = %crew.id, "crew unarchived");
    Ok(crew)
}

/// Brings back a bot archived on its own, with the routines archived with
/// it. A bot of an archived crew comes back with its crew, and one whose
/// name another bot of the crew took meanwhile cannot come back.
pub fn unarchive_bot(daemon: &Daemon, params: BotIdParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let mut record = bots::find(&store, &params.bot_id)?;
    let crew = crews::find(&store, &record.crew_id)?;
    let Some(at) = record.archived_at else {
        return Ok(bots::to_protocol(daemon, &store, &crew, record));
    };
    if crew.archived_at.is_some() {
        return Err(ApiError::Conflict(format!(
            "crew {} is archived; unarchive the crew to bring its bots back",
            crew.id
        )));
    }
    if store
        .active_bot_by_handle(&crew.id, &record.handle)?
        .is_some()
    {
        return Err(ApiError::Conflict(format!(
            "another bot in this crew already answers to @{}; rename or archive it first",
            record.handle
        )));
    }
    record.archived_at = None;
    store.update_bot(&record)?;
    routines::unarchive_of(daemon, &store, &record.id, at);
    let bot = bots::changed(daemon, &store, &crew, record);
    daemon.supervisor.wake();
    tracing::info!(bot = %bot.id, "bot unarchived");
    Ok(bot)
}
