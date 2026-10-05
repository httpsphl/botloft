//! Exporting a backup (spec 14.2): a copy of the database and of every
//! crew's folder in `workspaces_root`, zipped and sealed with the owner's
//! passphrase. Work folders the owner chose elsewhere are theirs and stay
//! out; so do the files Botloft writes again on every start.

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

use botloft_core::protocol::{BackupCrew, BackupExported, BackupManifest};
use zip::ZipWriter;
use zip::write::SimpleFileOptions;

use super::seal::{PASSPHRASE_MIN, SealError, seal};
use super::{BACKUP_EXTENSION, DATABASE_ENTRY, MANIFEST_ENTRY, WORKSPACES_ENTRY};
use crate::state::Daemon;

/// What a bot's folder holds that Botloft writes again on every start
/// (spec 7.5): never worth carrying to another computer.
fn regenerated(relative: &Path) -> bool {
    let parts: Vec<_> = relative.components().collect();
    let named = |part: &Component<'_>, name: &str| part.as_os_str().eq_ignore_ascii_case(name);
    parts.iter().any(|part| named(part, ".botloft"))
        || parts
            .windows(2)
            .any(|pair| named(&pair[0], ".claude") && named(&pair[1], "settings.json"))
}

/// Writes the sealed backup in `<home>\exports` and says what it holds.
/// The app then lets the owner save it where they want.
pub fn export(daemon: &Daemon, passphrase: &str) -> Result<BackupExported, SealError> {
    if passphrase.chars().count() < PASSPHRASE_MIN {
        return Err(SealError::ShortPassphrase);
    }
    let dir = daemon.paths.home.join("exports");
    // Only the newest export is kept: the owner saved the earlier ones.
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    let snapshot = dir.join("snapshot.db");
    let (crews, manifest) = {
        let store = daemon.store();
        store
            .snapshot_to(&snapshot)
            .map_err(|err| io::Error::other(err.to_string()))?;
        let crews = store
            .crews(true)
            .map_err(|err| io::Error::other(err.to_string()))?;
        let mut listed = Vec::new();
        for crew in &crews {
            let bots = store
                .bots(Some(&crew.id), false)
                .map_err(|err| io::Error::other(err.to_string()))?;
            listed.push(BackupCrew {
                name: crew.name.clone(),
                bots: bots.into_iter().map(|bot| bot.name).collect(),
                work_folder: crew.work_folder_chosen.then(|| crew.work_folder.clone()),
            });
        }
        let manifest = BackupManifest {
            format: 1,
            created_at: daemon.clock.now_ms(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            crews: listed,
        };
        (crews, manifest)
    };

    let archive = dir.join("backup.zip");
    {
        let mut zip = ZipWriter::new(BufWriter::new(File::create(&archive)?));
        let options = SimpleFileOptions::default();
        zip.start_file(MANIFEST_ENTRY, options)
            .map_err(io::Error::other)?;
        zip.write_all(&serde_json::to_vec_pretty(&manifest).map_err(io::Error::other)?)?;
        zip.start_file(DATABASE_ENTRY, options)
            .map_err(io::Error::other)?;
        io::copy(&mut BufReader::new(File::open(&snapshot)?), &mut zip)?;
        for crew in &crews {
            let folder = daemon.paths.crew_dir(&crew.slug);
            add_folder(&mut zip, &folder, &PathBuf::from(&crew.slug), options)?;
        }
        zip.finish().map_err(io::Error::other)?.flush()?;
    }
    fs::remove_file(&snapshot)?;

    let name = format!("botloft-{}.{BACKUP_EXTENSION}", stamp(manifest.created_at));
    let sealed = dir.join(name);
    {
        let mut plain = BufReader::new(File::open(&archive)?);
        let mut out = BufWriter::new(File::create(&sealed)?);
        seal(&mut plain, &mut out, passphrase)?;
    }
    fs::remove_file(&archive)?;
    Ok(BackupExported {
        path: sealed.display().to_string(),
        size: fs::metadata(&sealed)?.len(),
        manifest,
    })
}

/// Every file under `folder`, as `workspaces/<inside>/...`.
fn add_folder(
    zip: &mut ZipWriter<BufWriter<File>>,
    folder: &Path,
    inside: &Path,
    options: SimpleFileOptions,
) -> io::Result<()> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let kind = entry.file_type()?;
        let relative = inside.join(entry.file_name());
        if kind.is_symlink() || regenerated(&relative) {
            continue;
        }
        if kind.is_dir() {
            add_folder(zip, &entry.path(), &relative, options)?;
        } else if kind.is_file() {
            let name = Path::new(WORKSPACES_ENTRY).join(&relative);
            let name = name.to_string_lossy().replace('\\', "/");
            zip.start_file(name, options).map_err(io::Error::other)?;
            io::copy(&mut BufReader::new(File::open(entry.path())?), zip)?;
        }
    }
    Ok(())
}

/// `2026-10-05-1430`, in local time, for the file name.
fn stamp(ms: i64) -> String {
    jiff::Timestamp::from_millisecond(ms)
        .map(|at| {
            at.to_zoned(jiff::tz::TimeZone::system())
                .strftime("%Y-%m-%d-%H%M")
                .to_string()
        })
        .unwrap_or_else(|_| "backup".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_botloft_writes_again_stays_out() {
        assert!(regenerated(Path::new("site/scout/.botloft/mcp.json")));
        assert!(regenerated(Path::new("site/scout/.claude/settings.json")));
        assert!(!regenerated(Path::new(
            "site/scout/.claude/rules/botloft.md"
        )));
        assert!(!regenerated(Path::new("site/scout/CLAUDE.md")));
        assert!(!regenerated(Path::new("site/shared/report.pdf")));
    }
}
