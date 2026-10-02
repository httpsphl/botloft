//! The files a bot made (spec 8.5): what the app's files panel lists and
//! previews. They are in the crew's work folder or the bot's own folder, or
//! wherever the bot's `Write`/`Edit` calls put them.

mod scan;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::ids::BotId;
use botloft_core::protocol::{BotFile, FileData, FilesListParams, FilesReadParams};
use botloft_store::BotRecord;

pub(crate) use self::scan::media_type;
use self::scan::{Found, Root, key, keys, millis, walk};
use super::{ApiError, ApiResult, bots, crews};
use crate::state::Daemon;

/// The newest files listed; the rest are older than anything the owner wants.
const LIST_MAX: usize = 200;
/// How many of the bot's own writes are looked up.
const WRITTEN_MAX: u32 = 200;
/// Largest file the app previews: base64 of it must fit a 32 MiB frame.
const PREVIEW_MAX_BYTES: u64 = 20 * 1024 * 1024;
/// Generated for the bot by the daemon: not something it made for the owner.
const WORKSPACE_SKIP: [&str; 2] = ["attachments", "CLAUDE.md"];

/// Where a bot's files live, and what it wrote by name.
struct Places {
    work: PathBuf,
    workspace: PathBuf,
    written: Vec<String>,
}

fn places(daemon: &Daemon, bot_id: &BotId) -> ApiResult<(BotRecord, Places)> {
    let store = daemon.store();
    let bot = bots::find(&store, bot_id)?;
    let crew = crews::find(&store, &bot.crew_id)?;
    let written = store.written_files(&bot.id, WRITTEN_MAX)?;
    drop(store);
    let places = Places {
        work: daemon.paths.work_folder(&crew),
        workspace: daemon.paths.bot_workspace(&crew.slug, &bot.slug),
        written,
    };
    Ok((bot, places))
}

/// `files.list`: the files in the work folder and the bot's folder that are
/// newer than the bot, plus every file its `Write`/`Edit` calls changed that
/// still exists. Newest first.
pub fn list(daemon: &Daemon, params: FilesListParams) -> ApiResult<Vec<BotFile>> {
    let (bot, places) = places(daemon, &params.bot_id)?;
    let written = keys(&places.written);
    let roots = [
        Root {
            path: &places.work,
            skip_top: &[],
        },
        Root {
            path: &places.workspace,
            skip_top: &WORKSPACE_SKIP,
        },
    ];
    let mut found: HashMap<String, Found> = HashMap::new();
    for root in &roots {
        for file in walk(root, bot.created_at) {
            found.insert(key(&file.path), file);
        }
    }
    for path in &places.written {
        let path = PathBuf::from(path);
        if let Ok(meta) = std::fs::metadata(&path)
            && meta.is_file()
        {
            found.entry(key(&path)).or_insert_with(|| Found {
                size: meta.len(),
                modified_at: millis(&meta),
                path,
            });
        }
    }
    let folders = [places.work.as_path(), places.workspace.as_path()];
    let mut files: Vec<BotFile> = found
        .into_iter()
        .map(|(id, file)| describe(file, written.contains(&id), &folders))
        .collect();
    files.sort_by(|a, b| {
        b.modified_at
            .cmp(&a.modified_at)
            .then_with(|| a.path.cmp(&b.path))
    });
    files.truncate(LIST_MAX);
    Ok(files)
}

fn describe(file: Found, written_by_bot: bool, roots: &[&Path]) -> BotFile {
    let name = file_name(&file.path);
    let parent = file.path.parent().unwrap_or(Path::new(""));
    let folder = roots
        .iter()
        .find_map(|root| parent.strip_prefix(root).ok())
        .unwrap_or(parent)
        .to_string_lossy()
        .into_owned();
    BotFile {
        media_type: media_type(&name).to_owned(),
        path: file.path.to_string_lossy().into_owned(),
        name,
        folder,
        size: file.size,
        modified_at: file.modified_at,
        written_by_bot,
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().into_owned())
}

/// Why a path is not one of the bot's files.
enum Refused {
    /// Nothing is there, or it is not a file.
    Gone,
    /// Outside the bot's folders, and the bot did not write it.
    Outside,
}

/// The real path of one of the bot's files: inside one of its two folders,
/// or written by its own calls. `asked` must be absolute.
fn reach(places: &Places, asked: &Path) -> Result<(PathBuf, std::fs::Metadata), Refused> {
    let path = asked.canonicalize().map_err(|_| Refused::Gone)?;
    let inside = [&places.work, &places.workspace]
        .into_iter()
        .filter_map(|root| root.canonicalize().ok())
        .any(|root| path.starts_with(root));
    let written = places
        .written
        .iter()
        .filter_map(|file| Path::new(file).canonicalize().ok())
        .any(|file| file == path);
    if !asked.is_absolute() || !(inside || written) {
        return Err(Refused::Outside);
    }
    let meta = std::fs::metadata(&path).map_err(|_| Refused::Gone)?;
    if !meta.is_file() {
        return Err(Refused::Gone);
    }
    Ok((path, meta))
}

/// `files.read`: the bytes of a file `files.list` can show. A path outside
/// the bot's folders that the bot did not write is refused, so the panel
/// cannot be used to read anything else on the computer.
pub fn read(daemon: &Daemon, params: FilesReadParams) -> ApiResult<FileData> {
    let (_, places) = places(daemon, &params.bot_id)?;
    let gone = || ApiError::NotFound(format!("{} is no longer there", params.path));
    let (path, meta) =
        reach(&places, Path::new(&params.path)).map_err(|refused| match refused {
            Refused::Gone => gone(),
            Refused::Outside => {
                ApiError::NotFound(format!("{} is not one of this bot's files", params.path))
            }
        })?;
    if meta.len() > PREVIEW_MAX_BYTES {
        return Err(ApiError::validation(format!(
            "the file is larger than {} MB",
            PREVIEW_MAX_BYTES / (1024 * 1024)
        )));
    }
    let bytes = std::fs::read(&path).map_err(|_| gone())?;
    Ok(FileData {
        media_type: media_type(&file_name(&path)).to_owned(),
        data: BASE64.encode(bytes),
    })
}

/// A file the bot shows the owner in the chat (`share_file`, spec 10): one
/// `files.read` can read, so the owner can preview and save it. A relative
/// path is in the bot's own folder, where it runs.
pub fn shared(daemon: &Daemon, bot_id: &BotId, path: &str) -> ApiResult<BotFile> {
    let (_, places) = places(daemon, bot_id)?;
    let asked = std::path::absolute(places.workspace.join(path.trim()))
        .map_err(|_| ApiError::NotFound(format!("{path} is not a file path")))?;
    let (_, meta) = reach(&places, &asked).map_err(|refused| {
        ApiError::NotFound(match refused {
            Refused::Gone => format!("There is no file at {}", asked.display()),
            Refused::Outside => format!(
                "{} is outside the work folder and your own folder: copy it into the work \
                 folder first, then share the copy",
                asked.display()
            ),
        })
    })?;
    let written = keys(&places.written).contains(&key(&asked));
    let found = Found {
        size: meta.len(),
        modified_at: millis(&meta),
        path: asked,
    };
    let folders = [places.work.as_path(), places.workspace.as_path()];
    Ok(describe(found, written, &folders))
}
