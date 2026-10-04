//! Teaching a bot a task (spec 21.13): while the owner has the bot's
//! browser in their hands, what they do is kept as steps by what it means
//! (the page, the button, the field), never by what they typed.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// Most steps one lesson keeps; later ones are left out.
pub const LESSON_STEPS_MAX: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum LessonStepKind {
    /// A web address in the active tab: where the lesson began, or one
    /// the owner typed.
    Open,
    /// A click on something with a role (a button, a link, a field...).
    Click,
    /// Typing in a field. What was typed is never kept.
    Type,
    /// A key that does something: Enter, Escape, Tab.
    Press,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LessonStep {
    pub kind: LessonStepKind,
    /// The address for `open`, the element's name for `click` and `type`,
    /// the key for `press`; at most 80 characters.
    pub label: String,
    /// The element's role (`button`, `link`, `textbox`...), when it has one.
    pub role: Option<String>,
    /// Typing in a password field: the bot asks the owner to type it.
    pub secret: bool,
}

/// Starts or ends a lesson in the browser the connection has in its hands.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserTeachParams {
    pub bot_id: BotId,
    pub on: bool,
}
