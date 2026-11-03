//! What a bot's Claude Code session runs with and how full it is: the
//! effort level (spec 7.4) and the context in use (spec 8.6).

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

text_enum!(
    /// How much a bot thinks before it answers (spec 7.4): Claude Code's
    /// `--effort` levels, or the level its model uses by itself.
    BotEffort, "effort" {
        /// No `--effort`: the level Claude Code uses for the bot's model.
        Default => "default",
        Low => "low",
        Medium => "medium",
        High => "high",
        Xhigh => "xhigh",
        Max => "max",
    }
);

impl BotEffort {
    /// The value of Claude Code's `--effort`; `None` leaves the flag out.
    pub fn cli_value(self) -> Option<&'static str> {
        match self {
            Self::Default => None,
            level => Some(level.as_str()),
        }
    }
}

text_enum!(
    /// The effort a model runs at when the owner picks none, as Claude Code
    /// reports it (spec 7.4).
    ModelEffort, "model effort" {
        /// The model takes no effort level.
        None => "none",
        Low => "low",
        Medium => "medium",
        High => "high",
        Xhigh => "xhigh",
        Max => "max",
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsSetEffortParams {
    pub bot_id: BotId,
    pub effort: BotEffort,
}

/// How full a bot's conversation is, in tokens, as Claude Code counts it
/// (spec 8.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ContextUsage {
    /// What the conversation holds now.
    pub used_tokens: u64,
    /// The most it can hold.
    pub window_tokens: u64,
    /// Where Claude Code compacts the conversation by itself; `null` when it
    /// does not.
    pub auto_compact_tokens: Option<u64>,
    /// The conversation is being compacted right now.
    pub compacting: bool,
    /// Unix time in milliseconds.
    pub updated_at: i64,
    /// What the conversation holds, part by part, as of Claude Code's last
    /// answer (spec 33.3); empty when the agent does not say.
    #[serde(default)]
    pub parts: Vec<ContextPart>,
    /// The memory files the agent loaded.
    #[serde(default)]
    pub memory_files: Vec<ContextFile>,
}

text_enum!(
    /// What a part of a bot's conversation is (spec 33.3), from the
    /// categories Claude Code counts.
    ContextPartKind, "context part" {
        Messages => "messages",
        /// Claude Code's own instructions.
        Instructions => "instructions",
        /// Claude Code's own tools.
        Tools => "tools",
        /// The tools of the MCP servers, with their instructions.
        ConnectedTools => "connected_tools",
        Memory => "memory",
        Skills => "skills",
        /// Custom subagents.
        Agents => "agents",
        /// What Claude Code keeps free to compact in.
        Buffer => "buffer",
        Other => "other",
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ContextPart {
    pub kind: ContextPartKind,
    pub tokens: u64,
}

/// A memory file in a bot's conversation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ContextFile {
    /// As Claude Code gives it: absolute.
    pub path: String,
    pub tokens: u64,
}

/// Params of the `bot.context` notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotContextChanged {
    pub bot_id: BotId,
    /// `null` when the bot began a new conversation and its size is not
    /// known yet.
    pub context: Option<ContextUsage>,
}

text_enum!(
    /// What a bot left running in the background (spec 33.4), from
    /// Claude Code's `task_type`.
    BackgroundKind, "background kind" {
        /// A shell command, or a monitor watching one (`local_bash`).
        Command => "command",
        /// A subagent (`local_agent`).
        Agent => "agent",
        /// An agent running elsewhere (`remote_agent`).
        Remote => "remote",
        Other => "other",
    }
);

/// A task a bot left running in the background (spec 33.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BackgroundTask {
    /// Claude Code's id, which `bots.stopTask` takes.
    pub task_id: String,
    pub kind: BackgroundKind,
    /// What the bot said it is.
    pub description: String,
    /// When the daemon first saw it running, in Unix milliseconds.
    pub seen_at: i64,
}

/// Params of the `bot.background` notification: the whole live list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotBackgroundChanged {
    pub bot_id: BotId,
    pub tasks: Vec<BackgroundTask>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsStopTaskParams {
    pub bot_id: BotId,
    pub task_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn efforts_read_as_claude_code_names_them() {
        assert_eq!(BotEffort::Default.cli_value(), None);
        assert_eq!(BotEffort::Xhigh.cli_value(), Some("xhigh"));
        assert_eq!(
            "xhigh".parse::<ModelEffort>().ok(),
            Some(ModelEffort::Xhigh)
        );
        assert_eq!(
            serde_json::to_value(ModelEffort::None).expect("serialize"),
            serde_json::json!("none")
        );
        assert!("ultra".parse::<ModelEffort>().is_err());
    }
}
