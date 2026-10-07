//! Sending a copy up and bringing one down (spec 27.4), with the SHA-256 the
//! server and this computer check on both ends.

use std::io;
use std::path::Path;
use std::sync::Arc;

use botloft_core::protocol::CloudCopy;
use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::Body;
use reqwest::header::{CONTENT_LENGTH, HeaderValue};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::CloudError;
use super::client::{Server, copy_of};

/// Told how many bytes went, and how many there are in all.
pub type Progress = Arc<dyn Fn(u64, u64) + Send + Sync>;

const CHUNK: usize = 64 * 1024;

fn io_error(err: &io::Error) -> CloudError {
    CloudError::other(format!("could not use the copy's file: {}", err.kind()))
}

/// The SHA-256 of a file, read in pieces.
async fn hash_file(path: &Path) -> io::Result<String> {
    let mut file = tokio::fs::File::open(path).await?;
    let (mut hasher, mut buffer) = (Sha256::new(), vec![0u8; 1 << 20]);
    loop {
        let read = file.read(&mut buffer).await?;
        if read == 0 {
            return Ok(hex::encode(hasher.finalize()));
        }
        hasher.update(&buffer[..read]);
    }
}

/// Sends the copy at `path` as the next copy of the account.
pub async fn upload(
    server: &Server,
    token: &str,
    path: &Path,
    progress: Progress,
) -> Result<CloudCopy, CloudError> {
    let size = tokio::fs::metadata(path)
        .await
        .map_err(|e| io_error(&e))?
        .len();
    let hash = hash_file(path).await.map_err(|e| io_error(&e))?;
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|e| io_error(&e))?;
    let stream = futures_util::stream::unfold((file, 0u64), move |(mut file, sent)| {
        let progress = Arc::clone(&progress);
        async move {
            let mut buffer = vec![0u8; CHUNK];
            match file.read(&mut buffer).await {
                Ok(0) => None,
                Ok(read) => {
                    buffer.truncate(read);
                    let sent = sent + read as u64;
                    progress(sent, size);
                    Some((Ok::<_, io::Error>(Bytes::from(buffer)), (file, sent)))
                }
                Err(err) => Some((Err(err), (file, sent))),
            }
        }
    });
    let request = server
        .http
        .put(server.url("/v1/copies"))
        .bearer_auth(token)
        .header(CONTENT_LENGTH, size)
        .header("x-botloft-sha256", hash)
        .body(Body::wrap_stream(stream));
    let response = server.send(request).await?;
    let answer = response
        .json()
        .await
        .map_err(|_| CloudError::other("the server's answer was not understood".to_owned()))?;
    copy_of(&answer)
}

/// Brings copy `id` to `dest`, checked against its SHA-256. A copy that does
/// not match leaves nothing behind.
pub async fn download(
    server: &Server,
    token: &str,
    id: &str,
    dest: &Path,
    progress: Progress,
) -> Result<(), CloudError> {
    let response = server
        .send(
            server
                .http
                .get(server.url(&format!("/v1/copies/{id}")))
                .bearer_auth(token),
        )
        .await?;
    let total = response.content_length().unwrap_or(0);
    let wanted = response
        .headers()
        .get("x-botloft-sha256")
        .and_then(|value: &HeaderValue| value.to_str().ok())
        .map(str::to_owned);
    if let Some(folder) = dest.parent() {
        tokio::fs::create_dir_all(folder)
            .await
            .map_err(|e| io_error(&e))?;
    }
    let part = dest.with_extension("part");
    let result = write_stream(response, &part, total, progress).await;
    let got = match result {
        Ok(got) => got,
        Err(err) => {
            let _ = tokio::fs::remove_file(&part).await;
            return Err(err);
        }
    };
    if wanted.is_some_and(|wanted| wanted != got) {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(CloudError::from_server(
            "bad_hash",
            "the copy that came down is not the one that went up".to_owned(),
        ));
    }
    tokio::fs::rename(&part, dest)
        .await
        .map_err(|e| io_error(&e))
}

/// Writes the body to `part` and gives back its SHA-256.
async fn write_stream(
    response: reqwest::Response,
    part: &Path,
    total: u64,
    progress: Progress,
) -> Result<String, CloudError> {
    let mut file = tokio::fs::File::create(part)
        .await
        .map_err(|e| io_error(&e))?;
    let (mut hasher, mut sent) = (Sha256::new(), 0u64);
    let mut body = response.bytes_stream();
    while let Some(chunk) = body.next().await {
        let chunk = chunk.map_err(|_| CloudError::offline())?;
        hasher.update(&chunk);
        file.write_all(&chunk).await.map_err(|e| io_error(&e))?;
        sent += chunk.len() as u64;
        progress(sent, total.max(sent));
    }
    file.flush().await.map_err(|e| io_error(&e))?;
    Ok(hex::encode(hasher.finalize()))
}
