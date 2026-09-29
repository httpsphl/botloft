//! `settings.get` and `settings.update` (spec 11.2): what the owner
//! changes in the app's Settings.

use botloft_core::protocol::{Settings, SettingsUpdateParams};
use tracing::info;

use super::{ApiError, ApiResult};
use crate::autostart;
use crate::state::Daemon;

pub fn get(daemon: &Daemon) -> ApiResult<Settings> {
    Ok(daemon.settings.get())
}

/// Saves the change in `config.toml` and applies it: the scheduled task
/// follows starting with Windows (spec 14), and keeping the computer
/// awake turns on or off at once.
pub fn update(daemon: &Daemon, params: SettingsUpdateParams) -> ApiResult<Settings> {
    let before = daemon.settings.get();
    if let Some(start) = params.start_with_windows
        && start != before.start_with_windows
    {
        autostart::set_start_with_windows(&daemon.paths.home, start)
            .map_err(|err| ApiError::Settings(format!("{err:#}")))?;
    }
    let after = daemon
        .settings
        .save(params)
        .map_err(|err| ApiError::Settings(err.to_string()))?;
    info!(
        start_with_windows = after.start_with_windows,
        keep_awake = after.keep_awake,
        "settings changed"
    );
    Ok(after)
}
