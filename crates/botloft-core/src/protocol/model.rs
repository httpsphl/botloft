//! Entities the daemon returns and broadcasts.

use serde::{Deserialize, Serialize};

use super::Activity;
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
    /// Absolute path of the folder the crew works in (spec 5): one the owner
    /// chose, or the crew's `shared` folder.
    pub work_folder: String,
    /// Whether the owner chose `work_folder`.
    pub work_folder_chosen: bool,
    /// The bot that leads the crew and may suggest new bots (spec 10.2);
    /// `null` when the crew has no chief.
    pub lead_bot_id: Option<BotId>,
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

text_enum!(
    /// How much a bot may do without asking the owner: Claude Code's
    /// permission modes (spec 7.4).
    PermissionMode, "permission mode" {
        /// Asks before edits, commands and the network ("Manual").
        Default => "default",
        /// Edits files and runs common file commands without asking.
        AcceptEdits => "accept_edits",
        /// Plans first, then asks to go ahead with the plan.
        Plan => "plan",
        /// A classifier reviews each action; what it does not allow is asked.
        Auto => "auto",
        /// Does everything without asking.
        BypassPermissions => "bypass_permissions",
    }
);

impl PermissionMode {
    /// The value of Claude Code's `--permission-mode`, and what it reports
    /// in `system/init`.
    pub fn cli_value(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::AcceptEdits => "acceptEdits",
            Self::Plan => "plan",
            Self::Auto => "auto",
            Self::BypassPermissions => "bypassPermissions",
        }
    }

    pub fn from_cli(value: &str) -> Option<Self> {
        [
            Self::Default,
            Self::AcceptEdits,
            Self::Plan,
            Self::Auto,
            Self::BypassPermissions,
        ]
        .into_iter()
        .find(|mode| mode.cli_value() == value)
    }
}

text_enum!(
    /// Which Claude model a bot runs on (spec 7.4): Claude Code's `--model`
    /// aliases, or the account's default.
    BotModel, "model" {
        /// No `--model`: the default of the owner's Claude plan.
        Default => "default",
        Fable => "fable",
        Opus => "opus",
        Sonnet => "sonnet",
        Haiku => "haiku",
    }
);

impl BotModel {
    /// The value of Claude Code's `--model`; `None` leaves the flag out.
    pub fn cli_value(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            Self::Fable => Some("fable"),
            Self::Opus => Some("opus"),
            Self::Sonnet => Some("sonnet"),
            Self::Haiku => Some("haiku"),
        }
    }
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
    pub permission_mode: PermissionMode,
    pub model: BotModel,
    /// The model id Claude Code reported when the bot last started a turn
    /// (`claude-opus-5-5`); `null` before its first turn.
    pub model_in_use: Option<String>,
    pub state: BotState,
    /// Current process generation; `null` if the bot has not started since
    /// the daemon did. Changes on every (re)start.
    pub generation: Option<u64>,
    /// Absolute path of the bot's workspace folder.
    pub workspace: String,
    /// The last item of the bot's chat, in one line; `null` for a new bot.
    pub last_activity: Option<Activity>,
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
