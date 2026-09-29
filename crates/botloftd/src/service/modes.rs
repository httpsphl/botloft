//! A bot's permission mode (spec 7.4): what the owner picks in the chat,
//! and what Claude Code reports when it changes the mode by itself.

use botloft_core::ids::BotId;
use botloft_core::protocol::{Bot, BotsSetPermissionModeParams, PermissionMode};
use tracing::{info, warn};

use super::ApiResult;
use super::bots::{active, changed, to_protocol};
use crate::state::Daemon;

/// Saves the mode; the bot restarts into it when nothing is in progress.
pub fn set_permission_mode(daemon: &Daemon, params: BotsSetPermissionModeParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, mut record) = active(&store, &params.bot_id)?;
    if record.permission_mode == params.mode {
        return Ok(to_protocol(daemon, &store, &crew, record));
    }
    record.permission_mode = params.mode;
    store.update_bot(&record)?;
    let bot = changed(daemon, &store, &crew, record);
    drop(store);
    info!(bot = %params.bot_id, mode = params.mode.as_str(), "permission mode changed");
    daemon.supervisor.permission_mode_changed(&params.bot_id);
    Ok(bot)
}

/// The mode Claude Code reports in `system/init`. It leaves plan mode by
/// itself when the owner approves a plan; the stored mode follows, so the
/// app shows what the bot does and a restart keeps it. Any other difference
/// comes from a turn of a process that is about to restart into the mode
/// the owner just picked, and is ignored.
pub(crate) fn reported(daemon: &Daemon, bot: &BotId, reported: &str) {
    let Some(mode) = PermissionMode::from_cli(reported) else {
        return;
    };
    if mode == PermissionMode::Plan || daemon.supervisor.mode_change_pending(bot) {
        return;
    }
    let store = daemon.store();
    let Ok((crew, mut record)) = active(&store, bot) else {
        return;
    };
    if record.permission_mode != PermissionMode::Plan {
        return;
    }
    record.permission_mode = mode;
    if let Err(err) = store.update_bot(&record) {
        warn!(bot = %bot, "could not save the mode after the plan: {err}");
        return;
    }
    changed(daemon, &store, &crew, record);
}
