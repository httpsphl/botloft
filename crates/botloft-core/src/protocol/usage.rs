//! What the bots use (spec 8.7): tokens and the API price of each turn in
//! the chat, and the sums per bot, model, day and routine for the usage
//! dialog.

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
    /// What the bot's turns would cost at API list prices, in US dollars;
    /// `null` when its engine does not say (spec 8.7). The owner's plan
    /// does not charge it.
    pub cost: Option<f64>,
}

text_enum!(
    /// How `usage.costs` groups the cost (spec 8.7).
    CostBy, "cost grouping" {
        Model => "model",
        /// The day in the computer's time zone.
        Day => "day",
        Routine => "routine",
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct UsageCostsParams {
    /// Counts what was spent at or after this Unix time in ms.
    pub since: i64,
    pub by: CostBy,
}

/// The cost of one model, day or routine since the time asked for.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CostGroup {
    /// The model's id, the day (`2026-11-14`) or the routine's id; empty for
    /// the cost with no model (from before Botloft kept it) or not from a
    /// routine.
    pub key: String,
    /// At API list prices, in US dollars.
    pub cost: f64,
    /// The routine's name; absent elsewhere and for a deleted routine.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub name: Option<String>,
    /// The bot the routine belongs to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub bot_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub color: Option<String>,
}
