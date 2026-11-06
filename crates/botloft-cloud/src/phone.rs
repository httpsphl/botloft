//! The phone's page (spec 28.7): the files of `app/dist-phone`, served at
//! `/m`. They are the page and nothing else; what a bot asks never passes
//! here. Every answer says what the page may do (a policy of its own), so a
//! page that was changed could not send what it holds anywhere else.

use std::path::Path;

use axum::extract::{Path as UrlPath, State};
use axum::response::Response;

use crate::app::AppState;
use crate::statics::send;

/// What the page may load and talk to: its own address, and nothing more.
const POLICY: &str = "default-src 'none'; script-src 'self'; style-src 'self'; \
img-src 'self' data:; font-src 'self'; connect-src 'self'; manifest-src 'self'; \
worker-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'";

/// `GET /m` and `GET /m/`
pub async fn index(State(state): State<AppState>) -> Response {
    send(state.config.phone_dir.as_deref(), "index.html", POLICY).await
}

/// `GET /m/<path>`: a file of the page; a path with no file extension is the
/// page itself, so a reload anywhere in it works.
pub async fn file(State(state): State<AppState>, UrlPath(path): UrlPath<String>) -> Response {
    let dir = state.config.phone_dir.as_deref();
    if Path::new(&path).extension().is_none() {
        return send(dir, "index.html", POLICY).await;
    }
    send(dir, &path, POLICY).await
}
