//! Reading the owner's export of claude.ai (spec 36.6): the ZIP claude.ai
//! sends by e-mail, opened where it is, without extracting it nor copying
//! it. Only `conversations.json` (and `projects.json`, for the projects'
//! names) is read, and of each conversation only its title, its dates, how
//! many messages it has and the first three messages of the owner. The
//! answers, the attachments and the rest of the ZIP are never read.
//!
//! The format is not documented and changes (spec 19). Read as seen in
//! exports described on 2026-11-03: `conversations.json` is a list of
//! conversations with `name`, `created_at`, `updated_at` and
//! `chat_messages`, each message with `sender` (`human` or `assistant`),
//! `text` (or `content` blocks of `type` `text`) and `created_at`. A
//! conversation missing what the summary needs leaves it; a file that is
//! not such a list is refused, with no half summary.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use serde::Deserialize;

use super::claude_code::{PROMPT_MAX_CHARS, cut};

/// The largest ZIP read (spec 36.6).
const ZIP_MAX_BYTES: u64 = 4 * 1024 * 1024 * 1024;
/// The owner's messages kept per conversation.
const FIRST_MESSAGES: usize = 3;

/// Why an export cannot be read.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ExportError {
    #[error("the file cannot be opened")]
    Unreadable,
    #[error("the file is larger than 4 GB")]
    TooLarge,
    #[error("this is not an export of claude.ai: it has no conversations.json")]
    NotAnExport,
    #[error("conversations.json is not in the format of claude.ai")]
    Format,
}

/// One conversation, as the summary takes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conversation {
    pub title: String,
    /// Unix time in milliseconds.
    pub started_at: i64,
    pub messages: u32,
    /// The owner's first messages, cut to `PROMPT_MAX_CHARS`.
    pub first: Vec<String>,
}

/// What an export holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Export {
    /// Newest first.
    pub conversations: Vec<Conversation>,
    pub projects: Vec<String>,
}

#[derive(Deserialize)]
struct RawBlock {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    #[serde(default)]
    text: Option<String>,
}

#[derive(Deserialize)]
struct RawMessage {
    #[serde(default)]
    sender: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    content: Vec<RawBlock>,
}

#[derive(Deserialize)]
struct RawConversation {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    chat_messages: Option<Vec<RawMessage>>,
}

#[derive(Deserialize)]
struct RawProject {
    #[serde(default)]
    name: Option<String>,
}

fn text_of(message: &RawMessage) -> String {
    let text = match message
        .text
        .as_deref()
        .filter(|text| !text.trim().is_empty())
    {
        Some(text) => text.to_owned(),
        None => message
            .content
            .iter()
            .filter(|block| block.kind.as_deref() == Some("text"))
            .filter_map(|block| block.text.as_deref())
            .collect::<Vec<_>>()
            .join("\n"),
    };
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn conversation(raw: RawConversation) -> Option<Conversation> {
    let started_at = raw
        .created_at?
        .parse::<jiff::Timestamp>()
        .ok()?
        .as_millisecond();
    let messages = raw.chat_messages?;
    let first: Vec<String> = messages
        .iter()
        .filter(|message| message.sender.as_deref() == Some("human"))
        .map(text_of)
        .filter(|text| !text.is_empty())
        .take(FIRST_MESSAGES)
        .map(|text| cut(&text, PROMPT_MAX_CHARS))
        .collect();
    let title = raw
        .name
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| first.first().map(|text| cut(text, 60)).unwrap_or_default());
    if first.is_empty() && title.is_empty() {
        return None;
    }
    Some(Conversation {
        title,
        started_at,
        messages: messages.len() as u32,
        first,
    })
}

/// The entry of the ZIP whose name ends with `name`, wherever it sits.
fn entry_named(archive: &zip::ZipArchive<BufReader<File>>, name: &str) -> Option<String> {
    archive
        .file_names()
        .find(|entry| entry.rsplit('/').next() == Some(name))
        .map(str::to_owned)
}

/// Reads the export in the ZIP at `path`.
pub fn read(path: &Path) -> Result<Export, ExportError> {
    let size = std::fs::metadata(path)
        .map_err(|_| ExportError::Unreadable)?
        .len();
    if size > ZIP_MAX_BYTES {
        return Err(ExportError::TooLarge);
    }
    let file = File::open(path).map_err(|_| ExportError::Unreadable)?;
    let mut archive =
        zip::ZipArchive::new(BufReader::new(file)).map_err(|_| ExportError::NotAnExport)?;
    let name = entry_named(&archive, "conversations.json").ok_or(ExportError::NotAnExport)?;
    let raw: Vec<serde_json::Value> = {
        let entry = archive
            .by_name(&name)
            .map_err(|_| ExportError::NotAnExport)?;
        serde_json::from_reader(BufReader::new(entry)).map_err(|_| ExportError::Format)?
    };
    let mut conversations: Vec<Conversation> = raw
        .into_iter()
        .filter_map(|value| serde_json::from_value::<RawConversation>(value).ok())
        .filter_map(conversation)
        .collect();
    conversations.sort_by_key(|conversation| std::cmp::Reverse(conversation.started_at));
    let mut projects = Vec::new();
    if let Some(name) = entry_named(&archive, "projects.json")
        && let Ok(mut entry) = archive.by_name(&name)
    {
        let mut text = String::new();
        if entry.read_to_string(&mut text).is_ok()
            && let Ok(list) = serde_json::from_str::<Vec<RawProject>>(&text)
        {
            projects = list
                .into_iter()
                .filter_map(|project| project.name)
                .map(|name| name.trim().to_owned())
                .filter(|name| !name.is_empty())
                .collect();
        }
    }
    Ok(Export {
        conversations,
        projects,
    })
}
