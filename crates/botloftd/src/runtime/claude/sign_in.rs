//! When the claude.ai sign-in of the agents' Claude Code ends (spec 42.9),
//! as `claude auth status --text` says it from Claude Code 2.1.318 on: a
//! line starting with "Sign-in", such as "Sign-in ends: between 2026-11-20
//! and 2026-11-23 (2026-11-23T17:04:11.000Z)", or "any day now" and "may
//! have ended" when it is about to end. Other credentials, or no known
//! date, print no such line. Using Claude Code does not move the dates.

use botloft_core::protocol::SignInEnds;

/// The dates in the "Sign-in" line of `text`, read at `now` (Unix ms).
pub fn parse_sign_in_ends(text: &str, now: i64) -> Option<SignInEnds> {
    let line = text
        .lines()
        .map(str::trim)
        .find(|line| line.starts_with("Sign-in"))?;
    let rest = line.split_once(':').map_or(line, |(_, rest)| rest);
    if rest.contains("any day now") || rest.contains("may have ended") {
        return Some(SignInEnds { from: now, by: now });
    }
    // The earlier end comes first; the later one may come again with its time.
    let mut moments = rest
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || matches!(ch, '-' | ':' | '.')))
        .filter_map(moment);
    let from = moments.next()?;
    let by = moments.fold(from, i64::max);
    Some(SignInEnds { from, by })
}

/// A time ("2026-11-23T17:04:11.000Z") or a day, at midnight UTC.
fn moment(word: &str) -> Option<i64> {
    if let Ok(at) = word.parse::<jiff::Timestamp>() {
        return Some(at.as_millisecond());
    }
    let day = word.parse::<jiff::civil::Date>().ok()?;
    let midnight = day.to_zoned(jiff::tz::TimeZone::UTC).ok()?;
    Some(midnight.timestamp().as_millisecond())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_763_000_000_000;

    fn ms(text: &str) -> i64 {
        moment(text).expect("a moment")
    }

    #[test]
    fn the_documented_line_gives_both_ends() {
        let text = "Logged in\nSign-in ends: between 2026-11-20 and 2026-11-23 \
                    (2026-11-23T17:04:11.000Z)\nOrganization: x";
        assert_eq!(
            parse_sign_in_ends(text, NOW),
            Some(SignInEnds {
                from: ms("2026-11-20"),
                by: ms("2026-11-23T17:04:11.000Z"),
            })
        );
    }

    #[test]
    fn about_to_end_is_now() {
        for line in ["Sign-in ends: any day now", "Sign-in may have ended"] {
            assert_eq!(
                parse_sign_in_ends(line, NOW),
                Some(SignInEnds { from: NOW, by: NOW }),
                "{line}"
            );
        }
    }

    #[test]
    fn no_line_or_no_date_is_nothing() {
        assert_eq!(parse_sign_in_ends("Logged in with an API key", NOW), None);
        assert_eq!(parse_sign_in_ends("Sign-in ends: unknown", NOW), None);
    }
}
