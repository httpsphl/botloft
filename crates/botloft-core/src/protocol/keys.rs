//! An agent's keys (spec 32): the names the app lists. A key's value goes in
//! with `keys.set` and never comes back out.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// A key of an agent, without its value (spec 32.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AgentKey {
    pub name: String,
    /// What the key is, as the owner wrote it; empty without one.
    pub description: String,
    /// When its value last changed, in Unix ms.
    pub set_at: i64,
    /// The value is not on this computer, e.g. after restoring a copy.
    pub missing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeysListParams {
    pub bot_id: BotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeysSetParams {
    pub bot_id: BotId,
    pub name: String,
    pub value: String,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeysDeleteParams {
    pub bot_id: BotId,
    pub name: String,
}
