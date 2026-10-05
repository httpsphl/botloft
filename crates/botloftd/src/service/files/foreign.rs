//! A bot's files as a bot of another crew reaches them (spec 10.4): only in
//! its own folder and its crew's work folder, never a file it wrote
//! elsewhere, and never what Botloft keeps for it there (its memory, its
//! rules and settings, the owner's attachments).

use std::path::{Path, PathBuf};

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotFile, FilesListParams};

use super::{Places, list, places};
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// At the top of either folder, what bots load as memory and rules, and
/// what Botloft and the owner keep there: never another crew's to touch,
/// or it could give that crew's bots instructions.
const KEPT: [&str; 5] = [
    "CLAUDE.md",
    "CLAUDE.local.md",
    ".claude",
    ".botloft",
    "attachments",
];

/// The two folders, as they are on disk.
fn folders(places: &Places) -> Vec<PathBuf> {
    [&places.work, &places.workspace]
        .into_iter()
        .filter_map(|root| root.canonicalize().ok())
        .collect()
}

/// Whether `path` (real, absolute) is in one of the folders and not kept.
fn open_to_others(places: &Places, path: &Path) -> bool {
    folders(places).iter().any(|root| {
        path.strip_prefix(root).is_ok_and(|rest| {
            rest.components().next().is_some_and(|first| {
                let first = first.as_os_str();
                !KEPT.iter().any(|kept| first.eq_ignore_ascii_case(kept))
            })
        })
    })
}

/// The bot's files another crew may see: those of `files.list` in its
/// folders, kept ones aside.
pub(crate) fn listed(daemon: &Daemon, owner: &BotId) -> ApiResult<Vec<BotFile>> {
    let (_, places) = places(daemon, owner)?;
    Ok(list(
        daemon,
        FilesListParams {
            bot_id: owner.clone(),
        },
    )?
    .into_iter()
    .filter(|file| {
        Path::new(&file.path)
            .canonicalize()
            .is_ok_and(|real| open_to_others(&places, &real))
    })
    .collect())
}

/// An existing file of the bot's another crew may read, and its size.
pub(crate) fn readable(daemon: &Daemon, owner: &BotId, path: &str) -> ApiResult<(PathBuf, u64)> {
    let (_, places) = places(daemon, owner)?;
    let asked = absolute(path)?;
    let real = asked
        .canonicalize()
        .ok()
        .filter(|real| open_to_others(&places, real))
        .ok_or_else(|| out_of_reach(path))?;
    let meta = std::fs::metadata(&real).map_err(|_| out_of_reach(path))?;
    if !meta.is_file() {
        return Err(out_of_reach(path));
    }
    Ok((real, meta.len()))
}

/// Where another crew's bot may write: an existing file of the bot's, or a
/// new one in a folder that already exists in its folders.
pub(crate) fn writable(daemon: &Daemon, owner: &BotId, path: &str) -> ApiResult<PathBuf> {
    let (_, places) = places(daemon, owner)?;
    let asked = absolute(path)?;
    if let Ok(real) = asked.canonicalize() {
        return (real.is_file() && open_to_others(&places, &real))
            .then_some(real)
            .ok_or_else(|| out_of_reach(path));
    }
    let name = asked.file_name().ok_or_else(|| out_of_reach(path))?;
    let parent = asked
        .parent()
        .and_then(|parent| parent.canonicalize().ok())
        .filter(|parent| parent.is_dir())
        .ok_or_else(|| out_of_reach(path))?;
    let new = parent.join(name);
    open_to_others(&places, &new)
        .then_some(new)
        .ok_or_else(|| out_of_reach(path))
}

fn absolute(path: &str) -> ApiResult<PathBuf> {
    let path = Path::new(path.trim());
    if !path.is_absolute() {
        return Err(ApiError::validation(
            "give the file's full path, as crew_files lists it",
        ));
    }
    Ok(path.to_path_buf())
}

fn out_of_reach(path: &str) -> ApiError {
    ApiError::NotFound(format!(
        "{} is not a file you can reach in that crew; call crew_files to see which",
        path.trim()
    ))
}
