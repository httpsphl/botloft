//! Several conversations of one agent (spec 40).

use serde::{Deserialize, Serialize};

use super::{BackgroundTask, BotState, ContextUsage};
use crate::ids::{BotId, ConversationId};

text_enum!(
    /// Who gave a conversation its title (spec 40.6).
    TitleBy, "title author" {
        Owner => "owner",
        Ai => "ai",
    }
);

/// One conversation of an agent (spec 40.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Conversation {
    pub id: ConversationId,
    pub bot_id: BotId,
    /// `null` until the owner or the AI names it; the main one has none.
    pub title: Option<String>,
    pub title_by: Option<TitleBy>,
    /// The agent's main conversation, which gets what comes from outside
    /// (spec 40.2) and is never renamed or archived.
    pub main: bool,
    pub state: BotState,
    /// The process it runs in; `null` while it has none.
    pub generation: Option<u64>,
    /// How full it is; `null` while that is not known (spec 8.6).
    pub context: Option<ContextUsage>,
    /// What it left running in the background (spec 33.4).
    pub background: Vec<BackgroundTask>,
    /// Its newest chat item, or when it was made: Unix milliseconds.
    pub last_activity: i64,
    pub created_at: i64,
    pub archived_at: Option<i64>,
}

/// `conversations.archive`, `conversations.unarchive`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ConversationIdParams {
    pub conversation_id: ConversationId,
}

/// `conversations.rename`: the owner names a conversation, and the AI no
/// longer does (spec 40.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ConversationsRenameParams {
    pub conversation_id: ConversationId,
    pub title: String,
}

/// A method that acts on one conversation of a bot: the main one when
/// `conversationId` is absent (spec 40.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotConversationParams {
    pub bot_id: BotId,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub conversation_id: Option<ConversationId>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_conversation_means_the_main_one() {
        let params: BotConversationParams =
            serde_json::from_str(r#"{"botId":"bot_01J9Z3K8000000000000000000"}"#).unwrap();
        assert_eq!(params.conversation_id, None);
        assert_eq!(TitleBy::Ai.as_str(), "ai");
        assert_eq!("owner".parse::<TitleBy>().ok(), Some(TitleBy::Owner));
    }
}
