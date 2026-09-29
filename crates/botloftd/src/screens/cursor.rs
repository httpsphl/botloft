//! The bot's cursor on a screen it writes (spec 22.3): each draft served
//! carries a small script that tells the app where the page ends now.

const SCRIPT: &str = include_str!("cursor.js");
const DOCTYPE: &str = "<!doctype";

/// `html` with the cursor script first: right after the doctype, which has
/// to stay first for the page to keep its layout mode, or at the very top.
/// A doctype still being written gets no script this time.
pub(super) fn with_cursor(html: &str) -> String {
    let script = format!("<script data-botloft-cursor>{SCRIPT}</script>");
    let rest = html.trim_start_matches(['\u{feff}', ' ', '\t', '\r', '\n']);
    let start = html.len() - rest.len();
    let head = rest.get(..DOCTYPE.len());
    if head.is_some_and(|head| head.eq_ignore_ascii_case(DOCTYPE)) {
        return match rest.find('>') {
            Some(close) => {
                let at = start + close + 1;
                format!("{}{script}{}", &html[..at], &html[at..])
            }
            None => html.to_owned(),
        };
    }
    let begun = rest.len() < DOCTYPE.len()
        && !rest.is_empty()
        && DOCTYPE.starts_with(&rest.to_ascii_lowercase());
    if begun {
        return html.to_owned();
    }
    format!("{script}{html}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAG: &str = "<script data-botloft-cursor>";

    #[test]
    fn the_script_goes_after_the_doctype_or_first() {
        let page = with_cursor("<!DOCTYPE html>\n<h1>Bak");
        assert!(page.starts_with("<!DOCTYPE html><script data-botloft-cursor>"));
        assert!(page.ends_with("</script>\n<h1>Bak"));

        let bare = with_cursor("<h1>Bak");
        assert!(bare.starts_with(TAG));
        assert!(bare.ends_with("</script><h1>Bak"));

        let marked = with_cursor("\u{feff}  <!doctype html><p>é</p>");
        assert!(marked.starts_with("\u{feff}  <!doctype html><script"));
    }

    #[test]
    fn a_doctype_being_written_waits() {
        assert_eq!(with_cursor("<!doc"), "<!doc");
        assert_eq!(with_cursor("<!DOCTYPE htm"), "<!DOCTYPE htm");
        assert_eq!(with_cursor("<"), "<");
        assert!(with_cursor("<p>").starts_with(TAG));
        assert!(with_cursor("").starts_with(TAG));
    }

    #[test]
    fn the_script_never_closes_early() {
        assert!(!SCRIPT.contains("</script"));
    }
}
