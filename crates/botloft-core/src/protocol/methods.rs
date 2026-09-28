//! Params and results of the request methods (spec 11.2).

use std::fmt;

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
