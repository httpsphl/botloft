//! Text that looks like a key someone pasted (spec 32.7): a known prefix
//! followed by a long run of token characters, or a JWT. Works on raw text
//! and on the JSON a chat item is kept as.

/// Prefixes of keys from well-known services.
pub const PREFIXES: &[&str] = &[
    "sk-ant-",
    "sk_live_",
    "rk_live_",
    "github_pat_",
    "ghp_",
    "xoxb-",
    "xoxp-",
    "whsec_",
    "AKIA",
    "AIza",
    "sk-",
];
/// Characters after the prefix for it to count, so "sk-learn" does not.
pub const BODY_MIN: usize = 16;
const JWT_START: &str = "eyJ";

/// One key found: its value and how it starts, the part shown to the owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Found<'a> {
    pub value: &'a str,
    pub prefix: &'static str,
}

impl Found<'_> {
    /// The start and the last 4 characters, never the whole value.
    pub fn preview(&self) -> String {
        let tail = &self.value[self.value.len().saturating_sub(4)..];
        format!("{}…{tail}", self.prefix)
    }
}

fn token_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
}

/// Whether a key may start at `at`: not inside a word, except after an
/// escape in JSON text ("\nghp_...").
fn starts_word(bytes: &[u8], at: usize) -> bool {
    let Some(&before) = at.checked_sub(1).and_then(|i| bytes.get(i)) else {
        return true;
    };
    !token_char(before)
        || (at >= 2 && bytes[at - 2] == b'\\' && matches!(before, b'n' | b't' | b'r'))
}

fn run(bytes: &[u8], from: usize) -> usize {
    let mut end = from;
    while bytes.get(end).copied().is_some_and(token_char) {
        end += 1;
    }
    end
}

fn prefixed(bytes: &[u8], at: usize) -> Option<(usize, &'static str)> {
    let rest = &bytes[at..];
    let prefix = PREFIXES.iter().find(|p| rest.starts_with(p.as_bytes()))?;
    let end = run(bytes, at + prefix.len());
    (end - at - prefix.len() >= BODY_MIN).then_some((end, *prefix))
}

/// Three base64url parts joined by dots, the first one a JSON header.
fn jwt(bytes: &[u8], at: usize) -> Option<usize> {
    if !bytes[at..].starts_with(JWT_START.as_bytes()) {
        return None;
    }
    // Each part: where it ends, and whether it is long enough.
    let part = |from: usize, min: usize| {
        let end = run(bytes, from);
        (end - from >= min).then_some(end)
    };
    let dot = |end: usize| (bytes.get(end) == Some(&b'.')).then_some(end + 1);
    let second = part(at, 10).and_then(dot)?;
    let third = part(second, 4).and_then(dot)?;
    part(third, 4)
}

/// The keys in `text`, in order.
pub fn find(text: &str) -> Vec<Found<'_>> {
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if starts_word(bytes, at) {
            let hit = jwt(bytes, at)
                .map(|end| (end, JWT_START))
                .or_else(|| prefixed(bytes, at));
            // Only ASCII bytes match, so `at` and `end` are char boundaries.
            if let Some((end, prefix)) = hit {
                found.push(Found {
                    value: &text[at..end],
                    prefix,
                });
                at = end;
                continue;
            }
        }
        at += 1;
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(text: &str) -> Vec<&str> {
        find(text).into_iter().map(|f| f.value).collect()
    }

    #[test]
    fn each_known_prefix_is_found_with_a_long_enough_body() {
        let body = "a1B2c3D4e5F6g7H8";
        for prefix in PREFIXES {
            let key = format!("{prefix}{body}");
            let text = format!("use {key}, thanks");
            assert_eq!(values(&text), [key.as_str()], "{prefix}");
        }
        assert_eq!(values("AKIAIOSFODNN7EXAMPLE"), ["AKIAIOSFODNN7EXAMPLE"]);
        assert_eq!(
            values("sk-ant-api03-abcdefghijklmnop_QRST-uvw"),
            ["sk-ant-api03-abcdefghijklmnop_QRST-uvw"]
        );
    }

    #[test]
    fn words_and_short_runs_are_not_keys() {
        for text in [
            "task-runner and risk-free",
            "pip install sk-learn",
            "ghp_short",
            "the [chave GITHUB_TOKEN] is kept",
            "eyJhbGciOiJIUzI1NiJ9 without dots",
            "ASKIA is a name",
        ] {
            assert!(find(text).is_empty(), "{text}");
        }
    }

    #[test]
    fn a_jwt_is_found_whole() {
        let jwt =
            "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0In0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        assert_eq!(values(&format!("Bearer {jwt}.")), [jwt]);
    }

    #[test]
    fn keys_inside_json_text_and_after_an_escape_are_found() {
        let key = "ghp_abcdefghijklmnopqrstuvwxyz0123";
        let json = format!(r#"{{"text":"line one\n{key}\tnext","output":"{key}"}}"#);
        assert_eq!(values(&json), [key, key]);
    }

    #[test]
    fn the_preview_shows_the_start_and_the_end_only() {
        let found = find(concat!("xox", "b-1234567890-abcdefghij"))[0];
        assert_eq!(found.preview(), "xoxb-…ghij");
        assert_eq!(find("é sk-0123456789abcdefWXYZ")[0].preview(), "sk-…WXYZ");
    }
}
