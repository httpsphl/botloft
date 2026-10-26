//! The rules of an agent's keys (spec 32): what a key may be called, what
//! its value may be, and how a value that shows up in a text is replaced by
//! the key's name before the daemon keeps that text.

use crate::validate::ValidationError;

/// Longest key name, in characters (spec 32.3).
pub const KEY_NAME_MAX: usize = 64;
/// Longest key value, in bytes: tokens and certificates fit.
pub const KEY_VALUE_MAX: usize = 16 * 1024;
/// Longest description, in characters.
pub const KEY_DESCRIPTION_MAX: usize = 200;
/// Shorter values are not looked for in texts: they would match words.
pub const REDACT_MIN_LEN: usize = 8;

/// Names the system, the Botloft or the engine uses (spec 32.3).
const RESERVED: &[&str] = &[
    "PATH",
    "PATHEXT",
    "SYSTEMROOT",
    "COMSPEC",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "HOME",
    "APPDATA",
    "LOCALAPPDATA",
];
const RESERVED_PREFIXES: &[&str] = &[
    "BOTLOFT_",
    "CLAUDE",
    "ANTHROPIC_",
    "CODEX_",
    "OPENAI_",
    "NODE_",
];

/// Checks a key name: `[A-Z][A-Z0-9_]{1,63}`, not one of the reserved ones.
pub fn name(value: &str) -> Result<String, ValidationError> {
    let value = value.trim();
    let mut chars = value.chars();
    let shape = chars.next().is_some_and(|first| first.is_ascii_uppercase())
        && chars.all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        && (2..=KEY_NAME_MAX).contains(&value.len());
    if !shape {
        return Err(ValidationError(format!(
            "name must be 2 to {KEY_NAME_MAX} capital letters, digits or _, starting with a letter"
        )));
    }
    if RESERVED.contains(&value)
        || RESERVED_PREFIXES
            .iter()
            .any(|prefix| value.starts_with(prefix))
    {
        return Err(ValidationError(format!(
            "name {value} is used by the system, Botloft or the engine; pick another"
        )));
    }
    Ok(value.to_owned())
}

/// Checks a key value: not empty, one line, within the size.
pub fn value(value: &str) -> Result<String, ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ValidationError("value must not be empty".to_owned()));
    }
    if value.len() > KEY_VALUE_MAX {
        return Err(ValidationError(format!(
            "value must be at most {KEY_VALUE_MAX} bytes"
        )));
    }
    if value.contains(['\n', '\r', '\0']) {
        return Err(ValidationError("value must be a single line".to_owned()));
    }
    Ok(value.to_owned())
}

/// Checks a description, a single line that may be empty.
pub fn description(value: &str) -> Result<String, ValidationError> {
    let value = value.trim();
    if value.chars().count() > KEY_DESCRIPTION_MAX || value.contains(['\n', '\r']) {
        return Err(ValidationError(format!(
            "description must be one line of at most {KEY_DESCRIPTION_MAX} characters"
        )));
    }
    Ok(value.to_owned())
}

/// `text` with every value of `keys` (name, value) replaced by `[chave NAME]`,
/// longest values first so one that contains another goes whole. Values
/// shorter than `REDACT_MIN_LEN` are left alone. `None` when nothing changed.
pub fn redact(text: &str, keys: &[(String, String)]) -> Option<String> {
    let mut found: Vec<&(String, String)> = keys
        .iter()
        .filter(|(_, value)| value.len() >= REDACT_MIN_LEN && text.contains(value.as_str()))
        .collect();
    if found.is_empty() {
        return None;
    }
    found.sort_by_key(|(_, value)| std::cmp::Reverse(value.len()));
    let mut out = text.to_owned();
    for (name, value) in found {
        out = out.replace(value.as_str(), &format!("[chave {name}]"));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_capitals_and_not_the_systems() {
        assert_eq!(name(" WHATSAPP_TOKEN ").as_deref(), Ok("WHATSAPP_TOKEN"));
        assert!(name("STRIPE_KEY_2").is_ok());
        for bad in [
            "",
            "A",
            "whatsapp",
            "2FA_CODE",
            "MY-KEY",
            "MY KEY",
            "ÁGUA_KEY",
        ] {
            assert!(name(bad).is_err(), "{bad}");
        }
        for reserved in [
            "PATH",
            "TEMP",
            "BOTLOFT_X",
            "CLAUDE_CODE_X",
            "ANTHROPIC_API_KEY",
            "NODE_OPTIONS",
        ] {
            assert!(name(reserved).is_err(), "{reserved}");
        }
    }

    #[test]
    fn values_are_one_line_and_not_empty() {
        assert_eq!(value("  sk-abc123  ").as_deref(), Ok("sk-abc123"));
        assert!(value("   ").is_err());
        assert!(value("a\nb").is_err());
        assert!(value(&"x".repeat(KEY_VALUE_MAX + 1)).is_err());
        assert!(description("Token do gateway").is_ok());
        assert!(description("two\nlines").is_err());
    }

    #[test]
    fn values_in_a_text_become_the_name() {
        let keys = vec![
            ("WA_TOKEN".to_owned(), "wa-1234567890".to_owned()),
            ("SHORT".to_owned(), "abc".to_owned()),
            ("LONG".to_owned(), "wa-1234567890-extra".to_owned()),
        ];
        assert_eq!(
            redact("token=wa-1234567890 and wa-1234567890-extra", &keys).as_deref(),
            Some("token=[chave WA_TOKEN] and [chave LONG]")
        );
        // Too short to look for, and nothing to change.
        assert_eq!(redact("abc def", &keys), None);
        assert_eq!(redact("nothing here", &keys), None);
    }
}
