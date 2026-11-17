//! `GET /media/<token>` (spec 8.4): a sound or video of a bot, read from
//! disk in the pieces its player asks for. The file is checked again on
//! every request, so one that is gone, or no longer the bot's, stops playing.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{Path as UrlPath, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use futures_util::{Stream, stream};
use tracing::debug;

use super::range::{Span, span};
use crate::service::files::{located, media_type};
use crate::state::Daemon;

/// Bytes read from disk at a time.
const CHUNK: u64 = 256 * 1024;
/// Never a page: the answer runs with no rights, also if opened directly.
const SANDBOX: &str = "sandbox";

pub async fn handle(
    State(daemon): State<Arc<Daemon>>,
    UrlPath(token): UrlPath<String>,
    headers: HeaderMap,
) -> Response {
    let response = answer(daemon, &token, &headers).await;
    debug!(status = response.status().as_u16(), "media request");
    response
}

async fn answer(daemon: Arc<Daemon>, token: &str, headers: &HeaderMap) -> Response {
    let Some(linked) = daemon.media.find(token) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let range = headers
        .get(header::RANGE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let opened = tokio::task::spawn_blocking(move || {
        let (path, len) = located(&daemon, &linked.bot, &linked.path)?;
        let wanted = span(range.as_deref(), len);
        let mut file = File::open(&path).ok()?;
        if let Span::Part { first, .. } = wanted {
            file.seek(SeekFrom::Start(first)).ok()?;
        }
        Some((media_type(&name(&path)), len, wanted, file))
    })
    .await
    .ok()
    .flatten();
    let Some((media, len, wanted, file)) = opened else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let (status, count, piece) = match wanted {
        Span::Whole => (StatusCode::OK, len, None),
        Span::Part { first, last } => (
            StatusCode::PARTIAL_CONTENT,
            last - first + 1,
            Some(format!("bytes {first}-{last}/{len}")),
        ),
        Span::Outside => {
            let mut refused = StatusCode::RANGE_NOT_SATISFIABLE.into_response();
            set(
                &mut refused,
                header::CONTENT_RANGE,
                &format!("bytes */{len}"),
            );
            return refused;
        }
    };
    let mut response = (status, Body::from_stream(chunks(file, count))).into_response();
    set(&mut response, header::CONTENT_TYPE, media);
    set(&mut response, header::CONTENT_LENGTH, &count.to_string());
    if let Some(piece) = piece {
        set(&mut response, header::CONTENT_RANGE, &piece);
    }
    let out = response.headers_mut();
    let fixed = [
        (header::ACCEPT_RANGES, "bytes"),
        (header::CACHE_CONTROL, "no-store"),
        (header::CONTENT_SECURITY_POLICY, SANDBOX),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::REFERRER_POLICY, "no-referrer"),
    ];
    for (name, value) in fixed {
        out.insert(name, HeaderValue::from_static(value));
    }
    response
}

fn set(response: &mut Response, name: header::HeaderName, value: &str) {
    if let Ok(value) = HeaderValue::from_str(value) {
        response.headers_mut().insert(name, value);
    }
}

fn name(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

/// `count` bytes of `file` from where it stands, a piece at a time, each
/// read off the async threads.
fn chunks(file: File, count: u64) -> impl Stream<Item = io::Result<Bytes>> + Send + 'static {
    stream::try_unfold((file, count), |(mut file, left)| async move {
        if left == 0 {
            return Ok(None);
        }
        let size = left.min(CHUNK);
        let (file, piece) = tokio::task::spawn_blocking(move || {
            // At most CHUNK, so it fits in usize.
            let mut piece = vec![0; usize::try_from(size).unwrap_or(0)];
            file.read_exact(&mut piece).map(|()| (file, piece))
        })
        .await
        .map_err(io::Error::other)??;
        Ok(Some((Bytes::from(piece), (file, left - size))))
    })
}
