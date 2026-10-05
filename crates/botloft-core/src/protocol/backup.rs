//! Backups the owner exports and imports (spec 14.2).

use serde::{Deserialize, Serialize};

/// What a backup holds, as its manifest says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BackupManifest {
    /// The archive's layout version.
    pub format: u32,
    /// Unix time in milliseconds.
    pub created_at: i64,
    /// The Botloft version that made it.
    pub version: String,
    pub crews: Vec<BackupCrew>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BackupCrew {
    pub name: String,
    /// The crew's bots, archived ones aside.
    pub bots: Vec<String>,
    /// A work folder the owner chose outside Botloft's folders: not in the
    /// backup.
    pub work_folder: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BackupExportParams {
    pub passphrase: String,
}

/// A backup written and ready for the owner to save elsewhere.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BackupExported {
    pub path: String,
    /// Bytes.
    pub size: u64,
    pub manifest: BackupManifest,
}
