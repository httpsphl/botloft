//! Reading a page for a wait (spec 20.14): one GET without cookies, with
//! limits, and its text without tags, to look for the words a bot waits
//! for.

use std::time::Duration;

use botloft_core::slug::fold;
use reqwest::Client;
use reqwest::redirect::Policy;

const TIMEOUT: Duration = Duration::from_secs(20);
const REDIRECTS_MAX: usize = 5;
/// Most bytes of a page read; the rest is not looked at.
const PAGE_MAX_BYTES: usize = 2 * 1024 * 1024;

/// The page's text, or `None` when it could not be read: a page that
/// fails counts as "not yet".
pub(super) async fn fetch(url: &str) -> Option<String> {
    let client = Client::builder()
        .timeout(TIMEOUT)
        .redirect(Policy::limited(REDIRECTS_MAX))
        .user_agent(concat!("botloftd/", env!("CARGO_PKG_VERSION")))
        .build()
        .ok()?;
    let mut response = client.get(url).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.ok()? {
        bytes.extend_from_slice(&chunk);
        if bytes.len() >= PAGE_MAX_BYTES {
            bytes.truncate(PAGE_MAX_BYTES);
            break;
        }
    }
    Some(text_of(&String::from_utf8_lossy(&bytes)))
}

/// Whether the page's text has `wanted`, without accents, case or extra
/// spaces counting.
pub(crate) fn shows(page: &str, wanted: &str) -> bool {
    let wanted = fold(wanted);
    !wanted.is_empty() && fold(page).contains(&wanted)
}

/// The text of an HTML page: scripts and styles dropped, every tag a
/// space, the common entities read.
pub(super) fn text_of(html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut text = String::with_capacity(html.len() / 2);
    let mut at = 0;
    while let Some(open) = html[at..].find('<').map(|found| at + found) {
        text.push_str(&html[at..open]);
        text.push(' ');
        let skip_to = ["script", "style"].iter().find_map(|tag| {
            lower[open + 1..]
                .starts_with(tag)
                .then(|| format!("</{tag}"))
        });
        let close = match skip_to {
            Some(end) => lower[open..]
                .find(&end)
                .and_then(|found| lower[open + found..].find('>').map(|gt| open + found + gt)),
            None => lower[open..].find('>').map(|gt| open + gt),
        };
        let Some(close) = close else {
            at = html.len();
            break;
        };
        at = close + 1;
    }
    text.push_str(&html[at..]);
    text.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::{shows, text_of};

    #[test]
    fn the_text_leaves_tags_scripts_and_styles_out() {
        let html = "<html><head><style>p { color: red }</style>\
            <script>var shipped = 'Enviado';</script></head>\
            <body><p>Pedido <b>ENVIADO</b> &amp; pago</p></body></html>";
        let text = text_of(html);
        assert!(!text.contains("color"), "{text}");
        assert!(!text.contains("var shipped"), "{text}");
        assert!(shows(&text, "pedido enviado & pago"), "{text}");
        assert!(shows(&text, "Enviádo"));
        assert!(!shows(&text, "entregue"));
        assert!(!shows(&text, "  "));
    }

    #[test]
    fn a_tag_left_open_ends_the_text() {
        assert_eq!(text_of("Ready <b").trim(), "Ready");
    }
}
