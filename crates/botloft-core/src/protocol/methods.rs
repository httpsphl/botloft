//! Params and results of the request methods (spec 11.2).

use std::fmt;

use serde::{Deserialize, Serialize};

use super::{DeliveryBacklog, PermissionMode};
use crate::ids::{BotId, CrewId};

/// Identifies the connecting app in `session.hello`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

/// First request of every connection.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct HelloParams {
    /// Owner token from `secrets\owner.token`.
    pub token: String,
    pub client: ClientInfo,
    pub protocol: u32,
}

impl fmt::Debug for HelloParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HelloParams")
            .field("token", &"<redacted>")
            .field("client", &self.client)
            .field("protocol", &self.protocol)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct HelloResult {
    pub daemon_version: String,
    pub protocol: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SystemStatus {
    pub daemon_version: String,
    pub protocol: u32,
    pub uptime_ms: i64,
    /// Version reported by `claude --version`; `null` until the runtime probe runs.
    pub claude_version: Option<String>,
    /// Why bots cannot start (Claude Code missing or too old); `null` when fine.
    pub runtime_error: Option<String>,
    /// The Claude Code executable the bots run; `null` until it is found.
    pub claude_path: Option<String>,
    /// Whether Claude Code is signed in (`claude auth status`); `null` until checked.
    pub claude_signed_in: Option<bool>,
    /// Who the owner is, for the account area of the app.
    pub account: OwnerAccount,
    pub deliveries: DeliveryBacklog,
    /// The Claude account's usage as last reported; `null` before any turn.
    pub usage: Option<AccountUsage>,
}

/// The owner, as the app shows them in its account area (spec 15.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct OwnerAccount {
    /// The Windows account's display name, or the user name without one.
    pub name: String,
    /// The Claude account Claude Code is signed in to; `null` when signed out
    /// or not checked yet.
    pub claude: Option<ClaudeAccount>,
}

/// What `claude auth status` says about the account (spec 7.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ClaudeAccount {
    pub email: Option<String>,
    /// As Claude Code says it: `max`, `pro`, `team`, `enterprise`...
    pub plan: Option<String>,
    pub organization: Option<String>,
}

/// The account's usage, from Claude Code's `rate_limit_event` (spec 8.1).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AccountUsage {
    /// As Claude Code says it: `allowed`, `allowed_warning`, `rejected`...
    pub status: String,
    /// Unix time in milliseconds when the current limit resets.
    pub resets_at: Option<i64>,
    pub windows: Vec<UsageWindow>,
    /// Unix time in milliseconds.
    pub observed_at: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct UsageWindow {
    /// `five_hour`, `seven_day`...
    pub name: String,
    /// Share used, 0 to 1.
    pub utilization: f64,
    /// Unix time in milliseconds.
    pub resets_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CrewsCreateParams {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CrewsRenameParams {
    pub crew_id: CrewId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CrewsSetPausedParams {
    pub crew_id: CrewId,
    pub paused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CrewIdParams {
    pub crew_id: CrewId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsListParams {
    /// Only bots of this crew; every active bot when absent.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub crew_id: Option<CrewId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsCreateParams {
    pub crew_id: CrewId,
    pub name: String,
    pub role: String,
    pub instructions: String,
    /// Avatar color `#RRGGBB`; the next palette color when absent.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub color: Option<String>,
}

/// Fields left out stay unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsUpdateParams {
    pub bot_id: BotId,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub name: Option<String>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub role: Option<String>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub instructions: Option<String>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub color: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsSetPausedParams {
    pub bot_id: BotId,
    pub paused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsSetPermissionModeParams {
    pub bot_id: BotId,
    pub mode: PermissionMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotIdParams {
    pub bot_id: BotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsRestartParams {
    pub bot_id: BotId,
    /// Start a new conversation instead of resuming the last one.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub fresh: Option<bool>,
}
