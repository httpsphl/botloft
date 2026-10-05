//! A crew's chief (spec 10.2): created with the crew, moved by the owner,
//! and the only bot that may suggest new ones.

use botloft_core::ids::BotId;
use botloft_core::protocol::{Bot, Crew, CrewsSetLeadParams};
use botloft_store::{BotRecord, Store};
use tracing::{info, warn};

use super::bots::{self, NewBot};
use super::{ApiError, ApiResult, crews};
use crate::state::Daemon;

pub(crate) fn is_lead(crew: &Crew, bot: &BotId) -> bool {
    crew.lead_bot_id.as_ref() == Some(bot)
}

/// Saves the chief of a crew being created, under the caller's lock, so its
/// first start already reads that it leads.
pub(crate) fn create_chief(
    daemon: &Daemon,
    store: &Store,
    crew: &mut Crew,
    lead: NewBot,
) -> ApiResult<BotRecord> {
    let id = BotId::generate();
    crew.lead_bot_id = Some(id.clone());
    let record = bots::insert(daemon, store, crew, id, lead)?;
    store.update_crew(crew)?;
    Ok(record)
}

/// Makes a bot of the crew its chief, or leaves the crew without one. The
/// old and the new chief restart to read their rules once idle.
pub fn set_lead(daemon: &Daemon, params: CrewsSetLeadParams) -> ApiResult<Crew> {
    let store = daemon.store();
    let mut crew = crews::active(&store, &params.crew_id)?;
    if let Some(bot) = &params.bot_id {
        let (bot_crew, _) = bots::active(&store, bot)?;
        if bot_crew.id != crew.id {
            return Err(ApiError::validation(format!(
                "bot {bot} is not in crew {}",
                crew.id
            )));
        }
    }
    if crew.lead_bot_id == params.bot_id {
        return Ok(crews::present(daemon, crew));
    }
    let old = std::mem::replace(&mut crew.lead_bot_id, params.bot_id.clone());
    store.update_crew(&crew)?;

    let touched: Vec<BotId> = old.into_iter().chain(params.bot_id).collect();
    for id in &touched {
        match store.bot(id) {
            Ok(Some(record)) if record.archived_at.is_none() => {
                if let Err(err) = super::mcp::write_rules(daemon, &store, &crew, &record) {
                    warn!(bot = %id, "could not refresh the rules of a chief: {err}");
                }
            }
            Ok(_) => {}
            Err(err) => warn!(bot = %id, "could not read a chief: {err}"),
        }
    }
    drop(store);
    for id in &touched {
        daemon.supervisor.launch_settings_changed(id);
    }
    info!(crew = %crew.id, "chief changed");
    Ok(crews::changed(daemon, crew))
}

/// An archived chief leaves its crew without one.
pub(crate) fn forget_archived(
    daemon: &Daemon,
    store: &Store,
    crew: &Crew,
    bot: &BotId,
) -> ApiResult<()> {
    if !is_lead(crew, bot) {
        return Ok(());
    }
    let mut crew = crew.clone();
    crew.lead_bot_id = None;
    store.update_crew(&crew)?;
    crews::changed(daemon, crew);
    Ok(())
}

/// Whether the chief `lead` may add `new` to its crew now: it still leads,
/// the name is free and the crew has room. Checked before asking the owner
/// and again when creating.
pub(crate) fn check_suggestion(daemon: &Daemon, lead: &BotId, new: &NewBot) -> ApiResult<()> {
    let store = daemon.store();
    room_for(daemon, &store, lead, new).map(drop)
}

/// Creates a bot the chief suggested and the owner allowed (spec 10.2).
pub(crate) fn create_suggested(daemon: &Daemon, lead: &BotId, new: NewBot) -> ApiResult<Bot> {
    let store = daemon.store();
    let crew = room_for(daemon, &store, lead, &new)?;
    let record = bots::insert(daemon, &store, &crew, BotId::generate(), new)?;
    let bot = bots::changed(daemon, &store, &crew, record);
    drop(store);
    daemon.supervisor.wake();
    info!(crew = %crew.id, bot = %bot.id, "the chief's suggested bot was created");
    Ok(bot)
}

fn room_for(daemon: &Daemon, store: &Store, lead: &BotId, new: &NewBot) -> ApiResult<Crew> {
    let (crew, _) = bots::active(store, lead)?;
    if !is_lead(&crew, lead) {
        return Err(ApiError::Conflict(
            "only the crew's chief can suggest new bots; ask the chief with send_message".into(),
        ));
    }
    let size = store.bots(Some(&crew.id), false)?.len();
    let max = daemon.bots.max_per_crew;
    if size >= max {
        return Err(ApiError::Conflict(format!(
            "the crew already has {size} bots, the most a chief can reach ({max}); \
             use the bots you have, or ask the owner to archive one"
        )));
    }
    bots::ensure_handle_free(
        store,
        &crew.id,
        &botloft_core::slug::slugify(&new.name, "bot"),
        None,
    )?;
    Ok(crew)
}
