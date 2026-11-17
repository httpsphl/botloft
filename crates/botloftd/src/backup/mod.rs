//! Backups the owner keeps (spec 14.2): everything Botloft knows and every
//! crew's folder, in one file sealed with a passphrase only the owner has.

pub mod export;
pub mod fingerprint;
pub mod restore;
pub mod seal;

/// The extension of a backup file.
pub const BACKUP_EXTENSION: &str = "botloft";
/// Entries of the archive inside.
pub(crate) const MANIFEST_ENTRY: &str = "manifest.json";
pub(crate) const DATABASE_ENTRY: &str = "botloft.db";
pub(crate) const WORKSPACES_ENTRY: &str = "workspaces";

use std::path::Path;

use botloft_core::protocol::BackupScope;
use botloft_store::{Copier, StoreError};

use crate::state::Daemon;

/// A copy of the database at `to` (which must not exist yet), whole or
/// light. On a connection of its own, so the bots, the tools and the app go
/// on while it is made; on the store itself only in memory (the tests).
pub(crate) fn copy_database(
    daemon: &Daemon,
    scope: BackupScope,
    to: &Path,
) -> Result<(), StoreError> {
    let file = daemon.store().file();
    match (file, scope) {
        (Some(db), BackupScope::Full) => Copier::open(&db)?.snapshot_to(to),
        (Some(db), BackupScope::Light) => Copier::open(&db)?.snapshot_light_to(to),
        (None, BackupScope::Full) => daemon.store().snapshot_to(to),
        (None, BackupScope::Light) => daemon.store().snapshot_light_to(to),
    }
}

/// The hash of what the light copy would hold (27.10), made the same way.
pub(crate) fn database_fingerprint(daemon: &Daemon, scratch: &Path) -> Result<String, StoreError> {
    let file = daemon.store().file();
    match file {
        Some(db) => Copier::open(&db)?.light_fingerprint(scratch),
        None => daemon.store().light_fingerprint(scratch),
    }
}
