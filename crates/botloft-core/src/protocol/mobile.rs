//! The phone (spec 28): the `mobile.*` methods the app calls, and the sealed
//! messages that cross the relay between the computer and the phone (28.3).

use serde::{Deserialize, Serialize};

use crate::ids::{ApprovalId, BotId, ChatItemId, QuestionId};
use crate::protocol::{Activity, ApprovalStatus, BotState, NoticeLevel, QuestionStatus};

/// Whether the computer's connection to the relay is up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum MobileRelay {
    /// No phone is connected and none is being: nothing is open.
    Off,
    Connecting,
    Connected,
}

/// A phone the owner connected.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobilePhone {
    pub id: String,
    pub name: String,
    /// Unix time in milliseconds.
    pub paired_at: i64,
    /// When the phone last sent anything, in Unix milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub last_seen_at: Option<i64>,
    pub online: bool,
}

/// A phone that opened the QR code and waits for the owner to compare the
/// code and accept it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobileJoined {
    pub name: String,
    /// Six digits, the same on the phone's screen.
    pub code: String,
}

/// A QR code on screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobilePairing {
    pub pair_id: String,
    /// Unix time in milliseconds the code stops working.
    pub expires_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub joined: Option<MobileJoined>,
}

/// `mobile.status`, and the `mobile.changed` notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobileStatus {
    pub relay: MobileRelay,
    pub phones: Vec<MobilePhone>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub pending: Option<MobilePairing>,
}

/// `mobile.pair_start`: what the QR code carries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobilePairStarted {
    pub pair_id: String,
    /// The address the QR code holds, `#` and all.
    pub url: String,
    /// Seconds the code works.
    pub expires_in: u32,
}

/// `mobile.pair_cancel`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobilePairIdParams {
    pub pair_id: String,
}

/// `mobile.pair_confirm`: the owner compared the codes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobilePairConfirmParams {
    pub pair_id: String,
    pub accept: bool,
}

/// `mobile.revoke`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobileRevokeParams {
    pub phone_id: String,
}

/// The `mobile.pair_request` notification: a phone opened the code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct MobilePairRequest {
    pub pair_id: String,
    pub name: String,
    pub code: String,
}

/// The bot that asks, as the phone shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CardBot {
    pub name: String,
    pub color: String,
}

/// A request waiting for the owner, as the phone shows it (spec 28.5). It
/// carries what the owner needs to decide, and nothing else of the bot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ApprovalCard {
    pub approval_id: ApprovalId,
    pub bot: CardBot,
    pub crew: String,
    /// Unix time in milliseconds.
    pub created_at: i64,
    pub tool_name: String,
    pub summary: String,
    /// What the bot says the command is for; the bot wrote it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub explanation: Option<String>,
    /// The command, the plan or the request, whole unless `cut`.
    pub text: String,
    /// `text` is not the whole request: the phone may only deny it.
    pub cut: bool,
    /// The request is of a kind that is answered at the computer: the phone
    /// may only deny it.
    pub at_computer: bool,
}

/// A question waiting for the owner, as the phone shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct QuestionCard {
    pub question_id: QuestionId,
    pub bot: CardBot,
    pub crew: String,
    pub created_at: i64,
    pub text: String,
    pub options: Vec<String>,
}

/// A bot in the phone's list of conversations (spec 28.12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ChatLine {
    pub bot_id: BotId,
    pub name: String,
    pub color: String,
    pub crew: String,
    pub state: BotState,
    /// False for a bot that can only be written to at the computer
    /// (`bypass_permissions`, spec 28.12).
    pub can_send: bool,
    /// When the bot last finished a reply, in Unix milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub last_reply_at: Option<i64>,
    /// The last thing in the chat, on one line.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub last: Option<Activity>,
}

/// One thing in a conversation, as the phone shows it (spec 28.12): the
/// text, and never the output of a tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum PhoneItem {
    /// The owner wrote it.
    You {
        id: ChatItemId,
        at: i64,
        text: String,
        cut: bool,
    },
    /// Another bot, a routine or the daemon wrote it to this bot; `from` is
    /// the name.
    BotMessage {
        id: ChatItemId,
        at: i64,
        from: String,
        text: String,
        cut: bool,
    },
    /// The bot wrote it, markdown.
    Reply {
        id: ChatItemId,
        at: i64,
        text: String,
        cut: bool,
    },
    Tool {
        id: ChatItemId,
        at: i64,
        summary: String,
    },
    Approval {
        id: ChatItemId,
        at: i64,
        approval_id: ApprovalId,
        summary: String,
        status: ApprovalStatus,
    },
    Question {
        id: ChatItemId,
        at: i64,
        question_id: QuestionId,
        text: String,
        status: QuestionStatus,
    },
    /// A turn that failed.
    Failed {
        id: ChatItemId,
        at: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(test, ts(optional))]
        error: Option<String>,
    },
    Notice {
        id: ChatItemId,
        at: i64,
        level: NoticeLevel,
        text: String,
    },
}

/// What the computer sends the phone, sealed (spec 28.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum ToPhone {
    /// What is waiting right now: the answer to `sync`.
    #[serde(rename = "snapshot")]
    Snapshot {
        approvals: Vec<ApprovalCard>,
        questions: Vec<QuestionCard>,
    },
    #[serde(rename = "approval.open")]
    ApprovalOpen { card: ApprovalCard },
    /// It was answered (at the computer or here) or ran out.
    #[serde(rename = "approval.closed")]
    ApprovalClosed {
        approval_id: ApprovalId,
        status: ApprovalStatus,
    },
    #[serde(rename = "question.open")]
    QuestionOpen { card: QuestionCard },
    #[serde(rename = "question.closed")]
    QuestionClosed {
        question_id: QuestionId,
        status: QuestionStatus,
    },
    /// The list of conversations, in parts when it is long (spec 28.12).
    #[serde(rename = "chats")]
    Chats { bots: Vec<ChatLine>, first: bool },
    /// A part of a page of history, oldest first; `done` on the last part,
    /// `more` when there are older pages.
    #[serde(rename = "history")]
    History {
        req: u32,
        bot_id: BotId,
        items: Vec<PhoneItem>,
        more: bool,
        done: bool,
    },
    /// What came of a `send`.
    #[serde(rename = "sent")]
    Sent {
        client_id: String,
        ok: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        #[cfg_attr(test, ts(optional))]
        reason: Option<String>,
    },
    /// A new or changed item of the open conversation.
    #[serde(rename = "item")]
    Item { bot_id: BotId, item: PhoneItem },
    /// The whole reply the bot is writing now; empty when it is done.
    #[serde(rename = "live")]
    Live { bot_id: BotId, text: String },
    /// The open conversation's bot started or stopped working.
    #[serde(rename = "state")]
    State { bot_id: BotId, state: BotState },
    /// The line of a bot in the list changed.
    #[serde(rename = "line")]
    Line { bot: ChatLine },
}

/// What the phone sends the computer, sealed (spec 28.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "t", rename_all_fields = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum FromPhone {
    /// Asks for the `snapshot`.
    #[serde(rename = "sync")]
    Sync,
    #[serde(rename = "approval.answer")]
    ApprovalAnswer {
        approval_id: ApprovalId,
        allow: bool,
        #[serde(default)]
        #[cfg_attr(test, ts(optional))]
        note: Option<String>,
    },
    #[serde(rename = "question.answer")]
    QuestionAnswer {
        question_id: QuestionId,
        answer: String,
    },
    #[serde(rename = "question.dismiss")]
    QuestionDismiss { question_id: QuestionId },
    /// Asks for the list of conversations (spec 28.12).
    #[serde(rename = "chats")]
    Chats,
    /// A page of a conversation: the newest, or those before `before`.
    #[serde(rename = "history")]
    History {
        req: u32,
        bot_id: BotId,
        #[serde(default)]
        #[cfg_attr(test, ts(optional))]
        before: Option<ChatItemId>,
    },
    /// The owner writes to a bot.
    #[serde(rename = "send")]
    Send {
        client_id: String,
        bot_id: BotId,
        text: String,
    },
    /// Which conversation is open, if any.
    #[serde(rename = "watch")]
    Watch {
        #[serde(default)]
        #[cfg_attr(test, ts(optional))]
        bot_id: Option<BotId>,
    },
}

/// The biggest text of one chat item the phone gets, in bytes (spec 28.12).
pub const ITEM_TEXT_MAX: usize = 6 * 1024;
/// The biggest `text` a card carries, in bytes (spec 28.5).
pub const CARD_TEXT_MAX: usize = 8 * 1024;
/// The biggest sealed message either way, in bytes of JSON.
pub const SEALED_MAX: usize = 16 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sealed_messages_have_the_shape_the_phone_reads() {
        let sync = serde_json::to_value(FromPhone::Sync).expect("json");
        assert_eq!(sync, serde_json::json!({ "t": "sync" }));
        let id = QuestionId::generate();
        let dismiss = serde_json::to_value(FromPhone::QuestionDismiss {
            question_id: id.clone(),
        })
        .expect("json");
        assert_eq!(
            dismiss,
            serde_json::json!({ "t": "question.dismiss", "questionId": id })
        );
        let back: FromPhone = serde_json::from_value(serde_json::json!({
            "t": "approval.answer", "approvalId": ApprovalId::generate(), "allow": true,
        }))
        .expect("an answer without a note");
        assert!(matches!(
            back,
            FromPhone::ApprovalAnswer {
                allow: true,
                note: None,
                ..
            }
        ));
        let closed = serde_json::to_value(ToPhone::QuestionClosed {
            question_id: id,
            status: QuestionStatus::Answered,
        })
        .expect("json");
        assert_eq!(closed["t"], "question.closed");
        assert_eq!(closed["status"], "answered");
    }

    #[test]
    fn the_conversation_messages_have_the_shape_the_phone_reads() {
        let bot = BotId::generate();
        // What the phone asks for.
        let history: FromPhone = serde_json::from_value(serde_json::json!({
            "t": "history", "req": 3, "botId": bot,
        }))
        .expect("a first page has no `before`");
        assert!(matches!(
            history,
            FromPhone::History {
                req: 3,
                before: None,
                ..
            }
        ));
        let watch: FromPhone =
            serde_json::from_value(serde_json::json!({ "t": "watch" })).expect("nothing open");
        assert!(matches!(watch, FromPhone::Watch { bot_id: None }));
        let send = serde_json::to_value(FromPhone::Send {
            client_id: "c1".into(),
            bot_id: bot.clone(),
            text: "oi".into(),
        })
        .expect("json");
        assert_eq!(
            send,
            serde_json::json!({ "t": "send", "clientId": "c1", "botId": bot, "text": "oi" })
        );
        // What it gets: an item is told by its `kind`, in camelCase.
        let item = PhoneItem::Approval {
            id: ChatItemId::generate(),
            at: 5,
            approval_id: ApprovalId::generate(),
            summary: "git status".into(),
            status: ApprovalStatus::Allowed,
        };
        let shown = serde_json::to_value(ToPhone::Item { bot_id: bot, item }).expect("json");
        assert_eq!(shown["t"], "item");
        assert_eq!(shown["item"]["kind"], "approval");
        assert_eq!(shown["item"]["status"], "allowed");
        assert!(shown["item"]["approvalId"].is_string());
        let failed = serde_json::to_value(PhoneItem::Failed {
            id: ChatItemId::generate(),
            at: 1,
            error: None,
        })
        .expect("json");
        assert!(failed.get("error").is_none(), "an absent error is left out");
    }
}
