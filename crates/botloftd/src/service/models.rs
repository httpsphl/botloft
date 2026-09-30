//! A bot's model and effort (spec 7.4): what the owner picks, and what
//! Claude Code reports it applies.

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    Bot, BotEffort, BotsSetEffortParams, BotsSetModelParams, ModelEffort,
};
use serde_json::Value;
use tracing::{info, warn};

use super::ApiResult;
use super::bots::{active, changed, to_protocol};
use crate::state::Daemon;

/// Saves the model; the bot restarts on it when nothing is in progress.
pub fn set_model(daemon: &Daemon, params: BotsSetModelParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, mut record) = active(&store, &params.bot_id)?;
    if record.model == params.model {
        return Ok(to_protocol(daemon, &store, &crew, record));
    }
    record.model = params.model;
    store.update_bot(&record)?;
    // The effort a model uses by itself is that model's: not known again
    // until the bot runs on the new one.
    store.set_effort_default(&record.id, None)?;
    record.effort_default = None;
    let bot = changed(daemon, &store, &crew, record);
    drop(store);
    info!(bot = %params.bot_id, model = params.model.as_str(), "model changed");
    daemon.supervisor.launch_settings_changed(&params.bot_id);
    Ok(bot)
}

/// Saves the effort; the bot restarts on it when nothing is in progress.
pub fn set_effort(daemon: &Daemon, params: BotsSetEffortParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, mut record) = active(&store, &params.bot_id)?;
    if record.effort == params.effort {
        return Ok(to_protocol(daemon, &store, &crew, record));
    }
    record.effort = params.effort;
    store.update_bot(&record)?;
    let bot = changed(daemon, &store, &crew, record);
    drop(store);
    info!(bot = %params.bot_id, effort = params.effort.as_str(), "effort changed");
    daemon.supervisor.launch_settings_changed(&params.bot_id);
    Ok(bot)
}

/// The model id Claude Code reports, so the app can name the model a bot
/// really runs on, the plan's default included.
pub(crate) fn reported(daemon: &Daemon, bot: &BotId, model: &str) {
    let store = daemon.store();
    let Ok((crew, mut record)) = active(&store, bot) else {
        return;
    };
    if model.is_empty() || record.model_in_use.as_deref() == Some(model) {
        return;
    }
    if let Err(err) = store.set_model_in_use(bot, model) {
        warn!(bot = %bot, "could not save the model in use: {err}");
        return;
    }
    record.model_in_use = Some(model.to_owned());
    changed(daemon, &store, &crew, record);
}

/// What the session applies, from Claude Code's answer to `get_settings`
/// when the process starts: the model, before any turn, and the effort.
pub(crate) fn applied(daemon: &Daemon, bot: &BotId, applied: &Value) {
    if let Some(model) = applied["model"].as_str() {
        reported(daemon, bot, model);
    }
    let level = match applied.get("effort") {
        Some(Value::Null) => Some(ModelEffort::None),
        Some(Value::String(level)) => level.parse().ok(),
        _ => None,
    };
    let Some(level) = level else {
        return;
    };
    let launched = daemon.supervisor.launched_effort(bot);
    let store = daemon.store();
    let Ok((crew, mut record)) = active(&store, bot) else {
        return;
    };
    let default = match (level, launched) {
        // No level applies even when one is asked for: the model takes none.
        (ModelEffort::None, _) => Some(ModelEffort::None),
        // Started without `--effort`: this is the model's own level.
        (level, Some(BotEffort::Default)) => Some(level),
        // The owner's level says nothing about the model's own, only that
        // the model does take one.
        _ if record.effort_default == Some(ModelEffort::None) => None,
        _ => return,
    };
    if record.effort_default == default {
        return;
    }
    if let Err(err) = store.set_effort_default(bot, default) {
        warn!(bot = %bot, "could not save the model's effort: {err}");
        return;
    }
    record.effort_default = default;
    changed(daemon, &store, &crew, record);
}
