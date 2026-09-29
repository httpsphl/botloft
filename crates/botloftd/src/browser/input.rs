//! The owner's events on the page (spec 21.10), as the DevTools protocol
//! takes them: mouse and wheel, keys with or without text, and text pasted
//! or composed.

use botloft_core::protocol::{BrowserInput, MouseAction, MouseButton, modifier};
use serde_json::{Value, json};

use super::BrowserError;
use super::keys;
use super::session::Session;

impl Session {
    /// Sends one of the owner's events to the active tab.
    pub(super) async fn owner_input(&self, input: &BrowserInput) -> Result<(), BrowserError> {
        let (session, _) = self.page()?;
        for (method, params) in calls(input) {
            self.cdp.call(Some(&session), method, params).await?;
        }
        Ok(())
    }
}

/// The protocol calls for one event, in order.
fn calls(input: &BrowserInput) -> Vec<(&'static str, Value)> {
    match input {
        BrowserInput::Mouse {
            action,
            x,
            y,
            button,
            buttons,
            clicks,
            modifiers,
        } => {
            let kind = match action {
                MouseAction::Move => "mouseMoved",
                MouseAction::Down => "mousePressed",
                MouseAction::Up => "mouseReleased",
            };
            // A move names the button held, so a drag stays a drag.
            let button = match action {
                MouseAction::Move => held(*buttons),
                MouseAction::Down | MouseAction::Up => name(*button),
            };
            vec![(
                "Input.dispatchMouseEvent",
                json!({
                    "type": kind, "x": x, "y": y, "button": button, "buttons": buttons,
                    "clickCount": clicks, "modifiers": modifiers,
                }),
            )]
        }
        BrowserInput::Wheel {
            x,
            y,
            dx,
            dy,
            modifiers,
        } => vec![(
            "Input.dispatchMouseEvent",
            json!({
                "type": "mouseWheel", "x": x, "y": y, "deltaX": dx, "deltaY": dy,
                "modifiers": modifiers,
            }),
        )],
        BrowserInput::Text { text } => vec![("Input.insertText", json!({ "text": text }))],
        BrowserInput::Key {
            key,
            code,
            modifiers,
        } => press(key, code, *modifiers),
    }
}

fn name(button: MouseButton) -> &'static str {
    match button {
        MouseButton::None => "none",
        MouseButton::Left => "left",
        MouseButton::Middle => "middle",
        MouseButton::Right => "right",
    }
}

fn held(buttons: u32) -> &'static str {
    if buttons & 1 != 0 {
        "left"
    } else if buttons & 2 != 0 {
        "right"
    } else if buttons & 4 != 0 {
        "middle"
    } else {
        "none"
    }
}

/// A key going down and up. It types its text unless Ctrl or Meta is held;
/// Ctrl with Alt is AltGr, which types too. Without text, Chromium runs the
/// shortcut from the virtual key.
fn press(key: &str, code: &str, modifiers: u32) -> Vec<(&'static str, Value)> {
    let shortcut = modifiers & (modifier::CTRL | modifier::META) != 0;
    let alt_gr = modifiers & modifier::CTRL != 0 && modifiers & modifier::ALT != 0;
    let named = keys::named(key);
    let text = match named {
        Some(named) => named
            .text
            .filter(|_| modifiers & !modifier::SHIFT == 0)
            .map(str::to_owned),
        None if key.chars().count() == 1 && (!shortcut || alt_gr) => Some(key.to_owned()),
        None => None,
    };
    let virtual_key = named.map_or_else(|| keys::virtual_key(code, key), |named| named.virtual_key);
    let event = |kind: &str| {
        json!({
            "type": kind, "key": key, "code": code, "modifiers": modifiers,
            "windowsVirtualKeyCode": virtual_key, "nativeVirtualKeyCode": virtual_key,
        })
    };
    let mut down = event(if text.is_some() {
        "keyDown"
    } else {
        "rawKeyDown"
    });
    if let Some(text) = text {
        down["text"] = json!(text);
        down["unmodifiedText"] = json!(text);
    }
    vec![
        ("Input.dispatchKeyEvent", down),
        ("Input.dispatchKeyEvent", event("keyUp")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: &str, code: &str, modifiers: u32) -> Vec<Value> {
        calls(&BrowserInput::Key {
            key: key.to_owned(),
            code: code.to_owned(),
            modifiers,
        })
        .into_iter()
        .map(|(_, params)| params)
        .collect()
    }

    #[test]
    fn letters_type_and_shortcuts_run_from_the_virtual_key() {
        let typed = key("ã", "KeyA", 0);
        assert_eq!(typed[0]["type"], "keyDown");
        assert_eq!(typed[0]["text"], "ã");
        assert_eq!(typed[1]["type"], "keyUp");

        let all = key("a", "KeyA", modifier::CTRL);
        assert_eq!(all[0]["type"], "rawKeyDown");
        assert_eq!(all[0]["windowsVirtualKeyCode"], 65);
        assert_eq!(all[0]["modifiers"], 2);
        assert!(all[0].get("text").is_none());

        let alt_gr = key("/", "IntlRo", modifier::CTRL | modifier::ALT);
        assert_eq!(alt_gr[0]["text"], "/");

        let capital = key("A", "KeyA", modifier::SHIFT);
        assert_eq!(capital[0]["text"], "A");
    }

    #[test]
    fn named_keys_keep_their_text_only_without_other_modifiers() {
        let enter = key("Enter", "Enter", 0);
        assert_eq!(enter[0]["type"], "keyDown");
        assert_eq!(enter[0]["text"], "\r");
        let ctrl_enter = key("Enter", "Enter", modifier::CTRL);
        assert_eq!(ctrl_enter[0]["type"], "rawKeyDown");
        let back_tab = key("Tab", "Tab", modifier::SHIFT);
        assert_eq!(back_tab[0]["windowsVirtualKeyCode"], 9);
        assert_eq!(back_tab[0]["modifiers"], 8);
        let word = key("Backspace", "Backspace", modifier::CTRL);
        assert_eq!(word[0]["windowsVirtualKeyCode"], 8);
    }

    #[test]
    fn a_drag_names_the_button_held() {
        let drag = calls(&BrowserInput::Mouse {
            action: MouseAction::Move,
            x: 10.0,
            y: 20.0,
            button: MouseButton::None,
            buttons: 1,
            clicks: 0,
            modifiers: 0,
        });
        assert_eq!(drag[0].1["type"], "mouseMoved");
        assert_eq!(drag[0].1["button"], "left");
        let wheel = calls(&BrowserInput::Wheel {
            x: 1.0,
            y: 2.0,
            dx: 0.0,
            dy: 120.0,
            modifiers: 0,
        });
        assert_eq!(wheel[0].1["deltaY"], 120.0);
    }
}
