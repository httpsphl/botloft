//! Tokens the bots use (spec 8.7): per turn in the chat, and summed per bot
//! for the usage dialog.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// Tokens of one turn, as the `usage` of Claude Code's `result` event: the
/// sum of every request to the model in the turn.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TokenUsage {
    /// New input the model read (`input_tokens`).
    pub input: u64,
    /// New input also written to the prompt cache (`cache_creation_input_tokens`).
    pub cache_write: u64,
    /// Of the cache write, the conversation from before written again because
    /// the cache had expired: the daemon's estimate, 0 in older turns.
    #[serde(default)]
    pub reloaded: u64,
    /// Input read back from the prompt cache (`cache_read_input_tokens`):
    /// mostly the conversation so far, and much cheaper than new input.
    pub cache_read: u64,
    /// What the model wrote, thinking included (`output_tokens`).
    pub output: u64,
}

impl TokenUsage {
    pub fn add(&mut self, other: &Self) {
        self.input += other.input;
        self.cache_write += other.cache_write;
        self.reloaded += other.reloaded;
        self.cache_read += other.cache_read;
        self.output += other.output;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct UsageTokensParams {
    /// Counts turns that ended at or after this Unix time in ms.
    pub since: i64,
}

/// One bot's tokens since the time asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotTokens {
    pub bot_id: BotId,
    pub name: String,
    pub color: String,
    /// The name of the bot's crew, so two bots with one name stay apart.
    pub crew: String,
    /// The bot is archived.
    pub archived: bool,
    /// Turns that reported tokens.
    pub turns: u32,
    pub tokens: TokenUsage,
    /// About how much of the weekly plan the bot used in the period, 0 to
    /// 1; `null` until Botloft has learned it (spec 8.7).
    pub plan_share: Option<f64>,
}
