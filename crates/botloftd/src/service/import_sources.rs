//! Where each history of `imports.*` comes from (spec 36.5, 36.6): Claude
//! Code's folder, read from the owner's own environment, and the ZIP of a
//! claude.ai export, read where it is. Each gives a list for the owner to
//! choose from and a summary of what they chose.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use botloft_core::protocol::{ImportConversations, ImportProject, ImportsCreateParams};
use jiff::tz::TimeZone;

use super::{ApiError, ApiResult};
use crate::imports::claude_ai::{self, Export};
use crate::imports::claude_code::{self, Project};
use crate::imports::digest::{self, Digest};
use crate::imports::digest_ai;
use crate::imports::own;
use crate::imports::redact::Redactor;
use crate::imports::words::Words;
use crate::platform;
use crate::state::Daemon;

/// The owner's own variables, as an agent gets them (spec 7.4).
fn environment() -> Vec<(String, String)> {
    platform::user_environment()
        .map(|vars| {
            vars.into_iter()
                .map(|(key, value)| {
                    (
                        key.to_string_lossy().into_owned(),
                        value.to_string_lossy().into_owned(),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Claude Code's folder, and the folders whose projects never count:
/// Botloft's own (its agents work there) and the temporary ones.
fn where_to_read(daemon: &Daemon) -> (Option<PathBuf>, Vec<String>) {
    let vars = environment();
    let root = claude_code::root(&vars, dirs::home_dir());
    let mut skip = vec![
        daemon.paths.home.to_string_lossy().into_owned(),
        daemon.paths.workspaces_root.to_string_lossy().into_owned(),
    ];
    skip.extend(
        vars.iter()
            .filter(|(key, _)| key.eq_ignore_ascii_case("TEMP") || key.eq_ignore_ascii_case("TMP"))
            .map(|(_, value)| value.clone()),
    );
    (root, skip)
}

/// Claude Code's projects, newest first.
pub fn code_projects(daemon: &Daemon) -> Vec<ImportProject> {
    let (root, skip) = where_to_read(daemon);
    let mut projects: Vec<_> = root
        .map(|root| claude_code::read(&root, None, false, &skip))
        .unwrap_or_default()
        .into_iter()
        .map(|project| ImportProject {
            sessions: project.sessions,
            prompts: project.prompts.len() as u32,
            first_at: project.prompts.last().map(|prompt| prompt.at),
            last_at: project.prompts.first().map(|prompt| prompt.at),
            key: project.key,
            path: project.path,
        })
        .collect();
    projects.sort_by_key(|project| std::cmp::Reverse(project.last_at));
    projects
}

/// The summary of Claude Code's projects in `keys`.
pub fn code_digest(
    daemon: &Daemon,
    keys: &[String],
    redactor: &Redactor,
    words: &Words,
) -> ApiResult<Digest> {
    if keys.is_empty() {
        return Err(ApiError::validation("choose at least one project"));
    }
    let (root, skip) = where_to_read(daemon);
    let root =
        root.ok_or_else(|| ApiError::validation("Claude Code has no history on this computer"))?;
    let wanted: HashSet<String> = keys.iter().cloned().collect();
    let projects: Vec<Project> = claude_code::read(&root, Some(&wanted), true, &skip);
    if let Some(missing) = keys
        .iter()
        .find(|key| !projects.iter().any(|project| &project.key == *key))
    {
        return Err(ApiError::validation(format!(
            "project {missing} is not in the history"
        )));
    }
    let extras = own::gather(&root, &projects);
    Ok(digest::build(
        projects,
        &extras,
        redactor,
        words,
        &TimeZone::system(),
    ))
}

fn export(path: Option<&str>) -> ApiResult<Export> {
    let path =
        path.ok_or_else(|| ApiError::validation("choose the ZIP of your claude.ai export"))?;
    claude_ai::read(Path::new(path)).map_err(|err| ApiError::validation(err.to_string()))
}

/// What the claude.ai export at `path` holds.
pub fn ai_conversations(path: Option<&str>) -> ApiResult<ImportConversations> {
    let export = export(path)?;
    Ok(ImportConversations {
        count: export.conversations.len() as u32,
        first_at: export.conversations.last().map(|c| c.started_at),
        last_at: export.conversations.first().map(|c| c.started_at),
        projects: export.projects,
    })
}

/// The summary of the claude.ai export in `params`.
pub fn ai_digest(
    params: &ImportsCreateParams,
    redactor: &Redactor,
    words: &Words,
) -> ApiResult<Digest> {
    let export = export(params.path.as_deref())?;
    Ok(digest_ai::build(
        &export,
        params.from,
        params.to,
        redactor,
        words,
        &TimeZone::system(),
    ))
}
