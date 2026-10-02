//! "Allow always" (spec 10.1): what the owner let a bot do without asking
//! again, kept as rules of that bot.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, RuleId};

/// What an allow rule covers, besides its tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum AllowKind {
    /// This exact command (`Bash`, `PowerShell`).
    Command,
    /// Pages of this site and its subdomains (`WebFetch`).
    Site,
    /// This file (`Read`, `Write`, `Edit`, `MultiEdit`, `NotebookEdit`).
    File,
    /// Every call of the tool (`WebSearch`).
    Tool,
}

impl AllowKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Command => "command",
            Self::Site => "site",
            Self::File => "file",
            Self::Tool => "tool",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "command" => Self::Command,
            "site" => Self::Site,
            "file" => Self::File,
            "tool" => Self::Tool,
            _ => return None,
        })
    }
}

/// What "Allow always" would cover for one request: the tool and, for a
/// command, site or file, which one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AllowScope {
    pub tool_name: String,
    pub kind: AllowKind,
    /// The command, the site or the file's path; empty for `tool`.
    pub value: String,
}

/// A request the bot no longer asks for: the daemon allows it alone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AllowRule {
    pub id: RuleId,
    pub bot_id: BotId,
    pub scope: AllowScope,
    /// Unix time in milliseconds.
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RulesListParams {
    pub bot_id: BotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RuleIdParams {
    pub rule_id: RuleId,
}

/// A bot's rules after one was added or removed: the result of
/// `rules.delete` and the params of the `bot.rules` notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotRules {
    pub bot_id: BotId,
    /// Oldest first.
    pub rules: Vec<AllowRule>,
}
