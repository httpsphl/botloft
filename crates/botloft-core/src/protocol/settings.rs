//! What the owner changes in the app's Settings and the daemon keeps in
//! `config.toml` (spec 6, 11.2).

use serde::{Deserialize, Serialize};

/// The daemon's part of the app's Settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Settings {
    /// Botloft starts when the owner signs in to Windows (spec 14). Off,
    /// the bots wait until the owner opens the app.
    pub start_with_windows: bool,
    /// The computer does not sleep while a bot works (spec 14).
    pub keep_awake: bool,
}

/// The settings to change; the ones left out stay as they are.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SettingsUpdateParams {
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub start_with_windows: Option<bool>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub keep_awake: Option<bool>,
}
