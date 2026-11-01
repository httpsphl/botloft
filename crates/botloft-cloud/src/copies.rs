//! The copies (spec 27.4): send one, list them, download (with `Range`),
//! delete. The server never opens a copy; it keeps what the owner sealed.

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{ACCEPT_RANGES, CONTENT_LENGTH, CONTENT_RANGE, CONTENT_TYPE, RANGE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use serde_json::{Value, json};

use crate::app::AppState;
use crate::copy_rows::{self, Copy, blob_key};
use crate::error::ApiError;
use crate::session::Computer;
use crate::tokens;
use crate::upload::{self, header_text};

/// `PUT /v1/copies`
pub async fn put(
    State(state): State<AppState>,
    Computer(who): Computer,
    headers: HeaderMap,
    body: Body,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let (declared, wanted) = upload::declared(&headers)?;
    if declared > state.config.max_copy_bytes {
        return Err(ApiError::too_big());
    }
    let (keep, quota) = (state.config.keep, state.config.quota_bytes);
    let room = state
        .db
        .run(|conn| copy_rows::fits(conn, who.account_id, keep, declared, quota))?;
    if !room {
        return Err(ApiError::quota());
    }

    let temp = upload::receive(&state, body, declared, &wanted).await?;
    let id = format!("cpy_{}", &tokens::random()?[..16]);
    let key = blob_key(who.account_id, &id);
    state.store.put_file(&key, temp.path(), declared).await?;
    let copy = Copy {
        id,
        size: declared as i64,
        sha256: wanted,
        created_at: state.clock.now(),
    };
    let pruned = state
        .db
        .run(|conn| copy_rows::add_and_prune(conn, who.account_id, &copy, keep));
    let pruned = match pruned {
        Ok(pruned) => pruned,
        Err(err) => {
            state.store.delete(&key).await;
            return Err(err);
        }
    };
    // Only now, with the new one safe, do the oldest go.
    for old in pruned {
        state.store.delete(&old).await;
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({ "id": copy.id, "size": copy.size, "created": copy.created_at })),
    ))
}

/// `GET /v1/copies`, newest first.
pub async fn list(
    State(state): State<AppState>,
    Computer(who): Computer,
) -> Result<Json<Value>, ApiError> {
    let copies = state.db.run(|conn| copy_rows::list(conn, who.account_id))?;
    let copies: Vec<Value> = copies
        .into_iter()
        .map(|copy| json!({ "id": copy.id, "size": copy.size, "created": copy.created_at }))
        .collect();
    Ok(Json(json!({ "copies": copies })))
}

/// `GET /v1/copies/<id>`, all of it or one `Range: bytes=`.
pub async fn get(
    State(state): State<AppState>,
    Computer(who): Computer,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let copy = state
        .db
        .run(|conn| copy_rows::find(conn, who.account_id, &id))?
        .ok_or_else(ApiError::not_found)?;
    let size = copy.size as u64;
    let wanted = match header_text(&headers, RANGE.as_str()) {
        Some(text) => Some(parse_range(text, size).ok_or_else(ApiError::range_not_satisfiable)?),
        None => None,
    };
    let (first, last) = wanted.unwrap_or((0, size - 1));
    let range = wanted.map(|(first, last)| first..last + 1);
    let stream = state
        .store
        .read(&blob_key(who.account_id, &copy.id), range)
        .await?;

    let mut response = Response::new(Body::from_stream(stream));
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    headers.insert(CONTENT_LENGTH, HeaderValue::from(last - first + 1));
    if let Ok(hash) = HeaderValue::from_str(&copy.sha256) {
        headers.insert("x-botloft-sha256", hash);
    }
    if wanted.is_some() {
        *response.status_mut() = StatusCode::PARTIAL_CONTENT;
        if let Ok(value) = HeaderValue::from_str(&format!("bytes {first}-{last}/{}", copy.size)) {
            response.headers_mut().insert(CONTENT_RANGE, value);
        }
    }
    Ok(response)
}

/// `DELETE /v1/copies/<id>`
pub async fn remove(
    State(state): State<AppState>,
    Computer(who): Computer,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let removed = state
        .db
        .run(|conn| copy_rows::delete(conn, who.account_id, &id))?;
    if !removed {
        return Err(ApiError::not_found());
    }
    state.store.delete(&blob_key(who.account_id, &id)).await;
    Ok(StatusCode::NO_CONTENT)
}

/// One range of `bytes=a-b`, `a-` or `-n`, as inclusive `(first, last)` inside
/// `size` bytes. `None` for anything else or outside the copy.
fn parse_range(text: &str, size: u64) -> Option<(u64, u64)> {
    let spec = text.trim().strip_prefix("bytes=")?;
    if spec.contains(',') {
        return None;
    }
    let (from, to) = spec.split_once('-')?;
    let (first, last) = match (from.trim(), to.trim()) {
        ("", n) => {
            let n: u64 = n.parse().ok().filter(|n| *n > 0)?;
            (size.saturating_sub(n), size - 1)
        }
        (a, "") => (a.parse().ok()?, size - 1),
        (a, b) => (a.parse().ok()?, b.parse::<u64>().ok()?.min(size - 1)),
    };
    (first <= last && first < size).then_some((first, last))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_are_read_inside_the_copy() {
        assert_eq!(parse_range("bytes=0-9", 100), Some((0, 9)));
        assert_eq!(parse_range("bytes=90-", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=-10", 100), Some((90, 99)));
        assert_eq!(parse_range("bytes=50-5000", 100), Some((50, 99)));
        assert_eq!(parse_range("bytes=-500", 100), Some((0, 99)));
        for bad in [
            "bytes=100-",
            "bytes=5-2",
            "bytes=0-1,4-5",
            "items=0-1",
            "bytes=-0",
            "bytes=a-b",
        ] {
            assert_eq!(parse_range(bad, 100), None, "{bad}");
        }
    }
}
