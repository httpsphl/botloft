//! `files.chunk` (spec 42.7): a piece of one of the bot's files, for an app
//! on another computer to save or open it. It reaches what `files.read`
//! reaches, at any size.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::protocol::{CHUNK_MAX, FileChunk, FilesChunkParams};

use super::{places, reached};
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

pub fn chunk(daemon: &Daemon, params: FilesChunkParams) -> ApiResult<FileChunk> {
    let (_, places) = places(daemon, &params.bot_id)?;
    let (path, _) = reached(&places, &params.path)?;
    read_piece(&path, params.offset)
        .map_err(|_| ApiError::NotFound(format!("{} is no longer there", params.path)))
}

/// Up to `CHUNK_MAX` bytes of `path` from `offset`, and its size.
pub(crate) fn read_piece(path: &Path, offset: u64) -> io::Result<FileChunk> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    let mut bytes = Vec::new();
    if offset < size {
        file.seek(SeekFrom::Start(offset))?;
        file.take(CHUNK_MAX).read_to_end(&mut bytes)?;
    }
    Ok(FileChunk {
        data: BASE64.encode(bytes),
        size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pieces_cover_the_file_and_stop_at_its_end() {
        let dir = tempfile::tempdir().expect("dir");
        let path = dir.path().join("big.bin");
        let bytes: Vec<u8> = (0..(CHUNK_MAX + 10)).map(|at| (at % 251) as u8).collect();
        std::fs::write(&path, &bytes).expect("write");

        let first = read_piece(&path, 0).expect("first");
        assert_eq!(first.size, CHUNK_MAX + 10);
        let first = BASE64.decode(first.data).expect("base64");
        assert_eq!(first.len() as u64, CHUNK_MAX);
        let rest = BASE64
            .decode(read_piece(&path, CHUNK_MAX).expect("rest").data)
            .expect("base64");
        assert_eq!([first, rest].concat(), bytes);
        let past = read_piece(&path, CHUNK_MAX + 10).expect("past");
        assert_eq!(past.data, "");
    }
}
