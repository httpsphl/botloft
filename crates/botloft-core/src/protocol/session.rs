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
