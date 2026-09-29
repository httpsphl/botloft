//! The keys `browser_press` knows, as `Input.dispatchKeyEvent` wants them.

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
}
