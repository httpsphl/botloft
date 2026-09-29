//! `GET /view/<key>/<root>/<path>` (spec 22.2): a bot's screens and the
//! files next to them, for the app's design area. Every page runs in a
//! sandbox with an opaque origin, away from the daemon and the app.

use std::sync::Arc;

use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use tracing::debug;

use super::Place;
use super::place::{Root, roots};
use crate::service::files::media_type;
use crate::state::Daemon;

/// Largest file served.
const MAX_BYTES: u64 = 20 * 1024 * 1024;
const SANDBOX: &str = "sandbox allow-scripts allow-forms allow-popups allow-modals";

pub async fn handle(
    State(daemon): State<Arc<Daemon>>,
    UrlPath((key, root, rel)): UrlPath<(String, String, String)>,
    headers: HeaderMap,
) -> Response {
    let found = find(&daemon, &key, &root, &rel).await;
    let status = if found.is_some() { 200 } else { 404 };
    debug!(status, "view request");
    let Some((body, media)) = found else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let content_type = if media.starts_with("text/") || media.ends_with("javascript") {
        format!("{media}; charset=utf-8")
    } else {
        media.to_owned()
    };
    let mut response = (StatusCode::OK, body).into_response();
    let out = response.headers_mut();
    let fixed = [
        (header::CONTENT_SECURITY_POLICY, SANDBOX),
        (header::CACHE_CONTROL, "no-store"),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "no-referrer"),
    ];
    for (name, value) in fixed {
        out.insert(name, HeaderValue::from_static(value));
    }
    if let Ok(value) = HeaderValue::from_str(&content_type) {
        out.insert(header::CONTENT_TYPE, value);
    }
    // The screen itself, in its sandbox, fetching a file next to it.
    if headers
        .get(header::ORIGIN)
        .is_some_and(|origin| origin == "null")
    {
        out.insert(
            header::ACCESS_CONTROL_ALLOW_ORIGIN,
            HeaderValue::from_static("null"),
        );
    }
    response
}

/// The bytes and type at a `/view` address: the draft being written, or
/// the file, if it is inside the folder.
async fn find(
    daemon: &Daemon,
    key: &str,
    root: &str,
    rel: &str,
) -> Option<(Vec<u8>, &'static str)> {
    let bot = daemon.screens.bot_of(key)?;
    let place = Place::from_request(root, rel)?;
    let media = media_type(&place.rel);
    if let Some(draft) = daemon.screens.draft(&bot, &place) {
        return Some((draft.into_bytes(), media));
    }
    let (work, workspace) = roots(daemon, &bot)?;
    let folder = match place.root {
        Root::Work => work,
        Root::Bot => workspace,
    };
    let path = place.path_in(&folder);
    let body = tokio::task::spawn_blocking(move || {
        let folder = std::fs::canonicalize(folder).ok()?;
        let real = std::fs::canonicalize(path).ok()?;
        let meta = std::fs::metadata(&real).ok()?;
        (real.starts_with(&folder) && meta.is_file() && meta.len() <= MAX_BYTES)
            .then(|| std::fs::read(&real).ok())
            .flatten()
    })
    .await
    .ok()??;
    Some((body, media))
}
