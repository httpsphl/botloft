//! Messages, deliveries and tasks (spec 9 and 12), with the params of the
//! `messages.*`, `deliveries.*` and `tasks.*` methods.

use serde::{Deserialize, Serialize};

use crate::ids::{
    AttachmentId, BotId, ChatItemId, CrewId, DeliveryId, MessageId, QuestionId, RoutineId, TaskId,
};

text_enum!(
    /// Who wrote a message.
    SenderKind, "sender kind" {
        Owner => "owner",
        Bot => "bot",
        /// The daemon itself, e.g. to say a task expired.
        System => "system",
    }
);

text_enum!(
    /// What a message is for.
    MessageKind, "message kind" {
        Note => "note",
        /// Asks for work; linked to a [`Task`].
        Task => "task",
        /// The outcome of a task, sent back to whoever asked for it.
        Result => "result",
        System => "system",
        /// A routine's request, at its time (spec 20.5).
        Routine => "routine",
    }
);

text_enum!(
    /// Where a delivery is in the courier (spec 9.1).
    DeliveryState, "delivery state" {
        Pending => "pending",
        Sending => "sending",
        /// Written to the bot's process; `readAt` says when its turn began.
        /// Neither proves the work was done; tasks do.
        Sent => "sent",
        /// Gave up after `max_attempts`, or the bot was archived.
        Dead => "dead",
    }
);

text_enum!(
    TaskStatus, "task status" {
        Open => "open",
        Done => "done",
        Failed => "failed",
        Cancelled => "cancelled",
        /// Passed its deadline without a result; a late result is still accepted.
        Expired => "expired",
    }
);

/// Text sent to a bot, by the owner, another bot or the daemon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Message {
    pub id: MessageId,
    pub crew_id: CrewId,
    pub from_kind: SenderKind,
    /// Set when `fromKind` is `bot`.
    pub from_bot_id: Option<BotId>,
    pub to_bot_id: BotId,
    pub kind: MessageKind,
    pub body: String,
    /// The task this message asks for, answers or reports on.
    pub task_id: Option<TaskId>,
    /// The routine that sent it (spec 20.5).
    pub routine_id: Option<RoutineId>,
    /// The bot's question this message answers (spec 23.4).
    #[serde(default)]
    pub question_id: Option<QuestionId>,
    /// Files the owner attached (spec 9.5); empty for everything else.
    pub attachments: Vec<Attachment>,
    /// What in the bot's chat the owner replied to (spec 9.3).
    #[serde(default)]
    pub reply_to: Option<MessageReply>,
    /// Unix time in milliseconds.
    pub created_at: i64,
}

/// The chat item a message replies to, and its text as quoted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MessageReply {
    pub item_id: ChatItemId,
    /// The start of the item's text, on one line (at most 300 characters).
    pub text: String,
}

/// A file attached to a message, saved in the bot's folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Attachment {
    pub id: AttachmentId,
    /// File name as saved, without folders.
    pub name: String,
    pub media_type: String,
    /// Bytes.
    pub size: u64,
    /// Path relative to the bot's workspace, with forward slashes.
    pub path: String,
}

/// A file the owner sends with `messages.send`.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AttachmentUpload {
    pub name: String,
    pub media_type: String,
    /// The file's bytes, base64.
    pub data: String,
}

impl std::fmt::Debug for AttachmentUpload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentUpload")
            .field("name", &self.name)
            .field("media_type", &self.media_type)
            .field("data", &format_args!("<{} base64 chars>", self.data.len()))
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AttachmentIdParams {
    pub attachment_id: AttachmentId,
}

/// A saved attachment's bytes, for the app to show (spec 9.5).
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AttachmentData {
    pub media_type: String,
    /// The file's bytes, base64.
    pub data: String,
}

impl std::fmt::Debug for AttachmentData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttachmentData")
            .field("media_type", &self.media_type)
            .field("data", &format_args!("<{} base64 chars>", self.data.len()))
            .finish()
    }
}

/// Getting one message into one bot's process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Delivery {
    pub id: DeliveryId,
    pub message_id: MessageId,
    pub bot_id: BotId,
    pub state: DeliveryState,
    /// Failed attempts so far. Waiting for the bot to be ready is not one.
    pub attempts: u32,
    /// Unix time in milliseconds of the next try while `pending`.
    pub next_attempt_at: i64,
    /// Why the last attempt failed; never contains the message body.
    pub last_error: Option<String>,
    /// Unix time in milliseconds the bot began the turn for this message.
    pub read_at: Option<i64>,
    /// Unix time in milliseconds.
    pub updated_at: i64,
}

/// Work one bot asked another to do (spec 9.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Task {
    pub id: TaskId,
    pub crew_id: CrewId,
    pub requester_bot_id: BotId,
    pub assignee_bot_id: BotId,
    pub status: TaskStatus,
    /// Unix time in milliseconds.
    pub deadline_at: i64,
    /// Position in a chain of delegations: 1 for a task nobody else caused.
    pub hops: u32,
    /// First task of the chain this one belongs to; `null` for that first task.
    pub origin_task_id: Option<TaskId>,
    pub result: Option<String>,
    /// Unix time in milliseconds.
    pub created_at: i64,
    /// Unix time in milliseconds.
    pub updated_at: i64,
}

/// Deliveries still to go and those that gave up, for `system.status`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DeliveryBacklog {
    /// `pending` and `sending`.
    pub pending: u32,
    pub dead: u32,
}

/// The owner writes to a bot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MessagesSendParams {
    pub bot_id: BotId,
    pub body: String,
    /// Up to 10 files (spec 9.5).
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub attachments: Option<Vec<AttachmentUpload>>,
    /// A reply of the bot, or a message it got, in its chat (spec 9.3).
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub reply_to: Option<ChatItemId>,
}

/// Newest first. Page back with `before` set to the oldest id received.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MessagesListParams {
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub crew_id: Option<CrewId>,
    /// Messages sent to or by this bot.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub bot_id: Option<BotId>,
    /// Only messages older than this one.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub before: Option<MessageId>,
    /// 1 to 200; 50 when absent.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub limit: Option<u32>,
}

/// Most recently updated first.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DeliveriesListParams {
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub state: Option<DeliveryState>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub bot_id: Option<BotId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DeliveryIdParams {
    pub delivery_id: DeliveryId,
}

/// Newest first.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TasksListParams {
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub crew_id: Option<CrewId>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub status: Option<TaskStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_names_match_serde_and_parse_back() {
        for state in [
            DeliveryState::Pending,
            DeliveryState::Sending,
            DeliveryState::Sent,
            DeliveryState::Dead,
        ] {
            let json = serde_json::to_string(&state).expect("serialize");
            assert_eq!(json, format!("\"{}\"", state.as_str()));
            assert_eq!(state.as_str().parse(), Ok(state));
        }
        assert_eq!("needs_review".parse::<TaskStatus>().ok(), None);
        assert_eq!(MessageKind::Result.to_string(), "result");
    }
}
