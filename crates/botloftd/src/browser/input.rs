//! The owner's events on the page (spec 21.10): which ones make sense, and
//! each as the DevTools protocol takes it: mouse and wheel, keys with or
//! without text, and text pasted or composed.

use botloft_core::protocol::{BrowserInput, MouseAction, MouseButton, modifier};
use serde_json::{Value, json};

use super::keys;
use super::session::Session;
use super::{BrowserError, Viewport};

/// Longest text the owner sends at once, in characters.
const TEXT_MAX: usize = 10_000;
/// Longest key or code name.
const KEY_MAX: usize = 32;
/// Farthest one turn of the wheel scrolls, in pixels.
const WHEEL_MAX: f64 = 10_000.0;

/// Whether an event makes sense on a page of that size.
pub(super) fn fits(input: &BrowserInput, page: Viewport) -> bool {
    let on_page = |x: f64, y: f64| page.contains(x, y);
    let modifiers_ok = |modifiers: u32| modifiers <= 15;
    match input {
        BrowserInput::Mouse {
            x,
            y,
            buttons,
            clicks,
            modifiers,
            ..
        } => on_page(*x, *y) && *buttons <= 7 && *clicks <= 3 && modifiers_ok(*modifiers),
        BrowserInput::Wheel {
            x,
            y,
            dx,
            dy,
            modifiers,
        } => {
            on_page(*x, *y)
                && dx.abs() <= WHEEL_MAX
                && dy.abs() <= WHEEL_MAX
                && modifiers_ok(*modifiers)
        }
        BrowserInput::Key {
            key,
            code,
            modifiers,
        } => {
            !key.is_empty()
                && key.chars().count() <= KEY_MAX
                && code.len() <= KEY_MAX
                && modifiers_ok(*modifiers)
        }
        BrowserInput::Text { text } => !text.is_empty() && text.chars().count() <= TEXT_MAX,
    }
}

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
    fn events_off_the_page_or_too_big_are_refused() {
        let mouse = |x: f64, y: f64| BrowserInput::Mouse {
            action: MouseAction::Down,
            x,
            y,
            button: MouseButton::Left,
            buttons: 1,
            clicks: 1,
            modifiers: 0,
        };
        let fits = |input: &BrowserInput| super::fits(input, Viewport::default());
        assert!(fits(&mouse(640.0, 400.0)));
        assert!(fits(&mouse(1280.0, 800.0)));
        assert!(!fits(&mouse(-1.0, 10.0)));
        assert!(!fits(&mouse(f64::NAN, 10.0)));
        assert!(!fits(&mouse(10.0, 900.0)));
        // A taller page takes points further down.
        let tall = Viewport::fitting(640, 700);
        assert!(super::fits(&mouse(10.0, 900.0), tall));
        let long = BrowserInput::Text {
            text: "x".repeat(TEXT_MAX + 1),
        };
        assert!(!fits(&long));
        let empty = BrowserInput::Text {
            text: String::new(),
        };
        assert!(!fits(&empty));
        let key = BrowserInput::Key {
            key: "a".to_owned(),
            code: "KeyA".to_owned(),
            modifiers: 16,
        };
        assert!(!fits(&key));
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
