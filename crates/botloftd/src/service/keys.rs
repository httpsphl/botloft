//! `keys.list`, `keys.set` and `keys.delete` (spec 32): the owner keeps an
//! agent's keys out of the chat. A value goes in and never comes back out;
//! saving or removing one restarts the agent, resuming its conversation,
//! once nothing is in progress.

use botloft_core::ids::BotId;
use botloft_core::keys;
use botloft_core::protocol::{AgentKey, KeysDeleteParams, KeysListParams, KeysSetParams};
use tracing::info;

use super::{ApiError, ApiResult};
use crate::state::Daemon;

fn known(daemon: &Daemon, bot: &BotId) -> ApiResult<()> {
    match daemon.store().bot(bot)? {
        Some(_) => Ok(()),
        None => Err(ApiError::NotFound(format!("bot {bot} does not exist"))),
    }
}

/// The agent's keys, with the ones whose value is not on this computer
/// marked `missing` (spec 32.2).
pub fn listed(daemon: &Daemon, bot: &BotId) -> ApiResult<Vec<AgentKey>> {
    let mut keys = daemon.store().agent_keys(bot)?;
    let values = daemon.keys.pairs(&daemon.paths.secrets(), bot);
    for key in &mut keys {
        key.missing = !values.iter().any(|(name, _)| name == &key.name);
    }
    Ok(keys)
}

pub fn list(daemon: &Daemon, params: KeysListParams) -> ApiResult<Vec<AgentKey>> {
    known(daemon, &params.bot_id)?;
    listed(daemon, &params.bot_id)
}

pub fn set(daemon: &Daemon, params: KeysSetParams) -> ApiResult<Vec<AgentKey>> {
    let bot = &params.bot_id;
    known(daemon, bot)?;
    let name = keys::name(&params.name)?;
    let value = keys::value(&params.value)?;
    let description = params
        .description
        .as_deref()
        .map(keys::description)
        .transpose()?;
    daemon
        .keys
        .set(&daemon.paths.secrets(), bot, &name, &value)
        .map_err(ApiError::Workspace)?;
    let now = daemon.clock.now_ms();
    daemon
        .store()
        .set_agent_key(bot, &name, description.as_deref(), now)?;
    info!(bot = %bot, key = %name, "agent key saved");
    daemon.supervisor.launch_settings_changed(bot);
    listed(daemon, bot)
}

pub fn delete(daemon: &Daemon, params: KeysDeleteParams) -> ApiResult<Vec<AgentKey>> {
    let bot = &params.bot_id;
    known(daemon, bot)?;
    if !daemon.store().delete_agent_key(bot, &params.name)? {
        return Err(ApiError::NotFound(format!(
            "key {} does not exist",
            params.name
        )));
    }
    daemon
        .keys
        .remove(&daemon.paths.secrets(), bot, Some(&params.name))
        .map_err(ApiError::Workspace)?;
    info!(bot = %bot, key = %params.name, "agent key removed");
    daemon.supervisor.launch_settings_changed(bot);
    listed(daemon, bot)
}
