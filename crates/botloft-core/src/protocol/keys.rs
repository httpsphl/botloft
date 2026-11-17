//! An agent's keys (spec 32): the names the app lists. A key's value goes in
//! with `keys.set` and never comes back out.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, ChatItemId};

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
    /// The bot's request this answers (spec 32.5); its name must match.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub request_id: Option<ChatItemId>,
}

/// `keys.dismiss`: the owner does not give the key now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeysDismissParams {
    pub request_id: ChatItemId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeysDeleteParams {
    pub bot_id: BotId,
    pub name: String,
}

/// Something in the conversations that looks like a pasted key (spec 32.7),
/// once per value however many times it shows up.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeyHit {
    /// Opaque; names the value to `keys.scrub` without carrying it.
    pub id: String,
    /// How it starts and its last 4 characters, like `sk-…a1b2`.
    pub preview: String,
    /// The agents whose conversations have it, the latest first.
    pub bot_ids: Vec<BotId>,
    /// How many texts have it: messages, chat items, requests, questions.
    pub places: u32,
    /// The newest of those, in Unix ms.
    pub last_at: i64,
}

/// What to do with one hit: keep it as a key of an agent, or just take it
/// out of Botloft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeyScrub {
    pub id: String,
    /// The agent and the name to save it as; `null` deletes it.
    pub keep: Option<KeyKeep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeyKeep {
    pub bot_id: BotId,
    pub name: String,
}

/// `keys.scrub`; answers with what is left to scan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct KeysScrubParams {
    pub hits: Vec<KeyScrub>,
}
