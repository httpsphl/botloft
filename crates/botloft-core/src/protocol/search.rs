//! Searching the chats (spec 8.8): `chat.search`.

use serde::{Deserialize, Serialize};

use super::ChatItem;
use crate::ids::{BotId, ChatItemId, CrewId};

/// Marks where a matched word starts and ends in a snippet: the app shows
/// what is between them highlighted.
pub const SNIPPET_OPEN: char = '\u{2}';
pub const SNIPPET_CLOSE: char = '\u{3}';

/// Newest first. Page back with `before` set to the oldest item received.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChatSearchParams {
    /// Words to find, as the owner typed them; the last one may be the
    /// start of a word.
    pub query: String,
    /// Only this bot's chat.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub bot_id: Option<BotId>,
    /// Only the chats of this crew's bots.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub crew_id: Option<CrewId>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub before: Option<ChatItemId>,
    /// 1 to 100; 30 when absent.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub limit: Option<u32>,
}

/// A chat item with every searched word.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SearchHit {
    pub item: ChatItem,
    pub crew_id: CrewId,
    /// A few words around the match, on one line, with each matched word
    /// between `\u0002` and `\u0003`.
    pub snippet: String,
}
