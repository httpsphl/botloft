//! `bots.*` operations.

use botloft_core::ids::{BotId, CrewId};
use botloft_core::protocol::{
    Bot, BotEffort, BotIdParams, BotModel, BotState, BotsCreateParams, BotsListParams,
    BotsRestartParams, BotsSetPausedParams, BotsUpdateParams, Crew, PermissionMode,
};
use botloft_core::{avatar, now_ms, slug, validate};
use botloft_store::{BotRecord, Store};

use super::{ApiError, ApiResult, crews, pick_slug};
use crate::paths::SHARED_DIR;
use crate::state::{Daemon, Event};
use crate::workspace;

pub fn list(daemon: &Daemon, params: BotsListParams) -> ApiResult<Vec<Bot>> {
    let store = daemon.store();
    let records = match &params.crew_id {
        Some(crew_id) => {
            crews::find(&store, crew_id)?;
            store.bots(Some(crew_id), false)?
        }
        None => store.bots(None, false)?,
    };
    records
        .into_iter()
        .map(|record| {
            let crew = crews::find(&store, &record.crew_id)?;
            Ok(to_protocol(daemon, &store, &crew, record))
        })
        .collect()
}

pub fn create(daemon: &Daemon, params: BotsCreateParams) -> ApiResult<Bot> {
    let new = NewBot::check(
        &params.name,
        &params.role,
        &params.instructions,
        params.color,
        params.model,
    )?;
    let store = daemon.store();
    let crew = crews::active(&store, &params.crew_id)?;
    let record = insert(daemon, &store, &crew, BotId::generate(), new)?;
    let bot = changed(daemon, &store, &crew, record);
    daemon.supervisor.wake();
    Ok(bot)
}

/// The fields of a new bot, checked.
pub(crate) struct NewBot {
    pub name: String,
    pub role: String,
    pub instructions: String,
    pub color: Option<String>,
    pub model: BotModel,
}

impl NewBot {
    pub(crate) fn check(
        name: &str,
        role: &str,
        instructions: &str,
        color: Option<String>,
        model: Option<BotModel>,
    ) -> ApiResult<Self> {
        Ok(Self {
            name: validate::name("name", name)?,
            role: validate::role(role)?,
            instructions: validate::instructions(instructions)?,
            color: color.as_deref().map(parse_color).transpose()?,
            model: model.unwrap_or(BotModel::Default),
        })
    }
}

/// Saves a bot in `crew` and writes its workspace, under the caller's lock
/// on the store; the caller announces it and wakes the supervisor. When
/// `crew.lead_bot_id` already names `id`, its rules say it leads.
pub(crate) fn insert(
    daemon: &Daemon,
    store: &Store,
    crew: &Crew,
    id: BotId,
    new: NewBot,
) -> ApiResult<BotRecord> {
    let handle = slug::slugify(&new.name, "bot");
    ensure_handle_free(store, &crew.id, &handle, None)?;
    let color = match new.color {
        Some(color) => color,
        None => avatar::palette_color(store.count_bots(&crew.id)?).to_owned(),
    };
    let slug = pick_slug(&handle, |candidate| {
        Ok(candidate == SHARED_DIR
            || store.bot_slug_exists(&crew.id, candidate)?
            || daemon.paths.bot_workspace(&crew.slug, candidate).exists())
    })?;

    let record = BotRecord {
        id,
        crew_id: crew.id.clone(),
        name: new.name,
        handle,
        slug,
        role: new.role,
        instructions: new.instructions,
        color,
        paused: false,
        permission_mode: PermissionMode::Default,
        model: new.model,
        model_in_use: None,
        effort: BotEffort::Default,
        effort_default: None,
        created_at: now_ms(),
        archived_at: None,
    };
    workspace::prepare_bot(daemon.workspace_env(), crew, &record).map_err(ApiError::Workspace)?;
    store.insert_bot(&record)?;
    Ok(record)
}

pub fn update(daemon: &Daemon, params: BotsUpdateParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, mut record) = active(&store, &params.bot_id)?;
    if let Some(name) = params.name {
        record.name = validate::name("name", &name)?;
        record.handle = slug::slugify(&record.name, "bot");
        ensure_handle_free(&store, &crew.id, &record.handle, Some(&record.id))?;
    }
    if let Some(role) = params.role {
        record.role = validate::role(&role)?;
    }
    if let Some(instructions) = params.instructions {
        record.instructions = validate::instructions(&instructions)?;
    }
    if let Some(color) = params.color {
        record.color = parse_color(&color)?;
    }
    // Rules first: if the disk write fails, nothing is saved. A running bot
    // reads them at its next start.
    workspace::write_rules(&daemon.paths, &crew, &record).map_err(ApiError::Workspace)?;
    store.update_bot(&record)?;
    Ok(changed(daemon, &store, &crew, record))
}

pub fn set_paused(daemon: &Daemon, params: BotsSetPausedParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, mut record) = active(&store, &params.bot_id)?;
    if record.paused == params.paused {
        return Ok(to_protocol(daemon, &store, &crew, record));
    }
    record.paused = params.paused;
    store.update_bot(&record)?;
    let bot = changed(daemon, &store, &crew, record);
    daemon.supervisor.wake();
    if bot.paused {
        daemon.browsers.close(&bot.id);
    }
    Ok(bot)
}

/// Restarts the process; `fresh` drops the conversation. Also the way out
/// of `auth_error` (spec 7.3).
pub fn restart(daemon: &Daemon, params: BotsRestartParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, record) = active(&store, &params.bot_id)?;
    if record.paused || crew.paused {
        return Err(ApiError::Conflict(format!(
            "bot {} is paused; resume it instead",
            record.id
        )));
    }
    daemon
        .supervisor
        .restart(&record.id, params.fresh.unwrap_or(false));
    Ok(to_protocol(daemon, &store, &crew, record))
}

/// Archives the bot and stops its process. Archiving twice is not an error.
/// The workspace stays on disk for now (spec 14).
pub fn archive(daemon: &Daemon, params: BotIdParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let mut record = find(&store, &params.bot_id)?;
    let crew = crews::find(&store, &record.crew_id)?;
    if record.archived_at.is_some() {
        return Ok(to_protocol(daemon, &store, &crew, record));
    }
    record.archived_at = Some(now_ms());
    store.update_bot(&record)?;
    super::lead::forget_archived(daemon, &store, &crew, &record.id)?;
    super::routines::archive_of(daemon, &store, &record.id);
    let bot = changed(daemon, &store, &crew, record);
    daemon.supervisor.wake();
    daemon.browsers.forget(&bot.id);
    Ok(bot)
}

/// The protocol view of a stored bot, with the supervisor's live state.
pub(crate) fn to_protocol(daemon: &Daemon, store: &Store, crew: &Crew, record: BotRecord) -> Bot {
    let (live, generation) = daemon
        .supervisor
        .status(&record.id)
        .unwrap_or((BotState::Offline, None));
    let state = if record.archived_at.is_some() || crew.archived_at.is_some() {
        BotState::Archived
    } else {
        live
    };
    let workspace = daemon.paths.bot_workspace(&crew.slug, &record.slug);
    let last_activity = store.last_activity(&record.id).unwrap_or_else(|err| {
        tracing::warn!(bot = %record.id, "could not read the last activity: {err}");
        None
    });
    let context = daemon.contexts.get(&record.id);
    Bot {
        id: record.id,
        crew_id: record.crew_id,
        name: record.name,
        handle: record.handle,
        slug: record.slug,
        role: record.role,
        instructions: record.instructions,
        color: record.color,
        paused: record.paused,
        permission_mode: record.permission_mode,
        model: record.model,
        model_in_use: record.model_in_use,
        effort: record.effort,
        effort_default: record.effort_default,
        context,
        state,
        generation,
        workspace: workspace.to_string_lossy().into_owned(),
        last_activity,
        created_at: record.created_at,
        archived_at: record.archived_at,
    }
}

pub(crate) fn changed(daemon: &Daemon, store: &Store, crew: &Crew, record: BotRecord) -> Bot {
    let bot = to_protocol(daemon, store, crew, record);
    daemon.emit(Event::BotChanged(bot.clone()));
    bot
}

pub(crate) fn find(store: &Store, id: &BotId) -> ApiResult<BotRecord> {
    store
        .bot(id)?
        .ok_or_else(|| ApiError::NotFound(format!("bot {id} does not exist")))
}

/// The bot and its crew, if the bot exists and is not archived.
pub(crate) fn active(store: &Store, id: &BotId) -> ApiResult<(Crew, BotRecord)> {
    let record = find(store, id)?;
    if record.archived_at.is_some() {
        return Err(ApiError::Conflict(format!("bot {id} is archived")));
    }
    let crew = crews::active(store, &record.crew_id)?;
    Ok((crew, record))
}

pub(crate) fn ensure_handle_free(
    store: &Store,
    crew: &CrewId,
    handle: &str,
    except: Option<&BotId>,
) -> ApiResult<()> {
    match store.active_bot_by_handle(crew, handle)? {
        Some(owner) if Some(&owner) != except => Err(ApiError::validation(format!(
            "another bot in this crew already answers to @{handle}; pick a different name"
        ))),
        _ => Ok(()),
    }
}

fn parse_color(input: &str) -> ApiResult<String> {
    avatar::normalize_color(input)
        .ok_or_else(|| ApiError::validation("color must be a hex color like #5EC8FF"))
}
