//! `archive.list`: what the owner archived, for the app to show and delete
//! (spec 7.6). The other lists leave archived crews and bots out.

use std::cmp::Reverse;
use std::collections::HashMap;

use botloft_core::protocol::Archive;

use super::{ApiResult, bots, crews};
use crate::state::Daemon;

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
