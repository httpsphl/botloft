//! `imports.*` (spec 36.5, 36.6, 36.7): the owner's Claude Code history or
//! claude.ai export (import_sources.rs), listed, summarized for them to
//! read, then sent to the Maker as a file of an owner's message, or
//! forgotten. A summary waits in `<home>\imports\`
//! until then, and leaves by itself after a day. Only ids, counts and
//! sizes are logged, in `debug`; the store keeps no text.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::ids::ImportId;
use botloft_core::protocol::{
    AttachmentUpload, Import, ImportDraft, ImportIdParams, ImportScan, ImportSent, ImportSource,
    ImportsCreateParams, ImportsScanParams, MessagesSendParams,
};
use tracing::debug;

use super::import_sources as sources;
use super::{ApiError, ApiResult, maker, messages};
use crate::imports::redact::{Labels, Redactor};
use crate::imports::words::words;
use crate::state::Daemon;

const DIR: &str = "imports";
/// How long a summary that was not sent waits.
const KEEP_MS: i64 = 24 * 60 * 60 * 1000;
const LIST_LIMIT: u32 = 50;

/// One summary is made at a time: reading the history takes a while.
static MAKING: AtomicBool = AtomicBool::new(false);

fn file(daemon: &Daemon, id: &ImportId) -> PathBuf {
    daemon.paths.home.join(DIR).join(format!("{id}.md"))
}

/// Forgets the summaries that waited more than a day.
fn sweep(daemon: &Daemon) -> ApiResult<()> {
    let before = daemon.clock.now_ms() - KEEP_MS;
    let store = daemon.store();
    for id in store.stale_imports(before)? {
        let _ = fs::remove_file(file(daemon, &id));
        store.delete_unsent_import(&id)?;
        debug!(import = %id, "an unsent summary was forgotten after a day");
    }
    Ok(())
}

pub fn scan(daemon: &Daemon, params: ImportsScanParams) -> ApiResult<ImportScan> {
    sweep(daemon)?;
    let scan = match params.source {
        ImportSource::ClaudeCode => ImportScan {
            source: params.source,
            projects: sources::code_projects(daemon),
            conversations: None,
        },
        ImportSource::ClaudeAi => ImportScan {
            source: params.source,
            projects: Vec::new(),
            conversations: Some(sources::ai_conversations(params.path.as_deref())?),
        },
    };
    debug!(
        source = params.source.as_str(),
        projects = scan.projects.len(),
        conversations = scan.conversations.as_ref().map_or(0, |c| c.count),
        "a history was listed"
    );
    Ok(scan)
}

/// Releases the flag of a summary being made, however making it ends.
struct Making;

impl Drop for Making {
    fn drop(&mut self) {
        MAKING.store(false, Ordering::Release);
    }
}

/// The values of every agent's keys (spec 32), to swap out exactly.
fn agent_key_values(daemon: &Daemon) -> ApiResult<Vec<String>> {
    let bots = daemon.store().bots(None, true)?;
    let secrets = daemon.paths.secrets();
    Ok(bots
        .iter()
        .flat_map(|bot| {
            daemon
                .keys
                .pairs(&secrets, &bot.id)
                .iter()
                .map(|(_, value)| value.clone())
                .collect::<Vec<_>>()
        })
        .collect())
}

pub fn create(daemon: &Daemon, params: ImportsCreateParams) -> ApiResult<ImportDraft> {
    if MAKING.swap(true, Ordering::AcqRel) {
        return Err(ApiError::Conflict(
            "a summary is being made already".to_owned(),
        ));
    }
    let _making = Making;
    sweep(daemon)?;
    let locale = daemon.desktop.locale();
    let redactor = Redactor::new(Labels::for_locale(&locale), agent_key_values(daemon)?);
    let digest = match params.source {
        ImportSource::ClaudeCode => {
            sources::code_digest(daemon, &params.keys, &redactor, words(&locale))?
        }
        ImportSource::ClaudeAi => sources::ai_digest(&params, &redactor, words(&locale))?,
    };
    let chars = digest.text.chars().count() as u32;
    let id = ImportId::generate();
    let path = file(daemon, &id);
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(ApiError::Workspace)?;
    }
    fs::write(&path, &digest.text).map_err(ApiError::Workspace)?;
    let import = Import {
        id: id.clone(),
        source: params.source,
        created_at: daemon.clock.now_ms(),
        sent_at: None,
        sent_message_id: None,
        scope: digest.scope.clone(),
        chars,
        redactions: digest.redactions.clone(),
    };
    daemon.store().insert_import(&import)?;
    debug!(
        import = %id,
        projects = digest.scope.len(),
        cut = digest.cut.len(),
        chars,
        swapped = digest.redactions.values().sum::<u32>(),
        "a summary of the history was made"
    );
    Ok(ImportDraft {
        import_id: id,
        text: digest.text,
        chars,
        tokens_estimate: (f64::from(chars) / 3.6).round() as u32,
        cut: digest.cut,
        redactions: digest.redactions,
        from: digest.from,
        to: digest.to,
    })
}

fn unsent(daemon: &Daemon, id: &ImportId) -> ApiResult<Import> {
    let import = daemon
        .store()
        .import(id)?
        .ok_or_else(|| ApiError::NotFound(format!("summary {id} is gone")))?;
    if import.sent_at.is_some() {
        return Err(ApiError::Conflict(format!("summary {id} was sent already")));
    }
    Ok(import)
}

pub fn send(daemon: &Daemon, params: ImportIdParams) -> ApiResult<ImportSent> {
    let import = unsent(daemon, &params.import_id)?;
    let path = file(daemon, &import.id);
    let bytes = fs::read(&path)
        .map_err(|_| ApiError::NotFound(format!("summary {} is gone", import.id)))?;
    let maker = maker::ensure(daemon)?;
    let words = words(&daemon.desktop.locale());
    let (body, name) = match import.source {
        ImportSource::ClaudeCode => (words.message, "claude-code-history.md"),
        ImportSource::ClaudeAi => (words.message_ai, "claude-ai-history.md"),
    };
    let message = messages::send(
        daemon,
        MessagesSendParams {
            bot_id: maker,
            body: body.to_owned(),
            attachments: Some(vec![AttachmentUpload {
                name: name.to_owned(),
                media_type: "text/markdown".to_owned(),
                data: BASE64.encode(&bytes),
            }]),
            reply_to: None,
        },
    )?;
    daemon
        .store()
        .mark_import_sent(&import.id, daemon.clock.now_ms(), &message.id)?;
    let _ = fs::remove_file(&path);
    debug!(import = %import.id, message = %message.id, "a summary was sent to the Maker");
    Ok(ImportSent {
        message_id: message.id,
    })
}

pub fn forget(daemon: &Daemon, params: ImportIdParams) -> ApiResult<()> {
    let import = unsent(daemon, &params.import_id)?;
    let _ = fs::remove_file(file(daemon, &import.id));
    daemon.store().delete_unsent_import(&import.id)?;
    debug!(import = %import.id, "a summary was forgotten");
    Ok(())
}

pub fn list(daemon: &Daemon) -> ApiResult<Vec<Import>> {
    sweep(daemon)?;
    Ok(daemon.store().imports(LIST_LIMIT)?)
}
