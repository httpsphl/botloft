//! `terminal.*` methods and the `terminal.data` notification (spec 8).

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// Starts streaming a bot's terminal. With the `generation` and `offset` the
/// client last saw, only the missing bytes are replayed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TerminalAttachParams {
    pub bot_id: BotId,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub generation: Option<u64>,
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub offset: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TerminalAttachResult {
    pub generation: u64,
    /// Offset of the first byte the following `terminal.data` carries.
    pub offset: u64,
    /// True when the client must clear its screen before writing the replay.
    pub reset: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TerminalWriteParams {
    pub bot_id: BotId,
    /// Raw bytes, base64.
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TerminalResizeParams {
    pub bot_id: BotId,
    pub cols: u16,
    pub rows: u16,
}

/// Terminal output. `offset` counts bytes since the start of `generation`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct TerminalData {
    pub bot_id: BotId,
    pub generation: u64,
    pub offset: u64,
    /// Raw bytes, base64. May end in the middle of a UTF-8 sequence.
    pub data: String,
}
