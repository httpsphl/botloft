//! `crews.*` operations.

use botloft_core::avatar::normalize_color;
use botloft_core::ids::CrewId;
use botloft_core::protocol::{
    Crew, CrewIdParams, CrewsCreateParams, CrewsRenameParams, CrewsSetColorParams,
    CrewsSetPausedParams, CrewsSetWorkFolderParams,
};
use botloft_core::{now_ms, slug, validate};
use botloft_store::Store;
use tracing::warn;

use super::bots::NewBot;
use super::{ApiError, ApiResult, bots, pick_slug};
use crate::state::{Daemon, Event};
use crate::workspace;

pub fn list(daemon: &Daemon) -> ApiResult<Vec<Crew>> {
    let crews = daemon.store().crews(false)?;
    Ok(crews
        .into_iter()
        .map(|crew| present(daemon, crew))
        .collect())
}

pub fn create(daemon: &Daemon, params: CrewsCreateParams) -> ApiResult<Crew> {
    let name = validate::name("name", &params.name)?;
    let store = daemon.store();
    // A folder left on disk by something else is never adopted.
    let slug = pick_slug(&slug::slugify(&name, "crew"), |candidate| {
        Ok(store.crew_slug_exists(candidate)? || daemon.paths.crew_dir(candidate).exists())
    })?;
    let chosen = params
        .work_folder
        .as_deref()
        .map(|input| choose_folder(daemon, &store, &slug, None, input))
        .transpose()?;

    let lead = params
        .lead
        .map(|lead| NewBot::check(&lead.name, &lead.role, &lead.instructions, None, lead.model))
        .transpose()?;

    let mut crew = Crew {
        id: CrewId::generate(),
        name,
        slug,
        work_folder_chosen: chosen.is_some(),
        work_folder: chosen.unwrap_or_default(),
        lead_bot_id: None,
        paused: false,
        color: None,
        created_at: now_ms(),
        archived_at: None,
    };
    workspace::prepare_crew(&daemon.paths, &crew).map_err(ApiError::Workspace)?;
    store.insert_crew(&crew)?;
    // The chief is saved before the store is let go, so it starts as one.
    let chief = match lead {
        Some(lead) => Some(super::lead::create_chief(daemon, &store, &mut crew, lead)?),
        None => None,
    };
    let crew = changed(daemon, crew);
    if let Some(chief) = chief {
        bots::changed(daemon, &store, &crew, chief);
        daemon.supervisor.wake();
    }
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
        if let Err(err) = super::mcp::write_rules(daemon, &store, &crew, &bot) {
            warn!(bot = %bot.id, "could not refresh the bot rules after a crew rename: {err}");
        }
    }
    Ok(changed(daemon, crew))
}

/// Gives the crew a color, or clears it.
pub fn set_color(daemon: &Daemon, params: CrewsSetColorParams) -> ApiResult<Crew> {
    let color = match params.color.as_deref() {
        None => None,
        Some(input) => Some(
            normalize_color(input)
                .ok_or_else(|| ApiError::validation("color must look like #RRGGBB"))?,
        ),
    };
    let store = daemon.store();
    let mut crew = active(&store, &params.crew_id)?;
    crew.color = color;
    store.update_crew(&crew)?;
    Ok(changed(daemon, crew))
}

/// Moves the crew to another work folder, or back to `shared`. Each bot
/// restarts to reach it once nothing is in progress (spec 7.4).
pub fn set_work_folder(daemon: &Daemon, params: CrewsSetWorkFolderParams) -> ApiResult<Crew> {
    let store = daemon.store();
    let mut crew = active(&store, &params.crew_id)?;
    let chosen = params
        .work_folder
        .as_deref()
        .map(|input| choose_folder(daemon, &store, &crew.slug, Some(&crew.id), input))
        .transpose()?;
    if crew.work_folder_chosen == chosen.is_some()
        && chosen
            .as_deref()
            .is_none_or(|path| path == crew.work_folder)
    {
        return Ok(present(daemon, crew));
    }
    crew.work_folder_chosen = chosen.is_some();
    crew.work_folder = chosen.unwrap_or_default();
    workspace::prepare_crew(&daemon.paths, &crew).map_err(ApiError::Workspace)?;
    store.update_crew(&crew)?;

    let bots = store.bots(Some(&crew.id), false)?;
    drop(store);
    for bot in &bots {
        if let Err(err) = super::mcp::write_rules(daemon, &daemon.store(), &crew, bot) {
            warn!(bot = %bot.id, "could not refresh the bot rules after a folder change: {err}");
        }
        daemon.supervisor.launch_settings_changed(&bot.id);
    }
    tracing::info!(crew = %crew.id, "work folder changed");
    Ok(changed(daemon, crew))
}

pub fn set_paused(daemon: &Daemon, params: CrewsSetPausedParams) -> ApiResult<Crew> {
    let store = daemon.store();
    let mut crew = active(&store, &params.crew_id)?;
    if crew.paused != params.paused {
        crew.paused = params.paused;
        store.update_crew(&crew)?;
        if crew.paused {
            for bot in store.bots(Some(&crew.id), false)? {
                daemon.browsers.close(&bot.id);
            }
        }
        let crew = changed(daemon, crew);
        daemon.supervisor.wake();
        return Ok(crew);
    }
    Ok(present(daemon, crew))
}

/// Archives the crew and its bots. Archiving twice is not an error.
pub fn archive(daemon: &Daemon, params: CrewIdParams) -> ApiResult<Crew> {
    let store = daemon.store();
    let crew = find(&store, &params.crew_id)?;
    if crew.archived_at.is_some() {
        return Ok(present(daemon, crew));
    }
    let bots = store.bots(Some(&crew.id), false)?;
    store.archive_crew(&crew.id, now_ms())?;

    let crew = find(&store, &crew.id)?;
    for bot in &bots {
        super::routines::archive_of(daemon, &store, &bot.id);
        daemon.browsers.forget(&bot.id);
    }
    for bot in bots {
        if let Some(bot) = store.bot(&bot.id)? {
            daemon.emit(Event::BotChanged(bots::to_protocol(
                daemon, &store, &crew, bot,
            )));
        }
    }
    let crew = changed(daemon, crew);
    daemon.supervisor.wake();
    Ok(crew)
}

/// The crew as the app sees it: with the path of its `shared` folder when
/// the owner chose none.
pub(crate) fn present(daemon: &Daemon, mut crew: Crew) -> Crew {
    if !crew.work_folder_chosen {
        crew.work_folder = daemon.paths.work_folder(&crew).display().to_string();
    }
    crew
}

pub(crate) fn changed(daemon: &Daemon, crew: Crew) -> Crew {
    let crew = present(daemon, crew);
    daemon.emit(Event::CrewChanged(crew.clone()));
    crew
}

/// A work folder of the crew `slug` (`id` once it exists), apart from every
/// other crew's folders.
fn choose_folder(
    daemon: &Daemon,
    store: &Store,
    slug: &str,
    id: Option<&CrewId>,
    input: &str,
) -> ApiResult<String> {
    let others: Vec<Crew> = store
        .crews(true)?
        .into_iter()
        .filter(|crew| Some(&crew.id) != id)
        .collect();
    let path = workspace::folder::choose(&daemon.paths, slug, &others, input)
        .map_err(ApiError::validation)?;
    Ok(path.display().to_string())
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
