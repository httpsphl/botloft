//! The phone's page (spec 28.7): the files of `app/dist-phone`, served at
//! `/m`. They are the page and nothing else; what a bot asks never passes
//! here. Every answer says what the page may do (a policy of its own), so a
//! page that was changed could not send what it holds anywhere else.

use std::path::{Component, Path, PathBuf};

use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::app::AppState;

/// What the page may load and talk to: its own address, and nothing more.
const POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'self'; \
img-src 'self' data:; font-src 'self'; connect-src 'self'; manifest-src 'self'; \
worker-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

/// `GET /m` and `GET /m/`
pub async fn index(State(state): State<AppState>) -> Response {
    send(state.config.phone_dir.as_deref(), "index.html").await
}

/// `GET /m/<path>`: a file of the page; a path with no file extension is the
/// page itself, so a reload anywhere in it works.
pub async fn file(State(state): State<AppState>, UrlPath(path): UrlPath<String>) -> Response {
    let dir = state.config.phone_dir.as_deref();
    if Path::new(&path).extension().is_none() {
        return send(dir, "index.html").await;
    }
    send(dir, &path).await
}

/// The file, from inside `dir` only.
async fn send(dir: Option<&Path>, relative: &str) -> Response {
    let Some(dir) = dir else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(path) = inside(dir, relative) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    // A link inside the folder that leads out of it is not the page.
    let (Ok(real), Ok(root)) = (
        tokio::fs::canonicalize(&path).await,
        tokio::fs::canonicalize(dir).await,
    ) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if !real.starts_with(&root) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let Ok(bytes) = tokio::fs::read(&real).await else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut response = bytes.into_response();
    let headers = response.headers_mut();
    let mut put = |name: header::HeaderName, value: &str| {
        if let Ok(value) = HeaderValue::from_str(value) {
            headers.insert(name, value);
        }
    };
    put(header::CONTENT_TYPE, kind(&real));
    // The built files carry a hash in their names; the page itself, the
    // worker and the manifest are looked at every time.
    put(
        header::CACHE_CONTROL,
        if relative.starts_with("assets/") {
            "public, max-age=31536000, immutable"
        } else {
            "no-cache"
        },
    );
    put(header::CONTENT_SECURITY_POLICY, POLICY);
    put(header::X_CONTENT_TYPE_OPTIONS, "nosniff");
    put(header::REFERRER_POLICY, "no-referrer");
    response
}

/// `relative` under `dir`, if it holds only plain names: no `..`, no root,
/// no drive, no empty part.
fn inside(dir: &Path, relative: &str) -> Option<PathBuf> {
    if relative.is_empty() || relative.contains('\\') || relative.contains('\0') {
        return None;
    }
    let mut path = dir.to_path_buf();
    for part in Path::new(relative).components() {
        match part {
            Component::Normal(name) if !name.to_string_lossy().contains(':') => path.push(name),
            _ => return None,
        }
    }
    Some(path)
}

fn kind(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" | "map" => "application/json",
        "webmanifest" => "application/manifest+json",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "ico" => "image/x-icon",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_plain_names_stay_inside_the_folder() {
        let dir = Path::new("/srv/phone");
        assert_eq!(
            inside(dir, "assets/app.js"),
            Some(PathBuf::from("/srv/phone/assets/app.js"))
        );
        for bad in [
            "",
            "..",
            "../x",
            "a/../../x",
            "/etc/passwd",
            "a\\b",
            "c:/x",
            "a/c:x",
            "a\0",
        ] {
            assert_eq!(inside(dir, bad), None, "{bad:?}");
        }
    }
}
