//! Files that cross the connection in pieces, for an app on another computer
//! (spec 42.7): one of a bot's files or a backup going out, a file the owner
//! picked coming in. Paths are the daemon's; the app never sees its disk.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// Most bytes in one piece, before base64.
pub const CHUNK_MAX: u64 = 1024 * 1024;

/// `files.chunk`: a piece of one of the bot's files, as `files.read` reaches
/// them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct FilesChunkParams {
    pub bot_id: BotId,
    /// A `BotFile.path`.
    pub path: String,
    pub offset: u64,
}

/// `backup.chunk`: a piece of the backup `backup.export` wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BackupChunkParams {
    /// `BackupExported.path`.
    pub path: String,
    pub offset: u64,
}

/// Up to `CHUNK_MAX` bytes from the offset asked, and the whole file's size.
/// Past the end, no bytes.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct FileChunk {
    /// The bytes, base64.
    pub data: String,
    pub size: u64,
}

impl std::fmt::Debug for FileChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileChunk")
            .field("data", &format_args!("<{} base64 chars>", self.data.len()))
            .field("size", &self.size)
            .finish()
    }
}

/// `uploads.start`: a file the owner picked on the app's computer, to send.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct UploadsStartParams {
    pub name: String,
    /// Bytes.
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct UploadStarted {
    pub upload_id: String,
}

/// `uploads.chunk`: the next piece, right after what came so far.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct UploadsChunkParams {
    pub upload_id: String,
    pub offset: u64,
    /// At most `CHUNK_MAX` bytes, base64.
    pub data: String,
}

impl std::fmt::Debug for UploadsChunkParams {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UploadsChunkParams")
            .field("upload_id", &self.upload_id)
            .field("offset", &self.offset)
            .field("data", &format_args!("<{} base64 chars>", self.data.len()))
            .finish()
    }
}

/// `uploads.finish` and `uploads.cancel`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct UploadIdParams {
    pub upload_id: String,
}

/// Where the whole file is now, on the daemon's computer, for what takes a
/// path (`backup.stage`, `imports.scan`). It goes away after an hour.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct UploadFinished {
    pub path: String,
}
