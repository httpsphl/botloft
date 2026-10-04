//! Keys a bot presses with the real keyboard (spec 24.7): "Enter",
//! "Ctrl+S", "Shift+Tab". Never the system's own combinations: the Windows
//! key, switching or closing what is not the app (Alt+Tab, Alt+Esc,
//! Ctrl+Esc), or the secure screen and Task Manager (Ctrl+Alt+Del,
//! Ctrl+Shift+Esc).

/// A key a bot may press, by its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Enter,
    Tab,
    Escape,
    Backspace,
    Delete,
    Space,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Insert,
    /// F1 to F12.
    F(u8),
    /// A letter or digit, upper case.
    Char(char),
}

/// A key with the modifiers held while it is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Keys {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub key: Option<Key>,
}

/// Why keys are refused.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeysError {
    #[error("{0:?} is not a key you can press; use names like Enter, Tab, Ctrl+S")]
    Unknown(String),
    #[error("{0} belongs to Windows itself, not to the app, so you may never press it")]
    System(String),
}

fn key(name: &str) -> Option<Key> {
    let lower = name.to_ascii_lowercase();
    Some(match lower.as_str() {
        "enter" | "return" => Key::Enter,
        "tab" => Key::Tab,
        "escape" | "esc" => Key::Escape,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "space" => Key::Space,
        "up" | "arrowup" => Key::Up,
        "down" | "arrowdown" => Key::Down,
        "left" | "arrowleft" => Key::Left,
        "right" | "arrowright" => Key::Right,
        "home" => Key::Home,
        "end" => Key::End,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "insert" | "ins" => Key::Insert,
        _ => {
            if let Some(number) = lower.strip_prefix('f')
                && let Ok(number) = number.parse::<u8>()
                && (1..=12).contains(&number)
            {
                return Some(Key::F(number));
            }
            let mut chars = name.chars();
            match (chars.next(), chars.next()) {
                (Some(one), None) if one.is_ascii_alphanumeric() => {
                    Key::Char(one.to_ascii_uppercase())
                }
                _ => return None,
            }
        }
    })
}

/// Reads keys like "Ctrl+Shift+S": modifiers, then one key.
pub fn parse(text: &str) -> Result<Keys, KeysError> {
    let mut keys = Keys::default();
    for part in text.split('+').map(str::trim) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => keys.ctrl = true,
            "alt" => keys.alt = true,
            "shift" => keys.shift = true,
            "win" | "windows" | "meta" | "super" | "cmd" => {
                return Err(KeysError::System(text.to_owned()));
            }
            _ if keys.key.is_none() => {
                keys.key = Some(key(part).ok_or_else(|| KeysError::Unknown(text.to_owned()))?);
            }
            _ => return Err(KeysError::Unknown(text.to_owned())),
        }
    }
    let Some(pressed) = keys.key else {
        return Err(KeysError::Unknown(text.to_owned()));
    };
    let system = (keys.alt && matches!(pressed, Key::Tab | Key::Escape))
        || (keys.ctrl && !keys.alt && pressed == Key::Escape)
        || (keys.ctrl && keys.alt && pressed == Key::Delete);
    if system {
        return Err(KeysError::System(text.to_owned()));
    }
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_read_by_name_with_their_modifiers() {
        assert_eq!(parse("Enter").expect("enter").key, Some(Key::Enter));
        let save = parse("Ctrl+S").expect("save");
        assert!(save.ctrl && !save.alt && !save.shift);
        assert_eq!(save.key, Some(Key::Char('S')));
        let back = parse("shift + tab").expect("back");
        assert!(back.shift);
        assert_eq!(back.key, Some(Key::Tab));
        assert_eq!(parse("F5").expect("f5").key, Some(Key::F(5)));
        assert_eq!(parse("Alt+F4").expect("close").key, Some(Key::F(4)));
        assert!(matches!(parse("F13"), Err(KeysError::Unknown(_))));
        assert!(matches!(parse("Ctrl+"), Err(KeysError::Unknown(_))));
        assert!(matches!(parse("Ctrl+S+T"), Err(KeysError::Unknown(_))));
    }

    #[test]
    fn the_systems_own_combinations_are_never_pressed() {
        for system in [
            "Win+R",
            "Windows+D",
            "Alt+Tab",
            "Alt+Shift+Tab",
            "Alt+Esc",
            "Ctrl+Esc",
            "Ctrl+Alt+Delete",
            "Ctrl+Alt+Del",
            "Ctrl+Shift+Esc",
        ] {
            assert!(
                matches!(parse(system), Err(KeysError::System(_))),
                "{system}"
            );
        }
    }
}
