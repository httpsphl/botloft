//! What a page or an e-mail needs of HTML: escaping what a person or the app
//! typed, and the flame.

use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::IntoResponse;

/// The Botloft mascot, the flame of the app's icon.
const FLAME: &[u8] = include_bytes!("../assets/flame.png");

/// `text` as it can sit in HTML, in a text or in an attribute.
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// `GET /brand/flame.png`: the same image for the pages and the e-mails.
pub async fn flame() -> impl IntoResponse {
    (
        [
            (CONTENT_TYPE, "image/png"),
            (CACHE_CONTROL, "public, max-age=86400"),
        ],
        FLAME,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_a_person_typed_cannot_become_markup() {
        assert_eq!(
            escape("<b onclick=\"x\">&'"),
            "&lt;b onclick=&quot;x&quot;&gt;&amp;&#39;"
        );
        assert_eq!(escape("PC da Ana"), "PC da Ana");
    }

    #[test]
    fn the_flame_is_a_png() {
        assert_eq!(&FLAME[..8], b"\x89PNG\r\n\x1a\n");
    }
}
