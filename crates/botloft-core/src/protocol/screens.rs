//! The design area (spec 22): the HTML screens a bot made, and the drafts
//! of the ones it is writing right now.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// The device a screen is drawn for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum ScreenDevice {
    Desktop,
    Tablet,
    Mobile,
}

impl ScreenDevice {
    pub fn parse(text: &str) -> Option<Self> {
        match text.trim().to_ascii_lowercase().as_str() {
            "desktop" | "computer" => Some(Self::Desktop),
            "tablet" => Some(Self::Tablet),
            "mobile" | "phone" => Some(Self::Mobile),
            _ => None,
        }
    }
}

/// An HTML file the bot made, with where the app loads it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Screen {
    /// Absolute path.
    pub path: String,
    pub name: String,
    /// As in `BotFile`.
    pub folder: String,
    /// Unix time in milliseconds; the draft's time for a file not written yet.
    pub modified_at: i64,
    /// Where the app loads it (spec 22.2); changes when the file does.
    pub url: String,
    /// What the file asks for with `<meta name="botloft-device">`.
    pub device: Option<ScreenDevice>,
    /// The bot is writing it now.
    pub writing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ScreensListParams {
    pub bot_id: BotId,
}

/// A new version of a screen the bot is writing (spec 22.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ScreenDraft {
    pub bot_id: BotId,
    /// Absolute path of the file being written.
    pub path: String,
    /// Where to load this version.
    pub url: String,
    /// Counts up with each version of this draft.
    pub rev: u64,
    /// The write ended: the file on disk is the screen again.
    pub done: bool,
}
