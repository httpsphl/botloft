//! Backups the owner keeps (spec 14.2): everything Botloft knows and every
//! crew's folder, in one file sealed with a passphrase only the owner has.

pub mod export;
pub mod restore;
pub mod seal;

/// The extension of a backup file.
pub const BACKUP_EXTENSION: &str = "botloft";
/// Entries of the archive inside.
pub(crate) const MANIFEST_ENTRY: &str = "manifest.json";
pub(crate) const DATABASE_ENTRY: &str = "botloft.db";
pub(crate) const WORKSPACES_ENTRY: &str = "workspaces";
