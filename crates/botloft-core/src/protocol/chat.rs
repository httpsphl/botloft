//! A bot's chat (spec 8) and the approvals shown in it (spec 10.1).

use serde::{Deserialize, Serialize};

use super::Message;
use crate::ids::{ApprovalId, BotId, ChatItemId};

text_enum!(
    /// Where a tool call the bot made stands.
    ToolStatus, "tool status" {
        Running => "running",
        Done => "done",
        Failed => "failed",
    }
);

text_enum!(
    ApprovalStatus, "approval status" {
        /// Waiting for the owner.
        Pending => "pending",
        Allowed => "allowed",
        Denied => "denied",
        /// Nobody answered in time, or the bot's process ended first.
        Expired => "expired",
    }
);

text_enum!(
    NoticeLevel, "notice level" {
        Info => "info",
        Warning => "warning",
        Error => "error",
    }
);

text_enum!(
    /// What a notice is about, so the app can say it in the owner's language.
    NoticeCode, "notice code" {
        /// Claude Code is not signed in or the account cannot be used.
        SignedOut => "signed_out",
        /// The account reached its usage limit.
        UsageLimit => "usage_limit",
        /// The turn failed; `text` holds Claude Code's detail or error code.
        TurnFailed => "turn_failed",
        /// The bot's model does not exist or the account cannot use it.
        ModelUnavailable => "model_unavailable",
    }
);

text_enum!(
    /// What the conversation-list line shows; the app words it.
    ActivityKind, "activity kind" {
        /// The owner's message; `text` is its body.
        Owner => "owner",
        /// A message from another bot or the daemon; `text` is its body.
        Message => "message",
        Reply => "reply",
        /// `text` is the tool and its summary.
        Tool => "tool",
        /// A permission request; `text` is the tool.
        Approval => "approval",
        Notice => "notice",
    }
);

/// One entry of a bot's chat.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChatItem {
    pub id: ChatItemId,
    pub bot_id: BotId,
    pub body: ChatBody,
    /// Unix time in milliseconds.
    pub created_at: i64,
    /// Unix time in milliseconds; changes when a tool ends or an approval is
    /// answered.
    pub updated_at: i64,
}

/// What a chat item holds, by `kind`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum ChatBody {
    /// A message to the bot: from the owner, another bot or the daemon.
    Inbound(InboundItem),
    /// Text the bot wrote, markdown.
    Reply(ReplyItem),
    Tool(ToolItem),
    Approval(ApprovalItem),
    /// The end of a turn.
    Turn(TurnItem),
    Notice(NoticeItem),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct InboundItem {
    pub message: Message,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ReplyItem {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ToolItem {
    pub tool_use_id: String,
    pub name: String,
    /// One line about what the call does, e.g. the command or the file.
    pub summary: String,
    /// The call's input as JSON, cut at 4 KB.
    pub input: String,
    pub status: ToolStatus,
    /// The start of what the tool returned, cut at 8 KB.
    pub output: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ApprovalItem {
    pub approval_id: ApprovalId,
    pub tool_name: String,
    pub summary: String,
    pub input: String,
    pub status: ApprovalStatus,
    /// What the owner wrote with a denial.
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TurnItem {
    pub duration_ms: u64,
    /// As Claude Code reports it; `null` when it did not.
    pub cost_usd: Option<f64>,
    /// Why the turn failed, e.g. `rate_limit`; `null` when it worked.
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct NoticeItem {
    pub level: NoticeLevel,
    /// Absent on notices stored before codes existed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub code: Option<NoticeCode>,
    /// English text, or the detail for [`NoticeCode::TurnFailed`].
    pub text: String,
}

/// A permission request (spec 10.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Approval {
    pub id: ApprovalId,
    pub bot_id: BotId,
    pub tool_name: String,
    pub summary: String,
    pub input: String,
    pub status: ApprovalStatus,
    pub note: Option<String>,
    /// Unix time in milliseconds.
    pub created_at: i64,
    /// Unix time in milliseconds; `null` while pending.
    pub answered_at: Option<i64>,
}

/// Params of the `chat.item` notification: an item that is new or changed.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChatItemChanged {
    pub item: ChatItem,
    /// The bot's new conversation-list line; `null` when it did not change.
    pub activity: Option<Activity>,
}

/// Text the bot is writing right now, in order (spec 8.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChatDelta {
    pub bot_id: BotId,
    pub text: String,
}

/// Newest first. Page back with `before` set to the oldest id received.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChatHistoryParams {
    pub bot_id: BotId,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub before: Option<ChatItemId>,
    /// 1 to 200; 50 when absent.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ApprovalsAnswerParams {
    pub approval_id: ApprovalId,
    pub allow: bool,
    /// Passed to the bot with a denial.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub note: Option<String>,
}

/// The last thing in a bot's chat, for the conversation list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Activity {
    pub kind: ActivityKind,
    /// One line, at most 120 characters, without wording of its own.
    pub text: String,
    /// Unix time in milliseconds.
    pub at: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_carry_their_kind() {
        let body = ChatBody::Reply(ReplyItem {
            text: "done".into(),
        });
        let json = serde_json::to_value(&body).expect("serialize");
        assert_eq!(json, serde_json::json!({ "kind": "reply", "text": "done" }));
        let tool = ChatBody::Tool(ToolItem {
            tool_use_id: "toolu_1".into(),
            name: "Bash".into(),
            summary: "npm test".into(),
            input: "{}".into(),
            status: ToolStatus::Running,
            output: None,
        });
        let json = serde_json::to_value(&tool).expect("serialize");
        assert_eq!(json["kind"], "tool");
        assert_eq!(json["toolUseId"], "toolu_1");
        assert_eq!(serde_json::from_value::<ChatBody>(json).ok(), Some(tool));
    }
}
