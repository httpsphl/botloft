//! What the owner let each bot see and do on their desktop (spec 24.2).

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, DesktopGrantId};

/// What a grant reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum DesktopScope {
    /// The windows of one program, by its executable.
    App,
    /// Every window, but what is never granted (spec 24.3).
    Desktop,
}

impl DesktopScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Desktop => "desktop",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "app" => Self::App,
            "desktop" => Self::Desktop,
            _ => return None,
        })
    }
}

/// What a grant lets the bot do: see, or see and act.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum DesktopLevel {
    /// List the windows, read them and take pictures of them.
    See,
    /// Also click, type, choose and scroll (spec 24.5).
    Act,
}

impl DesktopLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::See => "see",
            Self::Act => "act",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "see" => Self::See,
            "act" => Self::Act,
            _ => return None,
        })
    }
}

/// A grant of a bot on the owner's desktop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopGrant {
    pub id: DesktopGrantId,
    pub bot_id: BotId,
    pub scope: DesktopScope,
    /// The executable, for an `app` grant.
    pub app_path: Option<String>,
    /// The app's name for people ("Microsoft Excel"), for an `app` grant.
    pub app_name: Option<String>,
    pub level: DesktopLevel,
    /// The bot may use the owner's real mouse and keyboard (spec 24.7).
    pub real_input: bool,
    /// The bot may use it while the owner is away (spec 24.8).
    pub unattended: bool,
    /// When the owner accepted the risks screen (spec 24.10), Unix ms.
    pub accepted_risks_at: Option<i64>,
    /// Unix time in milliseconds.
    pub created_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopGrantsParams {
    pub bot_id: BotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopGrantIdParams {
    pub grant_id: DesktopGrantId,
}

/// `desktop.setOptions`: what the owner changes in a grant (spec 24.10).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopOptionsParams {
    pub grant_id: DesktopGrantId,
    /// The bot may use the real mouse and keyboard there (spec 24.7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub real_input: Option<bool>,
}

/// A bot's grants after one changed: the result of `desktop.revoke` and
/// the params of the `bot.desktop` notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotDesktop {
    pub bot_id: BotId,
    /// Oldest first.
    pub grants: Vec<DesktopGrant>,
}

/// The window a bot is using on the owner's desktop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopWindow {
    pub id: u64,
    pub title: String,
    /// The app's name for people.
    pub app: String,
}

/// What a bot did in a window (spec 24.5), for its panel, which words it
/// in the owner's language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum DesktopActionKind {
    Click,
    Type,
    Select,
    Scroll,
    /// Keys pressed with the real keyboard; `option` says which.
    Press,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopAction {
    pub kind: DesktopActionKind,
    /// The control's name as the window reads it; empty when it has none.
    pub target: String,
    /// The option chosen, for `select`.
    pub option: Option<String>,
}

/// What a bot does on the owner's desktop, for its panel (spec 24.9): the
/// params of `desktop.changed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopState {
    pub bot_id: BotId,
    /// The window it read or acted in last.
    pub window: Option<DesktopWindow>,
    /// What it did last; `null` after only reading.
    pub action: Option<DesktopAction>,
    /// When it last read or acted, Unix ms.
    pub at: Option<i64>,
    /// The owner stopped it (spec 24.9): its desktop tools refuse until
    /// they let it go on.
    pub stopped: bool,
}

/// A picture of the window a bot is using, for whoever watches its panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopFrame {
    pub bot_id: BotId,
    /// JPEG, base64.
    pub data: String,
    pub width: u32,
    pub height: u32,
}

/// The result of `desktop.watch`: the bot's state and, when it uses a
/// window, its picture now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopView {
    pub state: DesktopState,
    pub frame: Option<DesktopFrame>,
}

/// `desktop.watch`, `desktop.stop` and `desktop.resume`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DesktopBotParams {
    pub bot_id: BotId,
}
