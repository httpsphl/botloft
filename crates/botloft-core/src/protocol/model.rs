//! Entities the daemon returns and broadcasts.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, CrewId};

/// A group of bots that can message each other and share a folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Crew {
    pub id: CrewId,
    pub name: String,
    /// Folder name under the workspaces root. Set at creation, never changes.
    pub slug: String,
    pub paused: bool,
    /// Unix time in milliseconds.
    pub created_at: i64,
    /// Unix time in milliseconds; `null` while the crew is active.
    pub archived_at: Option<i64>,
}

/// Lifecycle state of a bot (spec 7.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum BotState {
    Offline,
    Launching,
    Idle,
    Busy,
    NeedsApproval,
    RateLimited,
    AuthError,
    Backoff,
    Archived,
}

/// A persistent Claude Code session with a name, a role and instructions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Bot {
    pub id: BotId,
    pub crew_id: CrewId,
    pub name: String,
    /// Address other bots use (`send_message(to: "<handle>")`). Derived from
    /// the name and unique among the crew's active bots.
    pub handle: String,
    /// Folder name inside the crew folder. Set at creation, never changes.
    pub slug: String,
    pub role: String,
    pub instructions: String,
    /// Avatar color, `#RRGGBB`.
    pub color: String,
    pub paused: bool,
    pub state: BotState,
    /// Current process generation; `null` if the bot has not started since
    /// the daemon did. Changes on every (re)start (spec 8).
    pub generation: Option<u64>,
    /// Absolute path of the bot's workspace folder.
    pub workspace: String,
    /// Unix time in milliseconds.
    pub created_at: i64,
    /// Unix time in milliseconds; `null` while the bot is active.
    pub archived_at: Option<i64>,
}

/// Params of the `bot.state` notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotStateChanged {
    pub bot_id: BotId,
    pub state: BotState,
    pub generation: Option<u64>,
}
