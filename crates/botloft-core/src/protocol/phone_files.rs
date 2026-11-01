//! A file the phone sends with a message (spec 28.14). It waits on the
//! server, sealed with a key of its own; the key comes here, inside the
//! sealed `send`, so only the computer can open the file.

use serde::{Deserialize, Serialize};

/// The biggest file the phone sends, in bytes, before the seal.
pub const PHONE_FILE_MAX: u64 = 20 << 20;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PhoneFile {
    /// The file on the server (`fil_...`).
    pub id: String,
    /// The AES-256-GCM key that opens it, base64url; it seals this file only.
    pub key: String,
    pub name: String,
    pub media_type: String,
    /// The bytes of the file itself.
    pub size: u64,
}

impl std::fmt::Debug for PhoneFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never the key: it opens the file.
        f.debug_struct("PhoneFile")
            .field("id", &self.id)
            .field("media_type", &self.media_type)
            .field("size", &self.size)
            .finish_non_exhaustive()
    }
}
