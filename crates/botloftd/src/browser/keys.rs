//! The keys `browser_press` knows, as `Input.dispatchKeyEvent` wants them,
//! and the Windows virtual keys of the ones the owner presses (spec 21.10).

/// A key: its DOM name, its code, the Windows virtual key and the text it
/// types, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub key: &'static str,
    pub code: &'static str,
    pub virtual_key: u32,
    pub text: Option<&'static str>,
}

const KEYS: [Key; 15] = [
    key("Enter", "Enter", 13, Some("\r")),
    key("Tab", "Tab", 9, None),
    key("Escape", "Escape", 27, None),
    key("Backspace", "Backspace", 8, None),
    key("Delete", "Delete", 46, None),
    key(" ", "Space", 32, Some(" ")),
    key("ArrowUp", "ArrowUp", 38, None),
    key("ArrowDown", "ArrowDown", 40, None),
    key("ArrowLeft", "ArrowLeft", 37, None),
    key("ArrowRight", "ArrowRight", 39, None),
    key("PageUp", "PageUp", 33, None),
    key("PageDown", "PageDown", 34, None),
    key("Home", "Home", 36, None),
    key("End", "End", 35, None),
    key("Insert", "Insert", 45, None),
];

const fn key(
    key: &'static str,
    code: &'static str,
    virtual_key: u32,
    text: Option<&'static str>,
) -> Key {
    Key {
        key,
        code,
        virtual_key,
        text,
    }
}

/// The key named `name`, ignoring case: `enter`, `Space`, `arrowdown`,
/// `Esc`, `Up`.
pub fn find(name: &str) -> Option<Key> {
    let name = name.trim().to_ascii_lowercase();
    let name = match name.as_str() {
        "esc" => "escape",
        "return" => "enter",
        "up" | "down" | "left" | "right" => {
            return KEYS
                .iter()
                .find(|key| key.key.eq_ignore_ascii_case(&format!("arrow{name}")))
                .copied();
        }
        "space" | " " => return Some(KEYS[5]),
        other => other,
    };
    KEYS.iter()
        .find(|key| key.key.eq_ignore_ascii_case(name))
        .copied()
}

/// The key the DOM calls exactly `key`, like `Enter` or `ArrowUp`.
pub fn named(key: &str) -> Option<Key> {
    KEYS.iter().find(|known| known.key == key).copied()
}

/// Punctuation keys by their DOM `code`, on a US layout.
const PUNCTUATION: [(&str, u32); 17] = [
    ("Semicolon", 186),
    ("Equal", 187),
    ("Comma", 188),
    ("Minus", 189),
    ("Period", 190),
    ("Slash", 191),
    ("Backquote", 192),
    ("IntlRo", 193),
    ("BracketLeft", 219),
    ("Backslash", 220),
    ("BracketRight", 221),
    ("Quote", 222),
    ("IntlBackslash", 226),
    ("NumpadMultiply", 106),
    ("NumpadAdd", 107),
    ("NumpadSubtract", 109),
    ("NumpadDivide", 111),
];

/// The Windows virtual key of a key the owner pressed, from its DOM `code`
/// and `key`; 0 when there is none. Chromium runs the editing shortcuts
/// (Ctrl+A, Ctrl+Z) from it.
pub fn virtual_key(code: &str, key: &str) -> u32 {
    let numbered = |prefix: &str, base: u32, last: u32| {
        code.strip_prefix(prefix)
            .and_then(|rest| rest.parse::<u32>().ok())
            .filter(|n| *n <= last)
            .map(|n| base + n)
    };
    if let Some(letter) = code.strip_prefix("Key").filter(|rest| rest.len() == 1) {
        return u32::from(letter.as_bytes()[0].to_ascii_uppercase());
    }
    if let Some(vk) = numbered("Digit", 48, 9)
        .or_else(|| numbered("Numpad", 96, 9))
        .or_else(|| numbered("F", 111, 12).filter(|vk| *vk > 111))
    {
        return vk;
    }
    match code {
        "Space" => return 32,
        "NumpadEnter" => return 13,
        "NumpadDecimal" => return 110,
        _ => {}
    }
    if let Some((_, vk)) = PUNCTUATION.iter().find(|(name, _)| *name == code) {
        return *vk;
    }
    match key.as_bytes() {
        [byte] if byte.is_ascii_alphanumeric() => u32::from(byte.to_ascii_uppercase()),
        _ => 0,
    }
}

/// The names to list when a key is unknown.
pub fn names() -> String {
    KEYS.iter()
        .map(|key| if key.key == " " { "Space" } else { key.key })
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_are_found_by_common_names() {
        assert_eq!(find("enter").map(|k| k.virtual_key), Some(13));
        assert_eq!(find("Esc").map(|k| k.key), Some("Escape"));
        assert_eq!(find("space").and_then(|k| k.text), Some(" "));
        assert_eq!(find("down").map(|k| k.key), Some("ArrowDown"));
        assert_eq!(find("PAGEDOWN").map(|k| k.virtual_key), Some(34));
        assert_eq!(find("F13"), None);
        assert!(names().contains("Space"));
    }

    #[test]
    fn owner_keys_get_their_windows_virtual_key() {
        assert_eq!(virtual_key("KeyA", "a"), 65);
        assert_eq!(virtual_key("KeyZ", "Z"), 90);
        assert_eq!(virtual_key("Digit7", "&"), 55);
        assert_eq!(virtual_key("Numpad3", "3"), 99);
        assert_eq!(virtual_key("F5", "F5"), 116);
        assert_eq!(virtual_key("F12", "F12"), 123);
        assert_eq!(virtual_key("Slash", ";"), 191);
        assert_eq!(virtual_key("IntlRo", "/"), 193);
        assert_eq!(virtual_key("", "q"), 81);
        assert_eq!(virtual_key("", "ç"), 0);
        assert_eq!(named("ArrowUp").map(|k| k.virtual_key), Some(38));
        assert_eq!(named("arrowup"), None);
        assert_eq!(named(" ").and_then(|k| k.text), Some(" "));
    }
}
