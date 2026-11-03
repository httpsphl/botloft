//! The Maker (spec 36.2): an agent Botloft keeps in a crew of its own, above
//! the owner's crews. It is always there: it cannot be archived, deleted or
//! made to bypass permissions, and no other bot joins its crew.

use botloft_core::ids::{BotId, CrewId};
use botloft_core::protocol::{AgentKind, BotEffort, BotModel, Crew, MAKER_CREW, PermissionMode};
use botloft_core::{now_ms, slug};
use botloft_store::{BotRecord, Store};
use tracing::info;

use super::bots::{self, NewBot};
use super::{ApiError, ApiResult, pick_slug};
use crate::state::Daemon;
use crate::workspace;

/// The crew's name, which the app never shows.
const NAME: &str = "Maker";
/// Its handle, the same in every language (spec 36.13).
const HANDLE: &str = "maker";
/// White: the app draws every mascot in this color as the Maker's, in the
/// chat, the island and everywhere else (spec 36.3). No other bot takes it.
pub(crate) const COLOR: &str = "#FFFFFF";

/// Its name and role in the language the owner reads the app in.
fn words(locale: &str) -> (&'static str, &'static str) {
    if locale.starts_with("pt") {
        ("O Criador", "Monta e ajusta as suas equipes")
    } else if locale.starts_with("es") {
        ("El Creador", "Arma y ajusta tus equipos")
    } else {
        ("The Maker", "Builds and tunes your crews")
    }
}

pub(crate) fn is_maker_crew(crew: &Crew) -> bool {
    crew.system.as_deref() == Some(MAKER_CREW)
}

/// Whether `bot` is the Maker.
pub(crate) fn is_maker(store: &Store, bot: &BotId) -> ApiResult<bool> {
    let (crew, _) = bots::active(store, bot)?;
    Ok(is_maker_crew(&crew))
}

/// Refuses what would change the Maker's crew the way only the owner's own
/// crews change.
pub(crate) fn refuse(crew: &Crew, what: &str) -> ApiResult<()> {
    if is_maker_crew(crew) {
        return Err(ApiError::Conflict(format!("the Maker {what}")));
    }
    Ok(())
}

/// The Maker never bypasses permissions: one build makes a lot at once.
pub(crate) fn refuse_mode(crew: &Crew, mode: PermissionMode) -> ApiResult<()> {
    if mode == PermissionMode::BypassPermissions {
        refuse(crew, "always asks before it builds")?;
    }
    Ok(())
}

/// Makes the Maker's crew and the Maker when they are missing, once at
/// start, before the supervisor looks at the bots. Returns the Maker.
pub fn ensure(daemon: &Daemon) -> ApiResult<BotId> {
    let store = daemon.store();
    let crew = match store.crews(true)?.into_iter().find(is_maker_crew) {
        Some(crew) => crew,
        None => create_crew(daemon, &store)?,
    };
    if let Some(mut bot) = store.bots(Some(&crew.id), true)?.into_iter().next() {
        // A Maker made before it turned white turns white now.
        if bot.color != COLOR {
            bot.color = COLOR.to_owned();
            store.update_bot(&bot)?;
            info!(bot = %bot.id, "the Maker turned white");
        }
        return Ok(bot.id);
    }
    // Named as its handle first, so the handle is `maker` in every language.
    let (_, role) = words(&daemon.desktop.locale());
    let mut new = NewBot::check(
        HANDLE,
        role,
        "",
        Some(COLOR.to_owned()),
        Some(BotModel::Opus),
        None,
    )?;
    new.agent = AgentKind::Claude;
    new.effort = BotEffort::High;
    let record: BotRecord = bots::insert(daemon, &store, &crew, BotId::generate(), new)?;
    info!(crew = %crew.id, bot = %record.id, "the Maker was created");
    drop(store);
    localize(daemon, &daemon.desktop.locale());
    Ok(record.id)
}

/// Gives the Maker its name and role in the app's language, keeping the
/// handle: called when the app says which language it shows.
pub fn localize(daemon: &Daemon, locale: &str) {
    let (name, role) = words(locale);
    let store = daemon.store();
    let found = store.crews(true).ok().and_then(|crews| {
        let crew = crews.into_iter().find(is_maker_crew)?;
        let bot = store.bots(Some(&crew.id), true).ok()?.into_iter().next()?;
        Some((crew, bot))
    });
    let Some((crew, mut record)) = found else {
        return;
    };
    if record.name == name && record.role == role {
        return;
    }
    record.name = name.to_owned();
    record.role = role.to_owned();
    if let Err(err) = store.update_bot(&record) {
        tracing::warn!("could not rename the Maker: {err}");
        return;
    }
    if let Err(err) = super::mcp::write_rules(daemon, &store, &crew, &record) {
        tracing::warn!("could not refresh the Maker's rules: {err}");
    }
    bots::changed(daemon, &store, &crew, record);
}

fn create_crew(daemon: &Daemon, store: &Store) -> ApiResult<Crew> {
    let slug = pick_slug(&slug::slugify(MAKER_CREW, "crew"), |candidate| {
        Ok(store.crew_slug_exists(candidate)? || daemon.paths.crew_dir(candidate).exists())
    })?;
    let crew = Crew {
        id: CrewId::generate(),
        name: NAME.to_owned(),
        slug,
        work_folder: String::new(),
        work_folder_chosen: false,
        lead_bot_id: None,
        paused: false,
        auto_mode: false,
        color: None,
        system: Some(MAKER_CREW.to_owned()),
        created_at: now_ms(),
        archived_at: None,
    };
    workspace::prepare_crew(&daemon.paths, &crew).map_err(ApiError::Workspace)?;
    store.insert_crew(&crew)?;
    Ok(crew)
}
