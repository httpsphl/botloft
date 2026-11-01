//! The files a phone sends to its computer (spec 28.14): the phone puts the
//! sealed bytes here, its computer takes them, and they go. The key that
//! opens a file travels inside the relay, where the server cannot read it.

use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::Response;
use serde_json::{Value, json};

use crate::app::AppState;
use crate::error::ApiError;
use crate::file_rows::{self, NewFile, blob_key};
use crate::session::{Computer, Phone};
use crate::{accounts, limits, tokens, upload};

/// The biggest file: 20 MiB, and the 16 bytes of the seal's tag.
pub const MAX_FILE: u64 = (20 << 20) + 16;

/// `PUT /v1/files`
pub async fn put(
    State(state): State<AppState>,
    Phone(who): Phone,
    headers: HeaderMap,
    body: Body,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let (declared, wanted) = upload::declared(&headers)?;
    if declared > MAX_FILE {
        return Err(ApiError::file_too_big());
    }
    let computer = who.peer.clone().ok_or_else(ApiError::wrong_device)?;
    // What ran out makes room first.
    sweep(&state).await;
    let now = state.clock.now();
    let limit_key = tokens::hash(&who.device_id);
    let allowed = state.db.run(|conn| {
        if !file_rows::has_room(conn, &who.device_id, now)? {
            return Ok(Err(None));
        }
        Ok(limits::take(conn, now, &[(&limits::PHONE_FILES, &limit_key)])?.map_err(Some))
    })?;
    match allowed {
        Ok(()) => {}
        Err(None) => return Err(ApiError::files_waiting()),
        Err(Some(wait)) => return Err(ApiError::rate_limited(wait)),
    }

    let temp = upload::receive(&state, body, declared, &wanted).await?;
    let id = format!("fil_{}", &tokens::random()?[..16]);
    let key = blob_key(who.account_id, &id);
    state.store.put_file(&key, temp.path(), declared).await?;
    let file = NewFile {
        id: &id,
        account_id: who.account_id,
        phone: &who.device_id,
        computer: &computer,
        size: declared as i64,
        sha256: &wanted,
        now,
    };
    // The phone may have been cut off meanwhile: then the bytes go too.
    let expires = match state.db.run(|conn| file_rows::add(conn, &file)) {
        Ok(expires) => expires,
        Err(err) => {
            state.store.delete(&key).await;
            return Err(err);
        }
    };
    Ok((
        StatusCode::CREATED,
        Json(json!({ "id": id, "size": declared, "expires": expires })),
    ))
}

/// `GET /v1/files/<id>`: only for the computer the phone serves, and only
/// until the file runs out.
pub async fn get(
    State(state): State<AppState>,
    Computer(who): Computer,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let now = state.clock.now();
    let file = state
        .db
        .run(|conn| file_rows::find(conn, &who.device_id, &id, now))?
        .ok_or_else(ApiError::not_found)?;
    let stream = state
        .store
        .read(&blob_key(file.account_id, &id), None)
        .await?;
    let mut response = Response::new(Body::from_stream(stream));
    let headers = response.headers_mut();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    headers.insert(CONTENT_LENGTH, HeaderValue::from(file.size));
    if let Ok(hash) = HeaderValue::from_str(&file.sha256) {
        headers.insert("x-botloft-sha256", hash);
    }
    Ok(response)
}

/// `DELETE /v1/files/<id>`: the computer took it.
pub async fn remove(
    State(state): State<AppState>,
    Computer(who): Computer,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let key = state
        .db
        .run(|conn| file_rows::take(conn, &who.device_id, &id))?
        .ok_or_else(ApiError::not_found)?;
    state.store.delete(&key).await;
    Ok(StatusCode::NO_CONTENT)
}

/// Drops what ran out: the files nobody took, rows and bytes, and the rest
/// of what expires (spec 27.3). The server runs it every few minutes, and
/// before a phone sends a file.
pub async fn sweep(state: &AppState) {
    let now = state.clock.now();
    let keys = state.db.run(|conn| {
        accounts::purge(conn, now)?;
        file_rows::take_expired(conn, now)
    });
    for key in keys.unwrap_or_default() {
        state.store.delete(&key).await;
    }
}
