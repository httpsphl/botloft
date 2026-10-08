//! The phone (spec 28): the `mobile.*` methods the app calls, and the sealed
//! messages that cross the relay between the computer and the phone (28.3).

use serde::{Deserialize, Serialize};

use crate::ids::{ApprovalId, QuestionId};
use crate::protocol::{ApprovalStatus, QuestionStatus};

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
}

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
}
