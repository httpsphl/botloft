//! Each bot's browser (spec 21.7): where it is, what the bot does in it,
//! and the live frames the app shows while the owner watches.

use serde::{Deserialize, Serialize};

use super::{BrowserControl, LessonStep};
use crate::ids::BotId;

/// Where a bot's browser is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum BrowserStatus {
    /// No process; the bot's next browser tool starts one.
    Closed,
    Starting,
    Open,
    /// It could not start; `error` says why.
    Failed,
}

/// One of the browser's open tabs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserTab {
    /// Names the tab in `browser.switchTab`; means nothing else.
    pub id: String,
    /// Empty until the page has one.
    pub title: String,
    pub url: String,
    /// The tab the bot's tools and the owner's hands act on.
    pub active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserState {
    pub bot_id: BotId,
    pub status: BrowserStatus,
    /// The active tab's address.
    pub url: Option<String>,
    pub title: Option<String>,
    /// The active tab is loading a page.
    pub loading: bool,
    /// The open tabs, in the order they opened.
    pub tabs: Vec<BrowserTab>,
    /// Why it could not start, when `failed`.
    pub error: Option<String>,
    /// Who uses it now (spec 21.10).
    pub control: BrowserControl,
    /// Nobody uses it, so it rests: its pages stand still until the bot or
    /// the owner needs them (spec 21.2).
    pub resting: bool,
    /// What the bot asked the owner to do in it, while it waits.
    pub ask: Option<String>,
    /// The owner has it open in a window of its own, to sign in where a
    /// site refuses a browser a program drives (spec 21.11). It is
    /// `closed` meanwhile, and the bot's tools wait.
    pub window: bool,
    /// The steps of the lesson the owner is giving, while they teach the
    /// bot a task (spec 21.13); `null` when nobody teaches.
    #[serde(default)]
    pub lesson: Option<Vec<LessonStep>>,
    /// Unix time in milliseconds.
    pub updated_at: i64,
}

impl BrowserState {
    pub fn closed(bot_id: BotId, now: i64) -> Self {
        Self {
            bot_id,
            status: BrowserStatus::Closed,
            url: None,
            title: None,
            loading: false,
            tabs: Vec::new(),
            error: None,
            control: BrowserControl::Bot,
            resting: false,
            ask: None,
            window: false,
            lesson: None,
            updated_at: now,
        }
    }
}

/// One picture of the active tab, sent only to the connection watching it.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserFrame {
    pub bot_id: BotId,
    /// A JPEG, base64.
    pub data: String,
    /// Size of the page it shows, in CSS pixels: the space of
    /// `BrowserAction` points.
    pub width: u32,
    pub height: u32,
}

impl std::fmt::Debug for BrowserFrame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BrowserFrame")
            .field("bot_id", &self.bot_id)
            .field("data", &format_args!("<{} base64 chars>", self.data.len()))
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum BrowserActionKind {
    Open,
    Click,
    Type,
    Select,
    Press,
    Scroll,
    Back,
}

/// Something the bot did in its browser, for the cursor in the app.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserAction {
    pub bot_id: BotId,
    pub kind: BrowserActionKind,
    /// Where on the page, in CSS pixels, when the action has a point.
    pub x: Option<f64>,
    pub y: Option<f64>,
    /// The element's name, the key or the site; never the text typed.
    pub label: Option<String>,
    /// Unix time in milliseconds.
    pub at: i64,
}

/// What `browser.watch` answers: the state and the latest frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserView {
    pub state: BrowserState,
    pub frame: Option<BrowserFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserWatchParams {
    pub bot_id: BotId,
}

/// `browser.resize`: the room the app's panel has for the page, in the
/// app's pixels. The page takes its shape (spec 21.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserResizeParams {
    pub bot_id: BotId,
    pub width: u32,
    pub height: u32,
    /// The screen's pixels per pixel of the app, in percent (its
    /// `devicePixelRatio` times 100). The daemon ignores it: the page is
    /// always drawn in its own pixels, because a sharper page made clicks
    /// land at the wrong point (spec 21.3, 19). Absent, 100.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub scale: Option<u32>,
}
