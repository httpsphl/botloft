//! What a bot does in another crew once the owner let it in (spec 10.4):
//! `crew_roster` with `crew`, and `crew_files`, `read_crew_file` and
//! `write_crew_file`. A crew or a file out of reach reads like a missing
//! one.

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotsListParams, Crew};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::service::crew_access::{self, Need};
use crate::service::files::{foreign_files, foreign_readable, foreign_writable};
use crate::service::{ApiError, ApiResult, bots, lead};
use crate::state::Daemon;

pub const CREW_FILES: &str = "crew_files";
pub const READ_CREW_FILE: &str = "read_crew_file";
pub const WRITE_CREW_FILE: &str = "write_crew_file";

/// Most files `crew_files` lists, newest first.
const LIST_MAX: usize = 200;
/// Largest file read or written as text, in bytes.
const TEXT_MAX: u64 = 1024 * 1024;

pub(super) fn tools() -> [Value; 3] {
    let crew = json!({ "type": "string", "description": "The other crew's name." });
    let path = json!({
        "type": "string",
        "description": "The file's full path, as crew_files lists it.",
    });
    [
        json!({
            "name": CREW_FILES,
            "title": "Another crew's files",
            "description": "Lists the files of another crew you were let in to read \
                (ask_crew_access with `read`): of one bot, or of every bot you reach there, \
                with its crew's work folder. Newest first: full path, name, size, when it changed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "crew": crew,
                    "bot": { "type": "string", "description": "Only this bot's files (its handle)." },
                },
                "required": ["crew"],
                "additionalProperties": false,
            },
            "annotations": { "readOnlyHint": true },
        }),
        json!({
            "name": READ_CREW_FILE,
            "title": "Read another crew's file",
            "description": "Reads, as text, a file crew_files listed, in a crew you were let in to \
                read. Up to 1 MB.",
            "inputSchema": {
                "type": "object",
                "properties": { "crew": crew, "path": path },
                "required": ["crew", "path"],
                "additionalProperties": false,
            },
            "annotations": { "readOnlyHint": true },
        }),
        json!({
            "name": WRITE_CREW_FILE,
            "title": "Write another crew's file",
            "description": "Replaces a file of another crew you were let in to edit \
                (ask_crew_access with `edit`), or adds one in a folder that already exists there. \
                The whole new text, up to 1 MB. Tell the bots of that crew what you changed.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "crew": crew,
                    "path": path,
                    "content": { "type": "string", "description": "The file's whole new text." },
                },
                "required": ["crew", "path", "content"],
                "additionalProperties": false,
            },
        }),
    ]
}

/// The crew by name and the bots of it whose files the caller may `need`.
fn owners(daemon: &Daemon, me: &BotId, name: &str, need: Need) -> ApiResult<(Crew, Vec<BotId>)> {
    let store = daemon.store();
    let (own, _) = bots::active(&store, me)?;
    let crew = crew_access::other_crew(&store, &own.id, name)?;
    let mut owners = Vec::new();
    for bot in store.bots(Some(&crew.id), false)? {
        if crew_access::reaches(daemon, &store, me, &crew.id, Some(&bot.id), need)? {
            owners.push(bot.id);
        }
    }
    if owners.is_empty() {
        let what = if need == Need::Edit { "edit" } else { "read" };
        return Err(ApiError::Conflict(format!(
            "you cannot {what} the files of the crew {}; if the owner wants you to, call \
             ask_crew_access with `{what}` and why",
            crew.name
        )));
    }
    Ok((crew, owners))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct FilesArgs {
    crew: String,
    #[serde(default)]
    bot: Option<String>,
}

pub(super) fn files(daemon: &Daemon, me: &BotId, args: FilesArgs) -> ApiResult<Value> {
    let (crew, mut owners) = owners(daemon, me, &args.crew, Need::Read)?;
    if let Some(handle) = &args.bot {
        let handle = botloft_core::slug::slugify(handle.trim().trim_start_matches('@'), "");
        let id = daemon.store().active_bot_by_handle(&crew.id, &handle)?;
        owners.retain(|owner| Some(owner) == id.as_ref());
        if owners.is_empty() {
            return Err(ApiError::NotFound(format!(
                "you cannot read the files of @{handle} in the crew {}",
                crew.name
            )));
        }
    }
    let mut found = Vec::new();
    for owner in &owners {
        for file in foreign_files(daemon, owner)? {
            if !found
                .iter()
                .any(|seen: &botloft_core::protocol::BotFile| seen.path == file.path)
            {
                found.push(file);
            }
        }
    }
    found.sort_by_key(|file| std::cmp::Reverse(file.modified_at));
    found.truncate(LIST_MAX);
    let now = daemon.clock.now_ms();
    let files: Vec<Value> = found
        .iter()
        .map(|file| {
            json!({
                "path": file.path,
                "name": file.name,
                "size": file.size,
                "changed": ago(file.modified_at, now),
            })
        })
        .collect();
    Ok(json!({ "crew": crew.name, "files": files }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ReadArgs {
    crew: String,
    path: String,
}

pub(super) fn read(daemon: &Daemon, me: &BotId, args: ReadArgs) -> ApiResult<Value> {
    let (crew, owners) = owners(daemon, me, &args.crew, Need::Read)?;
    let (path, size) = first(&owners, |owner| foreign_readable(daemon, owner, &args.path))?;
    if size > TEXT_MAX {
        return Err(ApiError::validation("the file is larger than 1 MB"));
    }
    let bytes = std::fs::read(&path).map_err(|_| ApiError::NotFound(args.path.clone()))?;
    let text = String::from_utf8(bytes).map_err(|_| {
        ApiError::validation("the file is not text; only text files can be read this way")
    })?;
    Ok(json!({ "crew": crew.name, "path": path.display().to_string(), "text": text }))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct WriteArgs {
    crew: String,
    path: String,
    content: String,
}

pub(super) fn write(daemon: &Daemon, me: &BotId, args: WriteArgs) -> ApiResult<Value> {
    if args.content.len() as u64 > TEXT_MAX {
        return Err(ApiError::validation("content must be at most 1 MB"));
    }
    let (crew, owners) = owners(daemon, me, &args.crew, Need::Edit)?;
    let path = first(&owners, |owner| foreign_writable(daemon, owner, &args.path))?;
    std::fs::write(&path, args.content.as_bytes())
        .map_err(|err| ApiError::validation(format!("could not write the file: {err}")))?;
    Ok(json!({
        "crew": crew.name,
        "path": path.display().to_string(),
        "note": "Written. Tell the bots of that crew what you changed.",
    }))
}

/// How long ago, for a reader who does not know the current time.
fn ago(at_ms: i64, now_ms: i64) -> String {
    let minutes = now_ms.saturating_sub(at_ms).max(0) / 60_000;
    match minutes {
        0 => "just now".to_owned(),
        1..=89 => format!("{minutes} min ago"),
        90..=2879 => format!("{} h ago", (minutes + 30) / 60),
        _ => format!("{} days ago", minutes / 1440),
    }
}

/// The first owner for whom `try_one` works, or the last error.
fn first<T>(owners: &[BotId], try_one: impl Fn(&BotId) -> ApiResult<T>) -> ApiResult<T> {
    let mut last = None;
    for owner in owners {
        match try_one(owner) {
            Ok(found) => return Ok(found),
            Err(err) => last = Some(err),
        }
    }
    Err(last.unwrap_or_else(|| ApiError::NotFound("no file".into())))
}

/// `crew_roster` with `crew`: the bots of another crew the caller may reach.
pub(super) fn roster(daemon: &Daemon, me: &BotId, name: &str) -> ApiResult<Value> {
    let (crew, mine) = {
        let store = daemon.store();
        let (own, mine) = bots::active(&store, me)?;
        (crew_access::other_crew(&store, &own.id, name)?, mine)
    };
    let list = bots::list(
        daemon,
        BotsListParams {
            crew_id: Some(crew.id.clone()),
        },
    )?;
    let store = daemon.store();
    let mut reachable = Vec::new();
    for bot in list {
        let record = bots::find(&store, &bot.id)?;
        if crew_access::may_talk(daemon, &store, &mine, &record)? {
            reachable.push(json!({
                "handle": bot.handle,
                "name": bot.name,
                "role": bot.role,
                "state": bot.state,
                "chief": lead::is_lead(&crew, &bot.id),
            }));
        }
    }
    if reachable.is_empty() {
        return Err(ApiError::Conflict(format!(
            "you cannot reach the crew {}; if the owner wants you to, call ask_crew_access with \
             what you need and why",
            crew.name
        )));
    }
    Ok(json!({
        "crew": crew.name,
        "bots": reachable,
        "note": format!("Write with send_message(to, crew: \"{}\").", crew.name),
    }))
}
