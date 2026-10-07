//! Exporting a backup (spec 14.2): a copy of the database and of every
//! crew's folder in `workspaces_root`, zipped and sealed with the owner's
//! passphrase. Work folders the owner chose elsewhere are theirs and stay
//! out; so do the files Botloft writes again on every start.

use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Write};
use std::path::{Component, Path, PathBuf};

use std::collections::HashSet;

use botloft_core::protocol::{BackupCrew, BackupExported, BackupManifest, BackupScope};
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

/// The files of a bot's folder that the light copy takes: its memory and its
/// rules. `relative` starts with the crew's slug, and only a bot in `bots`
/// (`crew/bot`, lower case) can have any.
fn light_allows(relative: &Path, is_dir: bool, bots: &HashSet<String>) -> bool {
    let names: Vec<String> = relative
        .components()
        .map(|part| part.as_os_str().to_string_lossy().to_ascii_lowercase())
        .collect();
    let is = |index: usize, name: &str| names.get(index).is_some_and(|part| part == name);
    let bot = names.len() >= 2 && bots.contains(&format!("{}/{}", names[0], names[1]));
    if is_dir {
        return match names.len() {
            0 | 1 => true,
            2 => bot,
            3 => is(2, ".claude"),
            4 => is(2, ".claude") && is(3, "rules"),
            _ => true,
        };
    }
    match names.len() {
        3 => bot && (is(2, "claude.md") || is(2, "claude.local.md")),
        len if len >= 5 => bot && is(2, ".claude") && is(3, "rules"),
        _ => false,
    }
}

/// Writes the sealed backup in `<home>\exports` and says what it holds.
/// The app then lets the owner save it where they want. The light copy for
/// the cloud (spec 27) goes in a folder of its own, so it never replaces a
/// full export the owner has not saved yet.
pub fn export(
    daemon: &Daemon,
    passphrase: &str,
    scope: BackupScope,
) -> Result<BackupExported, SealError> {
    let folder = match scope {
        BackupScope::Full => "exports",
        BackupScope::Light => "cloud-upload",
    };
    export_to(daemon, passphrase, scope, &daemon.paths.home.join(folder))
}

fn export_to(
    daemon: &Daemon,
    passphrase: &str,
    scope: BackupScope,
    dir: &Path,
) -> Result<BackupExported, SealError> {
    if passphrase.chars().count() < PASSPHRASE_MIN {
        return Err(SealError::ShortPassphrase);
    }
    let dir = dir.to_path_buf();
    // Only the newest export is kept: the owner saved the earlier ones.
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    let snapshot = dir.join("snapshot.db");
    let mut light_bots = HashSet::new();
    let (crews, manifest) = {
        let store = daemon.store();
        match scope {
            BackupScope::Full => store.snapshot_to(&snapshot),
            BackupScope::Light => store.snapshot_light_to(&snapshot),
        }
        .map_err(|err| io::Error::other(err.to_string()))?;
        let crews = store
            .crews(true)
            .map_err(|err| io::Error::other(err.to_string()))?;
        let mut listed = Vec::new();
        for crew in &crews {
            let bots = store
                .bots(Some(&crew.id), true)
                .map_err(|err| io::Error::other(err.to_string()))?;
            light_bots.extend(
                bots.iter()
                    .map(|bot| format!("{}/{}", crew.slug, bot.slug).to_ascii_lowercase()),
            );
            listed.push(BackupCrew {
                name: crew.name.clone(),
                bots: bots
                    .into_iter()
                    .filter(|bot| bot.archived_at.is_none())
                    .map(|bot| bot.name)
                    .collect(),
                work_folder: crew.work_folder_chosen.then(|| crew.work_folder.clone()),
            });
        }
        let manifest = BackupManifest {
            format: 1,
            created_at: daemon.clock.now_ms(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            scope,
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
            let filter = Filter {
                scope,
                bots: &light_bots,
            };
            add_folder(
                &mut zip,
                &folder,
                &PathBuf::from(&crew.slug),
                options,
                &filter,
            )?;
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

/// What a backup of this scope takes of the folders.
struct Filter<'a> {
    scope: BackupScope,
    bots: &'a HashSet<String>,
}

impl Filter<'_> {
    fn takes(&self, relative: &Path, is_dir: bool) -> bool {
        if regenerated(relative) {
            return false;
        }
        match self.scope {
            BackupScope::Full => true,
            BackupScope::Light => light_allows(relative, is_dir, self.bots),
        }
    }
}

/// Every file under `folder` the scope takes, as `workspaces/<inside>/...`.
fn add_folder(
    zip: &mut ZipWriter<BufWriter<File>>,
    folder: &Path,
    inside: &Path,
    options: SimpleFileOptions,
    filter: &Filter<'_>,
) -> io::Result<()> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let kind = entry.file_type()?;
        let relative = inside.join(entry.file_name());
        if kind.is_symlink() || !filter.takes(&relative, kind.is_dir()) {
            continue;
        }
        if kind.is_dir() {
            add_folder(zip, &entry.path(), &relative, options, filter)?;
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

    #[test]
    fn the_light_copy_takes_only_a_bots_memory_and_rules() {
        let bots: HashSet<String> = ["site/scout".to_owned()].into();
        let takes = |path: &str, dir: bool| light_allows(Path::new(path), dir, &bots);
        assert!(takes("site/scout", true));
        assert!(takes("site/scout/CLAUDE.md", false));
        assert!(takes("site/scout/claude.local.md", false));
        assert!(takes("site/scout/.claude", true));
        assert!(takes("site/scout/.claude/rules", true));
        assert!(takes("site/scout/.claude/rules/botloft.md", false));
        assert!(takes("site/scout/.claude/rules/more/x.md", false));
        // The rest of the bot's folder, the shared folder and other files.
        assert!(!takes("site/shared", true));
        assert!(!takes("site/shared/CLAUDE.md", false));
        assert!(!takes("site/scout/node_modules", true));
        assert!(!takes("site/scout/attachments", true));
        assert!(!takes("site/scout/notes.md", false));
        assert!(!takes("site/scout/.claude/projects", true));
        assert!(!takes("site/scout/.claude/settings.json", false));
        assert!(!takes("site/CLAUDE.md", false));
    }
}
