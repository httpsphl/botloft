//! Reading a tool's input while the model is still writing it (spec 22.3):
//! the string fields of a JSON object that may stop anywhere, even in the
//! middle of an escape.

use std::collections::HashMap;
use std::iter::Peekable;
use std::str::Chars;

/// A string field as far as it arrived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    pub value: String,
    /// The closing quote arrived.
    pub complete: bool,
}

/// The top-level string fields of `json`, which may be cut anywhere. Other
/// values are skipped; anything malformed ends the reading.
pub fn fields(json: &str) -> HashMap<String, Field> {
    let mut out = HashMap::new();
    let mut chars = json.chars().peekable();
    skip_space(&mut chars);
    if chars.next() != Some('{') {
        return out;
    }
    loop {
        skip_space(&mut chars);
        match chars.peek() {
            Some(',') => {
                chars.next();
            }
            Some('"') => {
                let key = string(&mut chars);
                if !key.complete {
                    return out;
                }
                skip_space(&mut chars);
                if chars.next() != Some(':') {
                    return out;
                }
                skip_space(&mut chars);
                if chars.peek() == Some(&'"') {
                    let value = string(&mut chars);
                    let complete = value.complete;
                    out.insert(key.value, value);
                    if !complete {
                        return out;
                    }
                } else if !skip_value(&mut chars) {
                    return out;
                }
            }
            _ => return out,
        }
    }
}

fn skip_space(chars: &mut Peekable<Chars<'_>>) {
    while chars.peek().is_some_and(|c| c.is_whitespace()) {
        chars.next();
    }
}

/// A JSON string from its opening quote, unescaped as far as it arrived.
/// An escape cut short is left out.
fn string(chars: &mut Peekable<Chars<'_>>) -> Field {
    chars.next();
    let mut value = String::new();
    let cut = |value: String| Field {
        value,
        complete: false,
    };
    loop {
        match chars.next() {
            None => return cut(value),
            Some('"') => {
                return Field {
                    value,
                    complete: true,
                };
            }
            Some('\\') => match chars.next() {
                None => return cut(value),
                Some('n') => value.push('\n'),
                Some('t') => value.push('\t'),
                Some('r') => value.push('\r'),
                Some('b') => value.push('\u{8}'),
                Some('f') => value.push('\u{c}'),
                Some('u') => match unicode(chars) {
                    Some(Some(c)) => value.push(c),
                    Some(None) => value.push(char::REPLACEMENT_CHARACTER),
                    None => return cut(value),
                },
                Some(other) => value.push(other),
            },
            Some(c) => value.push(c),
        }
    }
}

/// The character of a `\u` escape, after the `u`: `None` when it is cut
/// short, `Some(None)` when it names no character.
fn unicode(chars: &mut Peekable<Chars<'_>>) -> Option<Option<char>> {
    let high = hex4(chars)?;
    if !(0xD800..0xDC00).contains(&high) {
        return Some(char::from_u32(high));
    }
    // A high surrogate needs the low one that follows it.
    if chars.next()? != '\\' || chars.next()? != 'u' {
        return Some(None);
    }
    let low = hex4(chars)?;
    if !(0xDC00..0xE000).contains(&low) {
        return Some(None);
    }
    Some(char::from_u32(
        0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00),
    ))
}

fn hex4(chars: &mut Peekable<Chars<'_>>) -> Option<u32> {
    let mut value = 0;
    for _ in 0..4 {
        let digit = chars.next()?.to_digit(16)?;
        value = value * 16 + digit;
    }
    Some(value)
}

/// Skips a number, literal, object or array; `false` if it is cut short.
fn skip_value(chars: &mut Peekable<Chars<'_>>) -> bool {
    let mut depth = 0usize;
    loop {
        match chars.peek() {
            None => return false,
            Some('"') => {
                if !string(chars).complete {
                    return false;
                }
                if depth == 0 {
                    return true;
                }
            }
            Some('{' | '[') => {
                depth += 1;
                chars.next();
            }
            Some('}' | ']') if depth == 0 => return true,
            Some('}' | ']') => {
                depth -= 1;
                chars.next();
                if depth == 0 {
                    return true;
                }
            }
            Some(',') if depth == 0 => return true,
            Some(_) => {
                chars.next();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(json: &str, name: &str) -> Option<Field> {
        fields(json).remove(name)
    }

    #[test]
    fn a_whole_object_reads_every_string() {
        let json = r#"{"file_path": "C:\\site\\a.html", "content": "<p>\"hi\"</p>\n", "n": 3}"#;
        let found = fields(json);
        assert_eq!(found["file_path"].value, r"C:\site\a.html");
        assert!(found["file_path"].complete);
        assert_eq!(found["content"].value, "<p>\"hi\"</p>\n");
        assert!(found["content"].complete);
    }

    #[test]
    fn a_cut_string_gives_what_arrived() {
        let json = r#"{"file_path": "C:\\a.html", "content": "<h1>Bak"#;
        assert_eq!(
            field(json, "content"),
            Some(Field {
                value: "<h1>Bak".to_owned(),
                complete: false
            })
        );
        assert!(field(json, "file_path").is_some_and(|path| path.complete));
        // The path itself cut short is not complete yet.
        let early = r#"{"file_path": "C:\\si"#;
        assert!(field(early, "file_path").is_some_and(|path| !path.complete));
        assert!(field(early, "content").is_none());
    }

    #[test]
    fn escapes_cut_short_are_left_out() {
        assert_eq!(
            field(r#"{"content": "a\"#, "content").map(|f| f.value),
            Some("a".into())
        );
        assert_eq!(
            field(r#"{"content": "a\u00"#, "content").map(|f| f.value),
            Some("a".into())
        );
        assert_eq!(
            field(r#"{"content": "p\u00e3o"#, "content").map(|f| f.value),
            Some("pão".into())
        );
        // An emoji is two escapes; half of it waits for the rest.
        assert_eq!(
            field(r#"{"content": "x\ud83c"#, "content").map(|f| f.value),
            Some("x".into())
        );
        assert_eq!(
            field(r#"{"content": "x\ud83c\udf5e!"#, "content").map(|f| f.value),
            Some("x🍞!".into())
        );
    }

    #[test]
    fn other_values_are_skipped_in_any_order() {
        let json = r#"{"a": {"b": [1, "}"]}, "flag": true, "content": "ok", "file_path": "x.htm"#;
        let found = fields(json);
        assert_eq!(found["content"].value, "ok");
        assert_eq!(found["file_path"].value, "x.htm");
        assert!(!found["file_path"].complete);
        assert!(fields("").is_empty());
        assert!(fields("[1]").is_empty());
    }
}
