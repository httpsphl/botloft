//! Reading Claude Code's history on this computer (spec 36.5), only for
//! the fields the summary takes: the requests the owner typed, the titles
//! of the sessions, which tools and models were used, and the folder of
//! each project. Answers, thinking, tool input and output, attachments and
//! images are never read into the summary.
//!
//! What Claude Code writes is not documented (spec 19). Seen on 2026-11-03:
//! `projects/<project>/<session>.jsonl` with `user` records that have
//! `cwd`, `timestamp`, `isMeta`, `promptSource` (`sdk` and `system` are not
//! the owner), `origin.kind` (`human` is the owner; `peer` and
//! `task-notification` are not) and `message.content` as a string or
//! blocks (`text`, `image`, `tool_result`); `assistant` records with
//! `message.model` and `tool_use` blocks; `custom-title` and `ai-title`
//! records. `history.jsonl` has `display`, `project`, `sessionId` and
//! `timestamp` (ms) for what was typed in the terminal.

use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde_json::Value;

/// A request the owner typed, cut to `PROMPT_MAX_CHARS`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub text: String,
    /// Unix time in milliseconds.
    pub at: i64,
}

/// One project of the history.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Project {
    /// Claude Code's folder name for it, under `projects`.
    pub key: String,
    /// The folder it was worked in.
    pub path: String,
    pub sessions: u32,
    pub prompts: Vec<Prompt>,
    pub titles: Vec<String>,
    /// Calls by tool; a server's tools as `mcp__<server>`.
    pub tools: BTreeMap<String, u32>,
    pub models: BTreeMap<String, u32>,
}

/// How much of one request the summary keeps.
pub const PROMPT_MAX_CHARS: usize = 300;

/// Claude Code's folder: `CLAUDE_CONFIG_DIR` from the owner's own
/// environment, else `.claude` in their profile.
pub fn root(environment: &[(String, String)], home: Option<PathBuf>) -> Option<PathBuf> {
    let set = |name: &str| {
        environment
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    };
    let root = set("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| set("USERPROFILE").map(|profile| PathBuf::from(profile).join(".claude")))
        .or_else(|| home.map(|home| home.join(".claude")))?;
    root.is_dir().then_some(root)
}

/// Claude Code's folder name for a project folder: every character that is
/// not a letter or digit becomes `-`.
pub fn key_of(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect()
}

/// `text` cut to `max` characters, on a character boundary.
pub fn cut(text: &str, max: usize) -> String {
    match text.char_indices().nth(max) {
        Some((at, _)) => format!("{}…", text[..at].trim_end()),
        None => text.to_owned(),
    }
}

fn millis(value: &Value) -> Option<i64> {
    match value {
        Value::Number(number) => number.as_i64(),
        Value::String(text) => text
            .parse::<jiff::Timestamp>()
            .ok()
            .map(|at| at.as_millisecond()),
        _ => None,
    }
}

/// The text of a request the owner typed, or `None` for anything else.
fn owner_text(record: &Value) -> Option<String> {
    if record.get("isMeta").and_then(Value::as_bool) == Some(true)
        || record.get("isSidechain").and_then(Value::as_bool) == Some(true)
    {
        return None;
    }
    if let Some(source) = record.get("promptSource").and_then(Value::as_str)
        && source != "typed"
    {
        return None;
    }
    if let Some(kind) = record.pointer("/origin/kind").and_then(Value::as_str)
        && kind != "human"
    {
        return None;
    }
    let text = match record.pointer("/message/content")? {
        Value::String(text) => text.clone(),
        Value::Array(blocks) => {
            if blocks.iter().any(|block| block["type"] == "tool_result") {
                return None;
            }
            let texts: Vec<&str> = blocks
                .iter()
                .filter(|block| block["type"] == "text")
                .filter_map(|block| block["text"].as_str())
                .collect();
            texts.join("\n")
        }
        _ => return None,
    };
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    // Commands and notes Claude Code adds come wrapped in tags.
    if text.is_empty() || text.starts_with('<') {
        return None;
    }
    Some(cut(&text, PROMPT_MAX_CHARS))
}

fn tool_name(name: &str) -> String {
    let mut parts = name.splitn(3, "__");
    match (parts.next(), parts.next(), parts.next()) {
        (Some("mcp"), Some(server), Some(_)) => format!("mcp__{server}"),
        _ => name.to_owned(),
    }
}

/// Reads one session into `project`. With `full`, also titles, tools and models.
fn read_session(file: &Path, project: &mut Project, full: bool) {
    let Ok(reader) = File::open(file).map(BufReader::new) else {
        return;
    };
    let mut buf = Vec::new();
    let mut reader = reader;
    loop {
        buf.clear();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        let Ok(line) = std::str::from_utf8(&buf) else {
            continue;
        };
        let user = line.contains("\"type\":\"user\"");
        let assistant = full && line.contains("\"type\":\"assistant\"");
        let title = full && line.contains("-title\"");
        if !(user || assistant || title) {
            continue;
        }
        let Ok(record) = serde_json::from_str::<Value>(line) else {
            continue;
        };
        match record["type"].as_str() {
            Some("user") => {
                if project.path.is_empty()
                    && let Some(cwd) = record["cwd"].as_str()
                {
                    project.path = cwd.to_owned();
                }
                if let (Some(text), Some(at)) = (owner_text(&record), millis(&record["timestamp"]))
                {
                    project.prompts.push(Prompt { text, at });
                }
            }
            Some("assistant") => {
                if let Some(model) = record.pointer("/message/model").and_then(Value::as_str) {
                    *project.models.entry(model.to_owned()).or_default() += 1;
                }
                if let Some(blocks) = record.pointer("/message/content").and_then(Value::as_array) {
                    for block in blocks.iter().filter(|block| block["type"] == "tool_use") {
                        if let Some(name) = block["name"].as_str() {
                            *project.tools.entry(tool_name(name)).or_default() += 1;
                        }
                    }
                }
            }
            Some("custom-title") | Some("ai-title") => {
                let title = record["customTitle"]
                    .as_str()
                    .or(record["aiTitle"].as_str());
                if let Some(title) = title.map(str::trim).filter(|title| !title.is_empty())
                    && !project.titles.iter().any(|seen| seen == title)
                {
                    project.titles.push(title.to_owned());
                }
            }
            _ => {}
        }
    }
}

fn sessions_in(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().is_some_and(|ext| ext == "jsonl"))
        .collect()
}

/// What `history.jsonl` adds: requests typed in sessions whose file is gone.
fn from_history(root: &Path, projects: &mut BTreeMap<String, Project>) {
    let Ok(file) = File::open(root.join("history.jsonl")) else {
        return;
    };
    for line in BufReader::new(file).lines().map_while(Result::ok) {
        let Ok(record) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        let (Some(path), Some(display), Some(at)) = (
            record["project"].as_str(),
            record["display"].as_str(),
            millis(&record["timestamp"]),
        ) else {
            continue;
        };
        let key = key_of(path);
        let session = record["sessionId"].as_str().unwrap_or_default();
        let dir = root.join("projects").join(&key);
        if !session.is_empty() && dir.join(format!("{session}.jsonl")).is_file() {
            continue;
        }
        let text = display.split_whitespace().collect::<Vec<_>>().join(" ");
        if text.is_empty() || text.starts_with('/') || text.starts_with('<') {
            continue;
        }
        let project = projects.entry(key.clone()).or_insert_with(|| Project {
            key,
            path: path.to_owned(),
            ..Project::default()
        });
        project.prompts.push(Prompt {
            text: cut(&text, PROMPT_MAX_CHARS),
            at,
        });
    }
}

fn normalized(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_lowercase()
}

/// Whether `path` is one of `excluded` or inside one: Botloft's own folders
/// (its agents work there) and temporary folders.
pub fn excluded(path: &str, excluded: &[String]) -> bool {
    let path = normalized(path);
    excluded
        .iter()
        .map(|dir| normalized(dir))
        .any(|dir| !dir.is_empty() && (path == dir || path.starts_with(&format!("{dir}/"))))
        || path.contains("/appdata/local/temp/")
}

/// Every project with at least one request of the owner, or only those in
/// `keys` when given. Without `full`, only what the list needs.
pub fn read(
    root: &Path,
    keys: Option<&HashSet<String>>,
    full: bool,
    skip: &[String],
) -> Vec<Project> {
    let mut projects: BTreeMap<String, Project> = BTreeMap::new();
    if let Ok(entries) = fs::read_dir(root.join("projects")) {
        for entry in entries.filter_map(Result::ok) {
            let key = entry.file_name().to_string_lossy().into_owned();
            if keys.is_some_and(|keys| !keys.contains(&key)) || !entry.path().is_dir() {
                continue;
            }
            let mut project = Project {
                key: key.clone(),
                ..Project::default()
            };
            let sessions = sessions_in(&entry.path());
            project.sessions = sessions.len() as u32;
            for session in &sessions {
                read_session(session, &mut project, full);
            }
            projects.insert(key, project);
        }
    }
    from_history(root, &mut projects);
    projects
        .into_values()
        .filter(|project| keys.is_none_or(|keys| keys.contains(&project.key)))
        .filter(|project| !project.prompts.is_empty() && !project.path.is_empty())
        .filter(|project| !excluded(&project.path, skip))
        .map(|mut project| {
            project
                .prompts
                .sort_by_key(|prompt| std::cmp::Reverse(prompt.at));
            project
        })
        .collect()
}
