//! What the owner changes in the app's Settings and the daemon keeps in
//! `config.toml` (spec 6, 11.2).

use serde::{Deserialize, Serialize};

use super::AgentKind;

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
    /// How long a permission request waits for the owner before it is
    /// denied (spec 10.1).
    pub approval_wait_minutes: u32,
    /// The agent bots are made for when nothing says otherwise: new bots,
    /// the chief of a new crew, the bots of a template (spec 30).
    pub default_agent: AgentKind,
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
    /// From 1 to [`APPROVAL_WAIT_MAX_MINUTES`].
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub approval_wait_minutes: Option<u32>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub default_agent: Option<AgentKind>,
}

/// The longest a permission request may wait for the owner: a day.
pub const APPROVAL_WAIT_MAX_MINUTES: u32 = 24 * 60;
