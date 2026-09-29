//! `settings.get` and `settings.update` (spec 11.2): what the owner
//! changes in the app's Settings.

use botloft_core::protocol::{APPROVAL_WAIT_MAX_MINUTES, Settings, SettingsUpdateParams};
use tracing::info;

use super::{ApiError, ApiResult};
use crate::autostart;
use crate::state::Daemon;

pub fn get(daemon: &Daemon) -> ApiResult<Settings> {
    Ok(daemon.settings.get())
}

/// Saves the change in `config.toml` and applies it: the scheduled task
/// follows starting with Windows (spec 14), keeping the computer awake
/// turns on or off at once, and so does a shorter wait for approvals. A
/// longer one reaches each bot when it restarts, as soon as nothing is in
/// progress: Claude Code reads the wait in `mcp.json` when it starts
/// (spec 10.1).
pub fn update(daemon: &Daemon, params: SettingsUpdateParams) -> ApiResult<Settings> {
    if params
        .approval_wait_minutes
        .is_some_and(|minutes| !(1..=APPROVAL_WAIT_MAX_MINUTES).contains(&minutes))
    {
        return Err(ApiError::validation(format!(
            "approvalWaitMinutes must be between 1 and {APPROVAL_WAIT_MAX_MINUTES}"
        )));
    }
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
    if after.approval_wait_minutes > before.approval_wait_minutes {
        daemon.supervisor.workspace_settings_changed();
    }
    info!(
        start_with_windows = after.start_with_windows,
        keep_awake = after.keep_awake,
        approval_wait_minutes = after.approval_wait_minutes,
        "settings changed"
    );
    Ok(after)
}
