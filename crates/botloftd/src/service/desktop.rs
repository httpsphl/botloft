//! What each bot may see and do on the owner's desktop (spec 24.2,
//! 24.10): `desktop.grants` and `desktop.revoke`. Grants come from the
//! owner answering a bot's request (`tools/desktop.rs`). And stopping a
//! bot there, or every bot at once (spec 24.9): `desktop.stop` and
//! `desktop.resume`.

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    BotDesktop, DesktopBotParams, DesktopGrant, DesktopGrantIdParams, DesktopGrantsParams,
    DesktopState,
};
use tracing::{info, warn};

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

/// Stops the bot on the desktop, or lets it go on.
fn set_stopped(daemon: &Daemon, bot: &BotId, stopped: bool) -> DesktopState {
    let state = daemon.desktop.activity.set_stopped(bot, stopped);
    daemon.emit(Event::DesktopChanged(state.clone()));
    state
}

pub fn stop(daemon: &Daemon, params: DesktopBotParams) -> ApiResult<DesktopState> {
    super::bots::find(&daemon.store(), &params.bot_id)?;
    Ok(set_stopped(daemon, &params.bot_id, true))
}

pub fn resume(daemon: &Daemon, params: DesktopBotParams) -> ApiResult<DesktopState> {
    super::bots::find(&daemon.store(), &params.bot_id)?;
    Ok(set_stopped(daemon, &params.bot_id, false))
}

/// Stops every bot on the desktop: the owner pressed the stop shortcut
/// (spec 24.9).
pub fn stop_all(daemon: &Daemon) {
    let bots = match daemon.store().bots(None, false) {
        Ok(bots) => bots,
        Err(err) => {
            warn!("could not list the bots to stop them on the desktop: {err}");
            return;
        }
    };
    for bot in &bots {
        set_stopped(daemon, &bot.id, true);
    }
    info!(
        bots = bots.len(),
        "the owner stopped every bot on the desktop"
    );
}
