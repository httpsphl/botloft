//! The files a bot made, for the app's files panel (spec 8.5).

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// A file in the bot's folder or the crew's work folder, or one the bot's
/// own `Write`/`Edit` calls changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotFile {
    /// Absolute path.
    pub path: String,
    /// File name, without folders.
    pub name: String,
    /// Folder it is in, relative to the work folder or the bot's folder when
    /// it is inside one, else the absolute folder; empty at the top.
    pub folder: String,
    /// Guessed from the extension.
    pub media_type: String,
    /// Bytes.
    pub size: u64,
    /// Unix time in milliseconds.
    pub modified_at: i64,
    /// The bot's own `Write`/`Edit` calls changed it, as opposed to a file
    /// that only showed up in the folder.
    pub written_by_bot: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct FilesListParams {
    pub bot_id: BotId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct FilesReadParams {
    pub bot_id: BotId,
    /// A `BotFile.path`.
    pub path: String,
}

/// A file's bytes, for the app to preview.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct FileData {
    pub media_type: String,
    /// The file's bytes, base64.
    pub data: String,
}

impl std::fmt::Debug for FileData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileData")
            .field("media_type", &self.media_type)
            .field("data", &format_args!("<{} base64 chars>", self.data.len()))
            .finish()
    }
}
