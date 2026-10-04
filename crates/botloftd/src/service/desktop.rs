//! What each bot may see and do on the owner's desktop (spec 24.2,
//! 24.10): `desktop.grants` and `desktop.revoke`. Grants come from the
//! owner answering a bot's request (`tools/desktop.rs`).

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotDesktop, DesktopGrant, DesktopGrantIdParams, DesktopGrantsParams};

use super::{ApiError, ApiResult};
use crate::state::{Daemon, Event};

pub fn grants(daemon: &Daemon, params: DesktopGrantsParams) -> ApiResult<Vec<DesktopGrant>> {
    let store = daemon.store();
    if store.bot(&params.bot_id)?.is_none() {
        return Err(ApiError::NotFound(format!("bot {}", params.bot_id)));
    }
    Ok(store.desktop_grants(&params.bot_id)?)
}

/// Takes a grant away: the bot asks for that again.
pub fn revoke(daemon: &Daemon, params: DesktopGrantIdParams) -> ApiResult<BotDesktop> {
    let bot_id = daemon
        .store()
        .revoke_desktop_grant(&params.grant_id)?
        .ok_or_else(|| ApiError::NotFound(format!("desktop grant {}", params.grant_id)))?;
    changed(daemon, &bot_id)
}

/// Tells the apps the bot's grants now.
pub(crate) fn changed(daemon: &Daemon, bot_id: &BotId) -> ApiResult<BotDesktop> {
    let desktop = BotDesktop {
        grants: daemon.store().desktop_grants(bot_id)?,
        bot_id: bot_id.clone(),
    };
    daemon.emit(Event::BotDesktop(desktop.clone()));
    Ok(desktop)
}
