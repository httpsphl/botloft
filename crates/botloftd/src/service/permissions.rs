//! Everything the owner let each bot do, for the Permissions page in
//! Settings (spec 15.1): `permissions.list`, and taking a site of the
//! browser back, `sites.revoke` (spec 21.5).

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotSites, Permissions, SitesRevokeParams};
use tracing::info;

use super::{ApiError, ApiResult, crew_access};
use crate::state::{Daemon, Event};

/// Every bot's permissions, the archived bots' left out.
pub fn list(daemon: &Daemon) -> ApiResult<Permissions> {
    let store = daemon.store();
    let mut all = Permissions {
        desktop: Vec::new(),
        sites: Vec::new(),
        rules: Vec::new(),
        crew_access: Vec::new(),
    };
    for bot in store.bots(None, false)? {
        all.desktop.extend(store.desktop_grants(&bot.id)?);
        all.sites.extend(store.allowed_sites(&bot.id)?);
        all.rules.extend(store.allow_rules(&bot.id)?);
        all.crew_access
            .extend(crew_access::described(&store, &bot.id)?);
    }
    Ok(all)
}

/// Takes a site back: the bot asks for it again the next time.
pub fn revoke_site(daemon: &Daemon, params: SitesRevokeParams) -> ApiResult<BotSites> {
    super::bots::find(&daemon.store(), &params.bot_id)?;
    if !daemon
        .store()
        .revoke_browser_site(&params.bot_id, &params.site)?
    {
        return Err(ApiError::NotFound(format!("site {}", params.site)));
    }
    info!("the owner took a site back from a bot's browser");
    sites_changed(daemon, &params.bot_id)
}

/// Tells the apps the bot's sites now.
fn sites_changed(daemon: &Daemon, bot: &BotId) -> ApiResult<BotSites> {
    let sites = BotSites {
        sites: daemon.store().allowed_sites(bot)?,
        bot_id: bot.clone(),
    };
    daemon.emit(Event::BotSites(sites.clone()));
    Ok(sites)
}
