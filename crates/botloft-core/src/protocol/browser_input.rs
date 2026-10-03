//! The owner's hands in a bot's browser (spec 21.10): who controls it and
//! the mouse and keyboard events the owner sends to the page.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// Who uses a bot's browser right now.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum BrowserControl {
    #[default]
    Bot,
    /// The owner took it: the bot's tools wait.
    Owner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum MouseAction {
    Move,
    Down,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum MouseButton {
    None,
    Left,
    Middle,
    Right,
}

/// Alt, Ctrl, Meta and Shift in `modifiers`, as the DevTools protocol has
/// them.
pub mod modifier {
    pub const ALT: u32 = 1;
    pub const CTRL: u32 = 2;
    pub const META: u32 = 4;
    pub const SHIFT: u32 = 8;
}

/// One thing the owner did on the page. Points are in CSS pixels of the
/// page, like `BrowserAction`.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum BrowserInput {
    Mouse {
        action: MouseAction,
        x: f64,
        y: f64,
        button: MouseButton,
        /// The buttons held: 1 left, 2 right, 4 middle.
        buttons: u32,
        /// 2 for a double click.
        clicks: u32,
        modifiers: u32,
    },
    Wheel {
        x: f64,
        y: f64,
        dx: f64,
        dy: f64,
        modifiers: u32,
    },
    /// A key pressed and let go; `key` and `code` as the DOM names them.
    Key {
        key: String,
        code: String,
        modifiers: u32,
    },
    /// Text pasted, or composed with a dead key or an input method.
    Text { text: String },
}

impl std::fmt::Debug for BrowserInput {
    /// Never what was typed: the owner may be typing a password.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self {
            Self::Mouse { .. } => "mouse",
            Self::Wheel { .. } => "wheel",
            Self::Key { .. } => "key",
            Self::Text { .. } => "text",
        };
        write!(f, "BrowserInput::{kind}")
    }
}

/// `browser.take`, `browser.release`, `browser.reload`, `browser.newTab`
/// and `browser.window`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserControlParams {
    pub bot_id: BotId,
}

/// `browser.switchTab`: the tab that becomes the active one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserTabParams {
    pub bot_id: BotId,
    pub tab_id: String,
}

/// `browser.open`: the address the owner typed for the active tab.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserOpenParams {
    pub bot_id: BotId,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserInputParams {
    pub bot_id: BotId,
    pub input: BrowserInput,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_reads_as_the_app_sends_it_and_never_prints_the_text() {
        let input: BrowserInput = serde_json::from_value(serde_json::json!({
            "kind": "mouse", "action": "down", "x": 10.5, "y": 20.0,
            "button": "left", "buttons": 1, "clicks": 2, "modifiers": 0,
        }))
        .expect("mouse");
        assert!(matches!(
            input,
            BrowserInput::Mouse {
                action: MouseAction::Down,
                clicks: 2,
                ..
            }
        ));
        let secret = BrowserInput::Text {
            text: "hunter2".to_owned(),
        };
        assert_eq!(format!("{secret:?}"), "BrowserInput::text");
        let key = serde_json::to_value(BrowserInput::Key {
            key: "a".to_owned(),
            code: "KeyA".to_owned(),
            modifiers: modifier::CTRL,
        })
        .expect("key");
        assert_eq!(key["kind"], "key");
        assert_eq!(key["modifiers"], 2);
    }
}
