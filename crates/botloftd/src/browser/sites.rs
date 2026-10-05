//! Which addresses a bot may open and which site each one is (spec 21.5).
//! A site is the host in lower case without `www.`; allowing a site also
//! allows its subdomains.

use std::path::{Path, PathBuf};

/// Where `browser_open` goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Place {
    /// A web address, with its site (`None` for `about:blank`).
    Web { url: String, site: Option<String> },
    /// A file in the bot's folders, as a `file:` URL.
    File { url: String },
}

/// Reads what the bot asked to open: a web address, `about:blank`, or a
/// file inside `folders` given as a path (absolute or relative to `base`)
/// or a `file:` URL. The message explains a refusal to the bot.
pub fn resolve(input: &str, base: &Path, folders: &[PathBuf]) -> Result<Place, String> {
    let input = input.trim();
    let lower = input.to_ascii_lowercase();
    if lower == "about:blank" {
        return Ok(Place::Web {
            url: "about:blank".to_owned(),
            site: None,
        });
    }
    if lower.starts_with("http://") || lower.starts_with("https://") {
        let site = site_of(input).ok_or_else(|| format!("{input} has no host to open"))?;
        return Ok(Place::Web {
            url: input.to_owned(),
            site: Some(site),
        });
    }
    if lower.starts_with("file:") {
        return file(&file_path(input), base, folders);
    }
    if has_scheme(input) {
        return Err(format!(
            "{input} cannot be opened: use an http or https address, or a file in your folders"
        ));
    }
    // A path to one of the bot's files, or a bare address.
    let candidate = base.join(input);
    if candidate.is_file() {
        return file(&candidate, base, folders);
    }
    if looks_like_a_host(input) {
        let url = format!("https://{input}");
        let site = site_of(&url).ok_or_else(|| format!("{input} has no host to open"))?;
        return Ok(Place::Web {
            url,
            site: Some(site),
        });
    }
    Err(format!(
        "{input} is neither a web address nor a file in your folders"
    ))
}

/// Longest address the owner may type.
const TYPED_MAX: usize = 2000;

/// What the owner typed in the address bar, as the web address to open
/// (spec 21.10): `https://` goes in front of one without a scheme, also of
/// a host with a port. Anything that is not a web address is `None`.
pub fn typed(input: &str) -> Option<String> {
    let input = input.trim();
    if input.is_empty() || input.len() > TYPED_MAX || input.contains(char::is_whitespace) {
        return None;
    }
    let lower = input.to_ascii_lowercase();
    let url = if lower.starts_with("http://") || lower.starts_with("https://") {
        input.to_owned()
    } else if has_scheme(input) && !has_port(input) {
        return None;
    } else {
        format!("https://{input}")
    };
    site_of(&url).map(|_| url)
}

/// `localhost:3000/x`: what follows the colon is a port, not the rest of a
/// `scheme:` address.
fn has_port(input: &str) -> bool {
    input.split_once(':').is_some_and(|(_, rest)| {
        let port = rest.split(['/', '?', '#']).next().unwrap_or_default();
        !port.is_empty() && port.chars().all(|c| c.is_ascii_digit())
    })
}

/// The site of a web address, or `None` for anything else.
pub fn site_of(url: &str) -> Option<String> {
    let lower = url.to_ascii_lowercase();
    let rest = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))?;
    let authority = rest.split(['/', '?', '#']).next()?;
    let host = authority.rsplit('@').next()?;
    let host = if let Some(v6) = host.strip_prefix('[') {
        v6.split(']').next()?
    } else {
        host.split(':').next()?
    };
    let host = host.trim_end_matches('.');
    let host = host.strip_prefix("www.").unwrap_or(host);
    (!host.is_empty()).then(|| host.to_owned())
}

/// The port of a loopback address (`localhost`, `127.x.x.x`, `[::1]` or
/// `0.0.0.0`), or the scheme's own when none is written. `None` for any
/// other address.
pub fn loopback_port(url: &str) -> Option<u16> {
    let lower = url.to_ascii_lowercase();
    let (rest, default) = match lower.strip_prefix("https://") {
        Some(rest) => (rest, 443),
        None => (lower.strip_prefix("http://")?, 80),
    };
    let authority = rest.split(['/', '?', '#']).next()?;
    let host_port = authority.rsplit('@').next()?;
    let (host, port) = match host_port.strip_prefix('[') {
        Some(v6) => {
            let (host, after) = v6.split_once(']')?;
            (host, after.strip_prefix(':'))
        }
        None => match host_port.split_once(':') {
            Some((host, port)) => (host, Some(port)),
            None => (host_port, None),
        },
    };
    let host = host.trim_end_matches('.');
    let loopback = host == "localhost"
        || host.ends_with(".localhost")
        || host == "::1"
        || host == "0.0.0.0"
        || host
            .parse::<std::net::Ipv4Addr>()
            .is_ok_and(|ip| ip.is_loopback());
    if !loopback {
        return None;
    }
    match port {
        None | Some("") => Some(default),
        Some(port) => port.parse().ok(),
    }
}

/// Whether allowing `allowed` also allows `site`.
pub fn covers(allowed: &str, site: &str) -> bool {
    site == allowed
        || site
            .strip_suffix(allowed)
            .is_some_and(|prefix| prefix.ends_with('.'))
}

/// Whether `url` is a `file:` URL.
pub fn is_file(url: &str) -> bool {
    url.get(..5)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("file:"))
}

/// Whether a `file:` URL is inside `folders`.
pub fn file_allowed(url: &str, folders: &[PathBuf]) -> bool {
    is_file(url) && inside(&file_path(url), folders).is_some()
}

/// The path of a `file:` URL: `file:///C:/a%20b/x.html?q` is `C:/a b/x.html`
/// on Windows, and `file:///home/ana/x.html` is `/home/ana/x.html` elsewhere.
fn file_path(url: &str) -> PathBuf {
    let rest = url.get(5..).unwrap_or_default();
    let rest = rest.split(['?', '#']).next().unwrap_or_default();
    let path = decode(rest.trim_start_matches('/'));
    if cfg!(windows) {
        PathBuf::from(path)
    } else {
        PathBuf::from(format!("/{path}"))
    }
}

fn file(path: &Path, base: &Path, folders: &[PathBuf]) -> Result<Place, String> {
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        base.join(path)
    };
    let Some(real) = inside(&path, folders) else {
        return Err(format!(
            "{} is not a file in your folders; you can open only files in your own folder or \
             the crew's work folder",
            path.display()
        ));
    };
    if !real.is_file() {
        return Err(format!("{} is not a file", path.display()));
    }
    Ok(Place::File {
        url: file_url(&real),
    })
}

/// The real path of `path` if it exists inside one of `folders`.
fn inside(path: &Path, folders: &[PathBuf]) -> Option<PathBuf> {
    let real = std::fs::canonicalize(path).ok()?;
    folders
        .iter()
        .filter_map(|folder| std::fs::canonicalize(folder).ok())
        .any(|folder| real.starts_with(&folder))
        .then_some(real)
}

/// `C:\a b\x.html` as `file:///C:/a%20b/x.html`, and `/home/ana/x.html` as
/// `file:///home/ana/x.html`.
fn file_url(path: &Path) -> String {
    let text = path.to_string_lossy();
    let text = if cfg!(windows) {
        text.strip_prefix(r"\\?\")
            .unwrap_or(&text)
            .replace('\\', "/")
    } else {
        text.trim_start_matches('/').to_owned()
    };
    let mut url = String::from("file:///");
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b':' | b'-' | b'_' | b'.' | b'~' => {
                url.push(char::from(byte));
            }
            _ => url.push_str(&format!("%{byte:02X}")),
        }
    }
    url
}

/// Undoes `%XX` escapes; anything malformed stays as it is.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let escaped = (bytes[i] == b'%')
            .then(|| text.get(i + 1..i + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match escaped {
            Some(byte) => {
                out.push(byte);
                i += 3;
            }
            None => {
                out.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `mailto:x`, `javascript:x`, `edge://x`; not a drive letter (`C:\x`).
fn has_scheme(input: &str) -> bool {
    match input.split_once(':') {
        Some((scheme, _)) => {
            scheme.len() > 1
                && scheme
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
        }
        None => false,
    }
}

/// Endings of file names that are not top-level domains: `index.html` is a
/// missing file, not a site.
const FILE_ENDINGS: [&str; 13] = [
    "html", "htm", "pdf", "txt", "png", "jpg", "jpeg", "gif", "svg", "json", "csv", "css", "js",
];

/// `example.com` or `example.com/a`: a dotted name without spaces or
/// backslashes.
fn looks_like_a_host(input: &str) -> bool {
    let host = input.split(['/', '?', '#']).next().unwrap_or_default();
    let last = host.rsplit('.').next().unwrap_or_default();
    host.contains('.')
        && !FILE_ENDINGS.contains(&last.to_ascii_lowercase().as_str())
        && !input.contains('\\')
        && !input.contains(char::is_whitespace)
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':'))
}

#[cfg(test)]
#[path = "sites_tests.rs"]
mod tests;
