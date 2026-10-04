//! The owner's reactions to what a bot wrote (spec 8.9): one emoji per
//! reply, which the bot reads with the owner's next message instead of
//! being woken for it. With the params of the `reactions.*` methods.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, ChatItemId, MessageId};

/// The emoji the owner can react with.
pub const REACTIONS: &[&str] = &["👍", "❤️", "😂", "🎉", "🙏", "👀"];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Reaction {
    pub bot_id: BotId,
    /// The bot's reply it is on.
    pub item_id: ChatItemId,
    /// One of [`REACTIONS`].
    pub emoji: String,
    /// Unix time in milliseconds.
    pub created_at: i64,
    /// The owner's message that took it to the bot; `null` while it waits
    /// for one.
    pub sent_in: Option<MessageId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ReactionsListParams {
    pub bot_id: BotId,
}

/// Puts the owner's reaction on a reply, or takes it off with `null`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ReactionsSetParams {
    pub bot_id: BotId,
    pub item_id: ChatItemId,
    pub emoji: Option<String>,
}

/// A reaction was put, changed, sent or taken off (`reaction` is `null`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ReactionChanged {
    pub bot_id: BotId,
    pub item_id: ChatItemId,
    pub reaction: Option<Reaction>,
}
