//! Importing a backup (spec 14.2). The file is opened and staged while
//! Botloft runs, and only swapped in on the next start, before the
//! database opens: nothing has the old data open then. What was there goes
//! aside, never deleted.

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};

use botloft_core::protocol::BackupManifest;
use tracing::{info, warn};
use zip::ZipArchive;

use super::seal::{SealError, open};
use super::{DATABASE_ENTRY, MANIFEST_ENTRY, WORKSPACES_ENTRY};
use crate::paths::Paths;

const STAGED: &str = "staged.zip";
/// Written when the owner confirms: the next start applies the backup.
const READY: &str = "READY";
/// What the archive unpacks into before the swap.
const UNPACKED: &str = "unpacked";

fn restore_dir(paths: &Paths) -> PathBuf {
    paths.home.join("restore")
}

fn other(err: impl std::fmt::Display) -> io::Error {
    io::Error::other(err.to_string())
}

/// "0.10.0" as numbers, to compare.
fn version(text: &str) -> Vec<u64> {
    text.split('.')
        .map(|part| part.trim().parse().unwrap_or(0))
        .collect()
}

/// Opens the backup with the passphrase, checks it, and keeps it staged.
/// Nothing changes until `confirm` and a restart.
pub fn stage(paths: &Paths, file: &Path, passphrase: &str) -> Result<BackupManifest, SealError> {
    let dir = restore_dir(paths);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    let staged = dir.join(STAGED);
    let opened = {
        let mut sealed = BufReader::new(File::open(file)?);
        let mut out = BufWriter::new(File::create(&staged)?);
        open(&mut sealed, &mut out, passphrase)
    };
    if let Err(err) = opened {
        let _ = fs::remove_dir_all(&dir);
        return Err(err);
    }
    let checked = check(&staged);
    if checked.is_err() {
        let _ = fs::remove_dir_all(&dir);
    }
    checked
}

fn check(staged: &Path) -> Result<BackupManifest, SealError> {
    let mut zip = ZipArchive::new(File::open(staged)?).map_err(|_| SealError::NotABackup)?;
    let mut text = String::new();
    zip.by_name(MANIFEST_ENTRY)
        .map_err(|_| SealError::NotABackup)?
        .read_to_string(&mut text)?;
    let manifest: BackupManifest =
        serde_json::from_str(&text).map_err(|_| SealError::NotABackup)?;
    if manifest.format != 1 || version(&manifest.version) > version(env!("CARGO_PKG_VERSION")) {
        return Err(SealError::Newer);
    }
    zip.by_name(DATABASE_ENTRY)
        .map_err(|_| SealError::NotABackup)?;
    Ok(manifest)
}

/// The owner said yes: the staged backup replaces everything on the next
/// start.
pub fn confirm(paths: &Paths) -> io::Result<()> {
    let dir = restore_dir(paths);
    if !dir.join(STAGED).is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "no backup is ready to restore; open it again",
        ));
    }
    fs::write(dir.join(READY), b"")
}

/// The owner changed their mind: the staged backup goes.
pub fn cancel(paths: &Paths) {
    let _ = fs::remove_dir_all(restore_dir(paths));
}

/// On start, before the database opens: swaps in a confirmed backup. The
/// database and the crews' folders that were there move aside, to
/// `<home>\before-restore\<time>` and `<workspaces_root>-before-restore-<time>`.
/// Returns where the old data went, if a backup was applied.
pub fn apply_pending(paths: &Paths, stamp: &str) -> io::Result<Option<PathBuf>> {
    let dir = restore_dir(paths);
    if !dir.join(READY).is_file() {
        return Ok(None);
    }
    // Once only: a backup that cannot be applied is not tried forever.
    fs::remove_file(dir.join(READY))?;
    let unpacked = dir.join(UNPACKED);
    let _ = fs::remove_dir_all(&unpacked);
    unpack(&dir.join(STAGED), &unpacked)?;

    let aside = paths.home.join("before-restore").join(stamp);
    fs::create_dir_all(&aside)?;
    let db = paths.db();
    for suffix in ["", "-wal", "-shm"] {
        let file = PathBuf::from(format!("{}{suffix}", db.display()));
        if file.exists() {
            let name = file.file_name().ok_or_else(|| other("no file name"))?;
            fs::rename(&file, aside.join(name))?;
        }
    }
    let root = &paths.workspaces_root;
    if root.exists() {
        let name = root.file_name().ok_or_else(|| other("no folder name"))?;
        let moved =
            root.with_file_name(format!("{}-before-restore-{stamp}", name.to_string_lossy()));
        move_dir(root, &moved)?;
        info!(to = %moved.display(), "the crews' folders were moved aside");
    }
    let workspaces = unpacked.join(WORKSPACES_ENTRY);
    if workspaces.exists() {
        move_dir(&workspaces, root)?;
    } else {
        fs::create_dir_all(root)?;
    }
    fs::rename(unpacked.join(DATABASE_ENTRY), &db)?;
    if let Err(err) = fs::remove_dir_all(&dir) {
        warn!("could not clean up after restoring: {err}");
    }
    Ok(Some(aside))
}

/// Every entry of the archive under `to`, refusing any that would land
/// outside it.
fn unpack(archive: &Path, to: &Path) -> io::Result<()> {
    let mut zip = ZipArchive::new(File::open(archive)?).map_err(other)?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(other)?;
        let Some(relative) = entry.enclosed_name() else {
            return Err(other("the backup has a file outside its folders"));
        };
        let target = to.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&target)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        io::copy(&mut entry, &mut BufWriter::new(File::create(&target)?))?;
    }
    Ok(())
}

/// Renames, or copies and removes when the two are on different drives.
fn move_dir(from: &Path, to: &Path) -> io::Result<()> {
    if fs::rename(from, to).is_ok() {
        return Ok(());
    }
    copy_dir(from, to)?;
    fs::remove_dir_all(from)
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let kind = entry.file_type()?;
        let target = to.join(entry.file_name());
        if kind.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else if kind.is_file() {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_compare_by_number() {
        assert!(version("0.10.0") > version("0.9.1"));
        assert!(version("1.0.0") > version("0.10.0"));
        assert_eq!(version("0.10.0"), version("0.10.0"));
    }
}
