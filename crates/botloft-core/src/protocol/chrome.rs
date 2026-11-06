//! The owner's own Chrome in the agents (spec 37): which agents the owner
//! gave it to, and which one acts in it now.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// An agent the owner gave their Chrome to (spec 37.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChromeGrant {
    pub bot_id: BotId,
    /// Unix time in milliseconds.
    pub given_at: i64,
}

/// `chrome.set`: gives the owner's Chrome to an agent, or takes it back.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChromeSetParams {
    pub bot_id: BotId,
    pub on: bool,
    /// The owner read the risks and ticked that they understood them;
    /// giving needs it.
    #[serde(default)]
    pub accepted_risks: bool,
}

/// One agent's Chrome: the result of `chrome.set` and the params of the
/// `bot.chrome` notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotChrome {
    pub bot_id: BotId,
    /// Unix time in milliseconds; `null` when the agent does not have it.
    pub given_at: Option<i64>,
}

/// Who acts in the owner's Chrome now (spec 37.4): the result of
/// `chrome.status` and the params of `chrome.turn`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChromeTurn {
    /// `null` when no agent has the turn.
    pub bot_id: Option<BotId>,
}
