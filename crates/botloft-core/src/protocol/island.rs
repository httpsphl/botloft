//! What the island asks the daemon (spec 31): the cards that wait for the
//! owner and what the crews did today.

use serde::{Deserialize, Serialize};

use super::{ApprovalCard, EditLines, QuestionCard};
use crate::ids::BotId;

/// Every request and question waiting for the owner, oldest first, as the
/// phone sees them (spec 31.5, 28.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Cards {
    pub approvals: Vec<ApprovalCard>,
    pub questions: Vec<QuestionCard>,
}

/// Params of `activity.today`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ActivityTodayParams {
    /// Where the day starts, in Unix ms: the app sends its local midnight.
    pub since: i64,
}

/// What every crew did since `since`, counted from the database without a
/// bot's turn (spec 31.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DayActivity {
    /// Tasks between bots that ended done.
    pub tasks_done: u32,
    /// Tasks that ended failed or expired.
    pub tasks_failed: u32,
    /// Messages one bot sent another.
    pub bot_messages: u32,
    /// Requests the owner allowed or denied.
    pub requests_answered: u32,
    /// Distinct files that edits counted in `lines` changed.
    pub files_edited: u32,
    /// Lines added and removed by the edits that finished well (spec 8.2).
    pub lines: EditLines,
    /// The bot that spent the most time in turns; `None` on a quiet day.
    pub busiest: Option<BusyBot>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BusyBot {
    pub bot_id: BotId,
    pub name: String,
    pub color: String,
    pub crew: String,
    /// Time in turns since `since`, in ms.
    pub work_ms: i64,
}
