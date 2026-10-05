//! `backup.*` (spec 14.2).

use botloft_core::protocol::{BackupExportParams, BackupExported};
use tracing::info;

use super::{ApiError, ApiResult};
use crate::backup::{export, seal::SealError};
use crate::state::Daemon;

/// A problem the owner can fix, with a `reason` the app words (spec 20.8).
pub(crate) fn api(err: SealError) -> ApiError {
    let reason = match &err {
        SealError::Io(_) => return ApiError::Workspace(std::io::Error::other(err.to_string())),
        SealError::NotABackup => "not_a_backup",
        SealError::Newer => "newer_backup",
        SealError::WrongPassphrase => "wrong_passphrase",
        SealError::ShortPassphrase => "short_passphrase",
    };
    ApiError::Rule {
        reason,
        message: err.to_string(),
    }
}

/// `backup.export`: seals a backup and says where it is.
pub fn export(daemon: &Daemon, params: BackupExportParams) -> ApiResult<BackupExported> {
    let exported = export::export(daemon, &params.passphrase).map_err(api)?;
    info!(bytes = exported.size, "backup exported");
    Ok(exported)
}
