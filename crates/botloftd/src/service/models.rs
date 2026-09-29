//! A bot's model (spec 7.4): what the owner picks, and the model Claude
//! Code reports at the start of each turn.

use botloft_core::ids::BotId;
use botloft_core::protocol::{Bot, BotsSetModelParams};
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
    let bot = changed(daemon, &store, &crew, record);
    drop(store);
    info!(bot = %params.bot_id, model = params.model.as_str(), "model changed");
    daemon.supervisor.launch_settings_changed(&params.bot_id);
    Ok(bot)
}

/// The model id in `system/init`, so the app can name the model a bot
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
