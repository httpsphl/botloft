//! What the owner let a bot reach in another crew for good (spec 10.4), as
//! the bot's details list it.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, CrewAccessId, CrewId};

/// One lasting access: the whole crew, or one bot of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CrewAccess {
    pub id: CrewAccessId,
    /// The bot that may reach.
    pub bot_id: BotId,
    pub crew_id: CrewId,
    pub crew_name: String,
    /// One bot of the crew; `null` for the whole crew.
    pub target_bot_id: Option<BotId>,
    pub target_name: Option<String>,
    /// Unix time in milliseconds.
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CrewAccessListParams {
    pub bot_id: BotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CrewAccessIdParams {
    pub access_id: CrewAccessId,
}
