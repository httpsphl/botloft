//! The phone's Home (spec 28.15): the shortcuts the owner runs with a tap,
//! next to what waits for them, who works and what the crews did today.

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, RoutineId};

/// The most shortcuts the Home lists, so its message fits in one seal.
pub const PHONE_SHORTCUTS_MAX: usize = 40;

/// A routine the owner runs with a tap (spec 20.2), as the phone shows it.
/// The agent's name and color come from its line in the list (28.12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PhoneShortcut {
    pub routine_id: RoutineId,
    pub name: String,
    pub bot_id: BotId,
}
