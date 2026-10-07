//! A hash of what the light copy would hold now (spec 27.10), so the
//! automatic backup sends a copy only after something the owner made changed:
//! the structure and the routines (`Store::light_fingerprint`) and each bot's
//! memory and rules, taken by the same filter as the copy itself.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use botloft_core::protocol::BackupScope;
use sha2::{Digest, Sha256};

use super::export::Filter;
use crate::state::Daemon;

/// The file the database copy is made in; it is gone when the hash is.
const SCRATCH: &str = "cloud-fingerprint.db";

pub fn light_fingerprint(daemon: &Daemon) -> io::Result<String> {
    let other = |err: botloft_store::StoreError| io::Error::other(err.to_string());
    let (database, crews, bots) = {
        let store = daemon.store();
        let database = store
            .light_fingerprint(&daemon.paths.home.join(SCRATCH))
            .map_err(other)?;
        let crews = store.crews(true).map_err(other)?;
        let mut bots = HashSet::new();
        for crew in &crews {
            for bot in store.bots(Some(&crew.id), true).map_err(other)? {
                bots.insert(format!("{}/{}", crew.slug, bot.slug).to_ascii_lowercase());
            }
        }
        (database, crews, bots)
    };
    let mut hash = Sha256::new();
    hash.update(database.as_bytes());
    let filter = Filter {
        scope: BackupScope::Light,
        bots: &bots,
    };
    let mut slugs: Vec<_> = crews.iter().map(|crew| crew.slug.as_str()).collect();
    slugs.sort_unstable();
    for slug in slugs {
        walk(
            &daemon.paths.crew_dir(slug),
            &PathBuf::from(slug),
            &filter,
            &mut hash,
        )?;
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

/// Every file the filter takes under `folder`, by name, into `hash`.
fn walk(folder: &Path, inside: &Path, filter: &Filter<'_>, hash: &mut Sha256) -> io::Result<()> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Ok(());
    };
    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let kind = entry.file_type()?;
        let relative = inside.join(entry.file_name());
        if kind.is_symlink() || !filter.takes(&relative, kind.is_dir()) {
            continue;
        }
        if kind.is_dir() {
            walk(&entry.path(), &relative, filter, hash)?;
        } else if kind.is_file() {
            hash.update(relative.to_string_lossy().replace('\\', "/").as_bytes());
            hash.update([0]);
            let mut file = File::open(entry.path())?;
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let read = file.read(&mut buffer)?;
                if read == 0 {
                    break;
                }
                hash.update(&buffer[..read]);
            }
            hash.update([0xFF]);
        }
    }
    Ok(())
}
