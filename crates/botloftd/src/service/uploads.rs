//! `uploads.*` (spec 42.7): a file the owner picked on another computer comes
//! in pieces, into `<home>\uploads\<id>\<name>`, for what takes a path here
//! (`backup.stage`, `imports.scan`). The folder is the daemon's own, out of
//! the agents' reach (42.5), and an upload left alone for an hour goes.

use std::fs::{self, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::protocol::{
    CHUNK_MAX, UploadFinished, UploadIdParams, UploadStarted, UploadsChunkParams,
    UploadsStartParams,
};
use botloft_core::slug;
use tracing::info;

use super::{ApiError, ApiResult};
use crate::secrets;
use crate::state::Daemon;

const UPLOADS: &str = "uploads";
/// How long an upload is kept after its last piece.
const KEPT: Duration = Duration::from_secs(60 * 60);
/// The largest file taken.
const SIZE_MAX: u64 = 16 * 1024 * 1024 * 1024;
/// Beside the file: the size the app said it has.
const SIZE_FILE: &str = ".size";
/// Hex characters of an upload's id.
const ID_LEN: usize = 32;

/// `uploads.start`: an empty file of its own, waiting for the pieces.
pub fn start(daemon: &Daemon, params: UploadsStartParams) -> ApiResult<UploadStarted> {
    let root = daemon.paths.home.join(UPLOADS);
    sweep(&root, SystemTime::now());
    if params.size > SIZE_MAX {
        return Err(ApiError::validation(format!(
            "the file is larger than {} GB",
            SIZE_MAX >> 30
        )));
    }
    let token = secrets::random_token().map_err(ApiError::Token)?;
    let id = token[..ID_LEN].to_owned();
    let dir = root.join(&id);
    let name = slug::file_name(&params.name);
    fs::create_dir_all(&dir)
        .and_then(|()| fs::write(dir.join(SIZE_FILE), params.size.to_string()))
        .and_then(|()| fs::File::create(dir.join(&name)).map(drop))
        .map_err(ApiError::Workspace)?;
    Ok(UploadStarted { upload_id: id })
}

/// `uploads.chunk`: the next piece goes at the end; any other offset is a
/// conflict, so a piece sent twice never doubles.
pub fn chunk(daemon: &Daemon, params: UploadsChunkParams) -> ApiResult<()> {
    let (file, size) = find(daemon, &params.upload_id)?;
    let bytes = BASE64
        .decode(params.data.as_bytes())
        .map_err(|_| ApiError::validation("the piece is not base64"))?;
    if bytes.len() as u64 > CHUNK_MAX {
        return Err(ApiError::validation("the piece is too large"));
    }
    let had = fs::metadata(&file).map_err(ApiError::Workspace)?.len();
    if params.offset != had {
        return Err(ApiError::Conflict(format!(
            "the upload has {had} bytes, not {}",
            params.offset
        )));
    }
    if had + bytes.len() as u64 > size {
        return Err(ApiError::validation("the pieces are larger than the file"));
    }
    OpenOptions::new()
        .append(true)
        .open(&file)
        .and_then(|mut out| out.write_all(&bytes))
        .map_err(ApiError::Workspace)
}

/// `uploads.finish`: where the whole file is.
pub fn finish(daemon: &Daemon, params: UploadIdParams) -> ApiResult<UploadFinished> {
    let (file, size) = find(daemon, &params.upload_id)?;
    let had = fs::metadata(&file).map_err(ApiError::Workspace)?.len();
    if had != size {
        return Err(ApiError::Conflict(format!(
            "the upload has {had} of {size} bytes"
        )));
    }
    info!(bytes = size, "a file came from another computer");
    Ok(UploadFinished {
        path: file.display().to_string(),
    })
}

/// `uploads.cancel`: the upload and what came of it go.
pub fn cancel(daemon: &Daemon, params: UploadIdParams) -> ApiResult<()> {
    if is_id(&params.upload_id) {
        let _ = fs::remove_dir_all(daemon.paths.home.join(UPLOADS).join(&params.upload_id));
    }
    Ok(())
}

fn is_id(id: &str) -> bool {
    id.len() == ID_LEN && id.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// The upload's file and the size it will have.
fn find(daemon: &Daemon, id: &str) -> ApiResult<(PathBuf, u64)> {
    let gone = || ApiError::NotFound(format!("upload {id}"));
    if !is_id(id) {
        return Err(gone());
    }
    let dir = daemon.paths.home.join(UPLOADS).join(id);
    let size = fs::read_to_string(dir.join(SIZE_FILE))
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .ok_or_else(gone)?;
    let file = fs::read_dir(&dir)
        .map_err(|_| gone())?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.file_name().is_some_and(|name| name != SIZE_FILE))
        .ok_or_else(gone)?;
    Ok((file, size))
}

/// Removes the uploads nobody added to for longer than `KEPT`.
fn sweep(root: &Path, now: SystemTime) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for dir in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        let last = fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter_map(|entry| entry.metadata().ok()?.modified().ok())
            .max();
        let stale = last.is_none_or(|last| now.duration_since(last).unwrap_or_default() > KEPT);
        if stale {
            let _ = fs::remove_dir_all(&dir);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_old_uploads_are_swept() {
        let root = tempfile::tempdir().expect("dir");
        let fresh = root.path().join("a".repeat(ID_LEN));
        fs::create_dir_all(&fresh).expect("dir");
        fs::write(fresh.join("x.zip"), b"x").expect("file");
        let empty = root.path().join("b".repeat(ID_LEN));
        fs::create_dir_all(&empty).expect("dir");

        sweep(root.path(), SystemTime::now());
        assert!(fresh.exists(), "an upload in use stays");
        assert!(!empty.exists(), "an upload with nothing in it goes");

        sweep(
            root.path(),
            SystemTime::now() + KEPT + Duration::from_secs(1),
        );
        assert!(!fresh.exists(), "an hour later, it goes");
    }

    #[test]
    fn ids_are_hex_of_one_length() {
        assert!(is_id(&"0a".repeat(ID_LEN / 2)));
        assert!(!is_id("../../secrets"));
        assert!(!is_id(&"g".repeat(ID_LEN)));
        assert!(!is_id("abc"));
    }
}
