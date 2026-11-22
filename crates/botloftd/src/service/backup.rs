//! `backup.*` (spec 14.2).

use botloft_core::protocol::{
    BackupChunkParams, BackupExportParams, BackupExported, BackupManifest, BackupStageParams,
    FileChunk,
};
use tracing::info;

use super::{ApiError, ApiResult};
use crate::backup::{export, restore, seal::SealError};
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
    let scope = params.scope.unwrap_or_default();
    let exported = export::export(daemon, &params.passphrase, scope).map_err(api)?;
    info!(bytes = exported.size, "backup exported");
    Ok(exported)
}

/// `backup.stage`: opens a backup the owner picked and keeps it ready.
/// Nothing changes yet.
pub fn stage(daemon: &Daemon, params: BackupStageParams) -> ApiResult<BackupManifest> {
    let file = std::path::Path::new(params.path.trim());
    if !file.is_file() {
        return Err(ApiError::NotFound(format!(
            "there is no file at {}",
            params.path
        )));
    }
    restore::stage(&daemon.paths, file, &params.passphrase).map_err(api)
}

/// `backup.confirm`: the staged backup replaces everything when Botloft
/// starts again, which the app does next.
pub fn confirm(daemon: &Daemon) -> ApiResult<()> {
    restore::confirm(&daemon.paths).map_err(ApiError::Workspace)?;
    info!("a backup will be restored on the next start");
    Ok(())
}

/// `backup.cancel`: the staged backup goes.
pub fn cancel(daemon: &Daemon) -> ApiResult<()> {
    restore::cancel(&daemon.paths);
    Ok(())
}

/// `backup.chunk`: a piece of the backup `backup.export` wrote, for an app on
/// another computer to save it (spec 42.7). Nothing outside the exports
/// folder is read.
pub fn chunk(daemon: &Daemon, params: BackupChunkParams) -> ApiResult<FileChunk> {
    let not_ours = || {
        ApiError::NotFound(format!(
            "{} is not a backup this Botloft wrote",
            params.path
        ))
    };
    let exports = daemon
        .paths
        .home
        .join(export::EXPORTS)
        .canonicalize()
        .map_err(|_| not_ours())?;
    let path = std::path::Path::new(&params.path)
        .canonicalize()
        .map_err(|_| not_ours())?;
    if !path.starts_with(&exports) || !path.is_file() {
        return Err(not_ours());
    }
    super::files::read_piece(&path, params.offset).map_err(|_| not_ours())
}
