//! Questions a bot asks the owner without waiting (spec 23), with the
//! params of the `questions.*` methods.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, CrewId, QuestionId};

/// Longest question, in characters (spec 23.2).
pub const QUESTION_MAX_CHARS: usize = 2_000;
/// Longest ready answer, in characters.
pub const QUESTION_OPTION_MAX_CHARS: usize = 100;
/// Fewest and most ready answers a question may offer.
pub const QUESTION_OPTIONS_MIN: usize = 2;
pub const QUESTION_OPTIONS_MAX: usize = 5;
/// Most questions one bot may have open at once.
pub const OPEN_QUESTIONS_MAX: usize = 5;

text_enum!(
    QuestionStatus, "question status" {
        /// Waiting for the owner.
        Open => "open",
        Answered => "answered",
        /// Closed by the owner without an answer; the bot is not told.
        Dismissed => "dismissed",
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Question {
    pub id: QuestionId,
    pub crew_id: CrewId,
    pub bot_id: BotId,
    /// What the bot asks, markdown.
    pub text: String,
    /// Ready answers the owner may pick; empty for a free answer only.
    pub options: Vec<String>,
    pub status: QuestionStatus,
    /// What the owner answered; `null` unless `answered`.
    pub answer: Option<String>,
    /// Unix time in milliseconds.
    pub created_at: i64,
    /// Unix time in milliseconds the owner answered or dismissed it.
    pub answered_at: Option<i64>,
}

/// Newest first.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct QuestionsListParams {
    /// `open` when absent.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub status: Option<QuestionStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct QuestionsAnswerParams {
    pub question_id: QuestionId,
    pub answer: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct QuestionIdParams {
    pub question_id: QuestionId,
}
