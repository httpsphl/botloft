//! Routines (spec 20): a bot working on its own at set times.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, MessageId, RoutineId, RoutineRunId};

/// When a routine runs (spec 20.2), kept as structure rather than cron text
/// so the app can show and edit it without jargon.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum Schedule {
    /// On the given weekdays (1 = Monday … 7 = Sunday) at `time` (`HH:MM`),
    /// in the routine's time zone.
    Weekly { days: Vec<u8>, time: String },
    /// Every `minutes`, counted from the last scheduled time.
    Interval { minutes: u32 },
    /// A 5-field cron expression, in the routine's time zone.
    Cron { expr: String },
}

text_enum!(
    /// What happens when a routine's time comes while its last run is still
    /// waiting or working (spec 20.3).
    Overlap, "overlap" {
        /// Skip this time.
        Skip => "skip",
        /// Let one run wait behind the open one; skip the rest.
        Queue => "queue",
    }
);

text_enum!(
    /// What happens with times that passed while the daemon was not running
    /// (spec 20.4).
    Missed, "missed" {
        /// Run once now, for the latest of them.
        RunOnce => "run_once",
        /// Run none of them.
        Skip => "skip",
    }
);

text_enum!(
    /// Where a run is (spec 20.3).
    RunStatus, "run status" {
        /// Its message waits to be delivered or its turn to end.
        Queued => "queued",
        /// The turn that began with its message ended.
        Done => "done",
        /// Its message could not be delivered, or the turn ended in an error.
        Failed => "failed",
        /// It did not run; `reason` says why.
        Skipped => "skipped",
    }
);

text_enum!(
    /// Why a run was skipped.
    SkipReason, "skip reason" {
        /// The previous run was still open.
        Overlap => "overlap",
        /// The bot or its crew was paused.
        BotPaused => "bot_paused",
        /// The time passed while the daemon was not running.
        Missed => "missed",
    }
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Routine {
    pub id: RoutineId,
    pub bot_id: BotId,
    pub name: String,
    /// What the bot is asked to do, as the owner wrote it.
    pub prompt: String,
    pub schedule: Schedule,
    /// IANA name, like `America/Sao_Paulo`.
    pub timezone: String,
    pub overlap: Overlap,
    pub missed: Missed,
    pub enabled: bool,
    /// Unix ms of the next scheduled time; `null` while disabled.
    pub next_run_at: Option<i64>,
    /// The latest run, skipped ones included.
    pub last_run: Option<RoutineRun>,
    pub created_at: i64,
    pub updated_at: i64,
    pub archived_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RoutineRun {
    pub id: RoutineRunId,
    pub routine_id: RoutineId,
    /// The time it was for; the moment of the click for "run now".
    pub scheduled_for: i64,
    pub status: RunStatus,
    pub reason: Option<SkipReason>,
    /// For a `missed` skip: how many times passed.
    pub skipped_count: u32,
    /// The message it sent, unless skipped.
    pub message_id: Option<MessageId>,
    pub created_at: i64,
    pub finished_at: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RoutinesListParams {
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub bot_id: Option<BotId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RoutinesCreateParams {
    pub bot_id: BotId,
    pub name: String,
    pub prompt: String,
    pub schedule: Schedule,
    pub timezone: String,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub overlap: Option<Overlap>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub missed: Option<Missed>,
}

/// Fields left out stay as they are.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RoutinesUpdateParams {
    pub routine_id: RoutineId,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub name: Option<String>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub prompt: Option<String>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub schedule: Option<Schedule>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub timezone: Option<String>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub overlap: Option<Overlap>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub missed: Option<Missed>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RoutinesSetEnabledParams {
    pub routine_id: RoutineId,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RoutineIdParams {
    pub routine_id: RoutineId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct RoutinesRunsParams {
    pub routine_id: RoutineId,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub before: Option<RoutineRunId>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub limit: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedules_are_tagged_by_kind() {
        let weekly: Schedule =
            serde_json::from_str(r#"{"kind":"weekly","days":[1,2,3,4,5],"time":"09:00"}"#)
                .expect("weekly");
        assert_eq!(
            weekly,
            Schedule::Weekly {
                days: vec![1, 2, 3, 4, 5],
                time: "09:00".into()
            }
        );
        let every = serde_json::to_value(Schedule::Interval { minutes: 120 }).expect("json");
        assert_eq!(
            every,
            serde_json::json!({ "kind": "interval", "minutes": 120 })
        );
    }
}
