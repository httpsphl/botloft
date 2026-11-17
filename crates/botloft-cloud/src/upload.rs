//! An upload on its way in (spec 27.4, 28.14): to a temp file first, with its
//! size and SHA-256 checked as it comes, so a cut or a wrong one never
//! reaches the store.

use std::path::{Path, PathBuf};

use axum::body::Body;
use axum::http::HeaderMap;
use axum::http::header::CONTENT_LENGTH;
use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tokio::io::AsyncWriteExt;

use crate::app::AppState;
use crate::error::ApiError;
use crate::tokens;

/// The upload's temp file goes away however the request ends.
pub struct Temp(PathBuf);

impl Temp {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub fn header_text<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// The length and the SHA-256 (hex, lower case) an upload says it has.
pub fn declared(headers: &HeaderMap) -> Result<(u64, String), ApiError> {
    let length = header_text(headers, CONTENT_LENGTH.as_str())
        .and_then(|text| text.parse::<u64>().ok())
        .ok_or_else(ApiError::length_required)?;
    let hash = header_text(headers, "x-botloft-sha256")
        .map(str::to_ascii_lowercase)
        .filter(|hash| hash.len() == 64 && hash.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(ApiError::bad_hash)?;
    if length == 0 {
        return Err(ApiError::bad_length());
    }
    Ok((length, hash))
}

/// The body, written to a temp file: exactly `length` bytes with the SHA-256 `wanted`.
pub async fn receive(
    state: &AppState,
    body: Body,
    length: u64,
    wanted: &str,
) -> Result<Temp, ApiError> {
    let folder = state.config.data_dir.join("uploads");
    tokio::fs::create_dir_all(&folder).await.map_err(io)?;
    let temp = Temp(folder.join(format!("{}.part", &tokens::random()?[..20])));
    let mut file = tokio::fs::File::create(&temp.0).await.map_err(io)?;
    let (mut hasher, mut total) = (Sha256::new(), 0u64);
    let mut stream = body.into_data_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|_| ApiError::bad_length())?;
        total += chunk.len() as u64;
        if total > length {
            return Err(ApiError::bad_length());
        }
        hasher.update(&chunk);
        file.write_all(&chunk).await.map_err(io)?;
    }
    file.flush().await.map_err(io)?;
    drop(file);
    if total != length {
        return Err(ApiError::bad_length());
    }
    if hex::encode(hasher.finalize()) != wanted {
        return Err(ApiError::bad_hash());
    }
    Ok(temp)
}

fn io(err: std::io::Error) -> ApiError {
    tracing::error!("upload file failed: {}", err.kind());
    ApiError::internal()
}
