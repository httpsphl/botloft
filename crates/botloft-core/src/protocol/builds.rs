//! What the Maker built in one go, and undoing it (spec 36.4).

use serde::{Deserialize, Serialize};

use crate::ids::{ApprovalId, BotId, BuildId, CrewId};

/// How long the owner may undo a build: a week.
pub const BUILD_UNDO_DAYS: i64 = 7;

/// A plan of the Maker that was built.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Build {
    pub id: BuildId,
    /// The Maker.
    pub bot_id: BotId,
    /// The request the owner allowed; `null` when the Maker built without
    /// asking (spec 36.4).
    pub approval_id: Option<ApprovalId>,
    pub title: String,
    /// The crews it made, in the plan's order.
    pub crew_ids: Vec<CrewId>,
    /// Unix time in milliseconds.
    pub applied_at: i64,
    /// Until when it can be undone, Unix time in milliseconds.
    pub undo_until: i64,
    /// Unix time in milliseconds; `null` while it stands.
    pub undone_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BuildIdParams {
    pub build_id: BuildId,
}
