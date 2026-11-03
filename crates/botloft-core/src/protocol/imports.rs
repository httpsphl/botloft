//! Bringing the owner's history to the Maker (spec 36.5, 36.6, 36.7): a list of
//! what there is to bring, a summary the owner reads before anything goes,
//! and sending it to the Maker as a file.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::{ImportId, MessageId};

/// The most a summary may hold, in characters; past it, the oldest
/// projects leave it (spec 36.7).
pub const IMPORT_MAX_CHARS: usize = 200_000;

/// Where a history comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum ImportSource {
    /// Claude Code's history on this computer (spec 36.5).
    ClaudeCode,
    /// The owner's export of claude.ai, a ZIP they downloaded (spec 36.6).
    ClaudeAi,
}

impl ImportSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ClaudeCode => "claude_code",
            Self::ClaudeAi => "claude_ai",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ImportsScanParams {
    pub source: ImportSource,
    /// The ZIP of a claude.ai export, where it is; read there, never copied.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub path: Option<String>,
}

/// What a claude.ai export holds, as the list shows it: no text of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ImportConversations {
    pub count: u32,
    /// Unix time in milliseconds of the first and last conversation.
    pub first_at: Option<i64>,
    pub last_at: Option<i64>,
    /// The names of the claude.ai projects, when the export has them.
    pub projects: Vec<String>,
}

/// One project of the history, as the list shows it: no text of it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ImportProject {
    /// What `imports.create` takes to bring it.
    pub key: String,
    /// The folder the project was worked in.
    pub path: String,
    pub sessions: u32,
    /// How many requests the owner typed in it.
    pub prompts: u32,
    /// Unix time in milliseconds of its first and last request; `null`
    /// without any.
    pub first_at: Option<i64>,
    pub last_at: Option<i64>,
}

/// What there is to bring, newest project first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ImportScan {
    pub source: ImportSource,
    /// Claude Code's projects; empty for an export of claude.ai.
    pub projects: Vec<ImportProject>,
    /// What an export of claude.ai holds; `null` for Claude Code.
    pub conversations: Option<ImportConversations>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ImportsCreateParams {
    pub source: ImportSource,
    /// Claude Code's projects to bring, by `key`; empty for claude.ai.
    #[serde(default)]
    pub keys: Vec<String>,
    /// The ZIP of a claude.ai export.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub path: Option<String>,
    /// For claude.ai, only the conversations started in this period, Unix
    /// time in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub from: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub to: Option<i64>,
}

/// A summary made and kept for the owner to read before it goes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ImportDraft {
    pub import_id: ImportId,
    /// The whole summary, in markdown, as the Maker would read it.
    pub text: String,
    pub chars: u32,
    /// About how many tokens of the owner's plan reading it takes.
    pub tokens_estimate: u32,
    /// Projects that left it to keep it under the limit, by path; for
    /// claude.ai, how many conversations left it.
    pub cut: Vec<String>,
    /// What was swapped out, by kind (`key`, `email`, `number`, `card`).
    pub redactions: BTreeMap<String, u32>,
    /// The period it covers, Unix time in milliseconds.
    pub from: Option<i64>,
    pub to: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ImportIdParams {
    pub import_id: ImportId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ImportSent {
    /// The owner's message to the Maker that carries the summary.
    pub message_id: MessageId,
}

/// A summary made, as the store keeps it: never its text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Import {
    pub id: ImportId,
    pub source: ImportSource,
    /// Unix time in milliseconds.
    pub created_at: i64,
    pub sent_at: Option<i64>,
    pub sent_message_id: Option<MessageId>,
    /// The projects it covers, by path.
    pub scope: Vec<String>,
    pub chars: u32,
    pub redactions: BTreeMap<String, u32>,
}
