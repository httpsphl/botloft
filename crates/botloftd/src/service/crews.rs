//! `crews.*` operations.

use botloft_core::ids::CrewId;
use botloft_core::protocol::{
    Crew, CrewIdParams, CrewsCreateParams, CrewsRenameParams, CrewsSetPausedParams,
};
use botloft_core::{now_ms, slug, validate};
use botloft_store::Store;
use tracing::warn;

use super::{ApiError, ApiResult, bots, pick_slug};
use crate::state::{Daemon, Event};
use crate::workspace;

pub fn list(daemon: &Daemon) -> ApiResult<Vec<Crew>> {
    Ok(daemon.store().crews(false)?)
}

pub fn create(daemon: &Daemon, params: CrewsCreateParams) -> ApiResult<Crew> {
    let name = validate::name("name", &params.name)?;
    let store = daemon.store();
    // A folder left on disk by something else is never adopted.
    let slug = pick_slug(&slug::slugify(&name, "crew"), |candidate| {
        Ok(store.crew_slug_exists(candidate)? || daemon.paths.crew_dir(candidate).exists())
    })?;
    workspace::prepare_crew(&daemon.paths, &slug).map_err(ApiError::Workspace)?;

    let crew = Crew {
        id: CrewId::generate(),
        name,
        slug,
        paused: false,
        created_at: now_ms(),
        archived_at: None,
    };
    store.insert_crew(&crew)?;
    daemon.emit(Event::CrewChanged(crew.clone()));
    Ok(crew)
}

pub fn rename(daemon: &Daemon, params: CrewsRenameParams) -> ApiResult<Crew> {
    let name = validate::name("name", &params.name)?;
    let store = daemon.store();
    let mut crew = active(&store, &params.crew_id)?;
    crew.name = name;
    store.update_crew(&crew)?;

    // The crew name appears in every bot's rules file.
    for bot in store.bots(Some(&crew.id), false)? {
        if let Err(err) = workspace::write_rules(&daemon.paths, &crew, &bot) {
            warn!(bot = %bot.id, "could not refresh the bot rules after a crew rename: {err}");
        }
    }
    daemon.emit(Event::CrewChanged(crew.clone()));
    Ok(crew)
}

pub fn set_paused(daemon: &Daemon, params: CrewsSetPausedParams) -> ApiResult<Crew> {
    let store = daemon.store();
    let mut crew = active(&store, &params.crew_id)?;
    if crew.paused != params.paused {
        crew.paused = params.paused;
        store.update_crew(&crew)?;
        daemon.emit(Event::CrewChanged(crew.clone()));
        daemon.supervisor.wake();
    }
    Ok(crew)
}

/// Archives the crew and its bots. Archiving twice is not an error.
pub fn archive(daemon: &Daemon, params: CrewIdParams) -> ApiResult<Crew> {
    let store = daemon.store();
    let crew = find(&store, &params.crew_id)?;
    if crew.archived_at.is_some() {
        return Ok(crew);
    }
    let bots = store.bots(Some(&crew.id), false)?;
    store.archive_crew(&crew.id, now_ms())?;

    let crew = find(&store, &crew.id)?;
    for bot in bots {
        if let Some(bot) = store.bot(&bot.id)? {
            daemon.emit(Event::BotChanged(bots::to_protocol(
                daemon, &store, &crew, bot,
            )));
        }
    }
    daemon.emit(Event::CrewChanged(crew.clone()));
    daemon.supervisor.wake();
    Ok(crew)
}

pub(crate) fn find(store: &Store, id: &CrewId) -> ApiResult<Crew> {
    store
        .crew(id)?
        .ok_or_else(|| ApiError::NotFound(format!("crew {id} does not exist")))
}

/// The crew, if it exists and is not archived.
pub(crate) fn active(store: &Store, id: &CrewId) -> ApiResult<Crew> {
    let crew = find(store, id)?;
    if crew.archived_at.is_some() {
        return Err(ApiError::Conflict(format!("crew {id} is archived")));
    }
    Ok(crew)
}
