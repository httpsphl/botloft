//! `catalog.*` operations (spec 26.4).

use botloft_core::protocol::{
    Bot, BotTemplate, BotTemplateFull, CatalogAddParams, CatalogGetParams, CatalogListParams,
};
use tracing::info;

use super::bots::{self, NewBot};
use super::{ApiError, ApiResult, crews};
use crate::catalog;
use crate::state::Daemon;

pub fn list(params: CatalogListParams) -> ApiResult<Vec<BotTemplate>> {
    Ok(catalog::list(params.category))
}

pub fn get(params: CatalogGetParams) -> ApiResult<BotTemplateFull> {
    find(&params.id)
}

/// Creates a bot in the crew from a role. It is a plain bot: Manual mode, no
/// connected tools, no reach into other crews. The name and role come in the
/// owner's language; the instructions, model and effort come from the sheet.
pub fn add(daemon: &Daemon, params: CatalogAddParams) -> ApiResult<Bot> {
    let template = find(&params.template_id)?;
    let mut new = NewBot::check(
        &params.name,
        &params.role,
        &template.instructions,
        None,
        Some(template.model),
        Some(bots::default_agent(daemon)),
    )?;
    new.effort = template.effort;
    let store = daemon.store();
    let crew = crews::active(&store, &params.crew_id)?;
    let record = bots::insert(
        daemon,
        &store,
        &crew,
        botloft_core::ids::BotId::generate(),
        new,
    )?;
    let bot = bots::changed(daemon, &store, &crew, record);
    drop(store);
    daemon.supervisor.wake();
    info!(crew = %crew.id, bot = %bot.id, template = %template.id, "a bot was added from the catalog");
    Ok(bot)
}

fn find(id: &str) -> ApiResult<BotTemplateFull> {
    catalog::get(id).ok_or_else(|| ApiError::NotFound(format!("no bot template named {id}")))
}
