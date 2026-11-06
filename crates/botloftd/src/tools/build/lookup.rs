//! Finding what a step on an existing crew names (spec 36.8): the crew by
//! its name, a bot by its handle or `chief`, a routine by its name.

use std::collections::HashSet;

use botloft_core::protocol::{Crew, Routine, RoutinesListParams};
use botloft_core::slug;
use botloft_store::BotRecord;

use crate::service::{bots, maker, routines};
use crate::state::Daemon;
use crate::tools::calls::explain;

/// An active crew of the owner by its name; never the Maker's.
pub(crate) fn crew_named(daemon: &Daemon, name: &str) -> Result<Crew, String> {
    let wanted = name.trim().to_lowercase();
    let crews = daemon
        .store()
        .crews(false)
        .map_err(|err| explain(err.into()))?;
    let crew = crews
        .into_iter()
        .find(|crew| crew.name.trim().to_lowercase() == wanted)
        .ok_or_else(|| {
            format!(
                "no crew of the plan has the ref \"{name}\" and no crew is called that; \
                 crews_overview lists the owner's crews"
            )
        })?;
    if maker::is_maker_crew(&crew) {
        return Err("your own crew cannot be changed".to_owned());
    }
    Ok(crew)
}

/// The handles of the crew's active bots, for the plan's checks.
pub(crate) fn handles(daemon: &Daemon, name: &str) -> Result<HashSet<String>, String> {
    let crew = crew_named(daemon, name)?;
    let bots = daemon
        .store()
        .bots(Some(&crew.id), false)
        .map_err(|err| explain(err.into()))?;
    Ok(bots.into_iter().map(|bot| bot.handle).collect())
}

/// A bot of the crew by its handle (or name), or `chief`.
pub(crate) fn agent_of(daemon: &Daemon, crew: &Crew, agent: &str) -> Result<BotRecord, String> {
    let store = daemon.store();
    let id = if agent.trim().eq_ignore_ascii_case("chief") {
        crew.lead_bot_id.clone()
    } else {
        let handle = slug::slugify(agent.trim().trim_start_matches('@'), "");
        match handle.as_str() {
            "" => None,
            handle => store
                .active_bot_by_handle(&crew.id, handle)
                .map_err(|err| explain(err.into()))?,
        }
    };
    let id = id.ok_or_else(|| {
        format!(
            "no bot of the crew {} answers to {agent}; crews_overview lists its bots",
            crew.name
        )
    })?;
    bots::find(&store, &id).map_err(explain)
}

/// A routine of the crew's active bots by its name or id.
pub(crate) fn routine_of(
    daemon: &Daemon,
    crew: &Crew,
    agent: Option<&str>,
    wanted: &str,
) -> Result<Routine, String> {
    let owners = match agent {
        Some(agent) => vec![agent_of(daemon, crew, agent)?],
        None => daemon
            .store()
            .bots(Some(&crew.id), false)
            .map_err(|err| explain(err.into()))?,
    };
    let key = wanted.trim().to_lowercase();
    let mut found = Vec::new();
    for owner in owners {
        let list = routines::list(
            daemon,
            RoutinesListParams {
                bot_id: Some(owner.id.clone()),
            },
        )
        .map_err(explain)?;
        found.extend(list.into_iter().filter(|routine| {
            routine.id.as_str() == wanted.trim() || routine.name.trim().to_lowercase() == key
        }));
    }
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(format!(
            "the crew {} has no routine called {wanted}; crews_overview lists them",
            crew.name
        )),
        _ => Err(format!(
            "two routines of the crew {} are called {wanted}: give `agent`",
            crew.name
        )),
    }
}
