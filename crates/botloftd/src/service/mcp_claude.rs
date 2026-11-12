//! Connected tools brought from the owner's own Claude Code (spec 25.8).
//! Its `~/.claude.json` lists the servers set for every folder and those
//! set for one folder; the owner picks which to copy, and each goes through
//! `mcp.save` like a pasted one. The values of headers and variables are
//! read here twice and never sent to the app.

use std::collections::BTreeSet;
use std::path::PathBuf;

use botloft_core::protocol::{
    ClaudeMcpProblem, ClaudeMcpServer, ClaudeMcpServers, McpImportParams, McpKind, McpOverview,
    McpSaveParams,
};
use serde_json::{Map, Value};
use tracing::{info, warn};

use super::{ApiError, ApiResult, import_sources, mcp};
use crate::state::Daemon;

/// The settings a server of Claude Code may have and Botloft copies.
const FIELDS: [&str; 6] = ["type", "command", "args", "env", "url", "headers"];

/// A server as Claude Code's file has it: what the app sees, and what
/// `mcp.save` takes when it can come.
pub(crate) struct Found {
    pub listed: ClaudeMcpServer,
    pub save: Option<McpSaveParams>,
}

/// Claude Code's settings file: `.claude.json` in `CLAUDE_CONFIG_DIR` when
/// the owner set it, else in their profile.
pub(crate) fn config_file(environment: &[(String, String)], home: Option<PathBuf>) -> PathBuf {
    let set = |name: &str| {
        environment
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    };
    set("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| set("USERPROFILE").map(PathBuf::from))
        .or(home)
        .unwrap_or_default()
        .join(".claude.json")
}

/// A folder as one comparable text: `/` between parts, lowercase.
fn plain(folder: &str) -> String {
    folder
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase()
}

/// The servers in `text`, those for every folder first. Folders inside
/// `skip` (Botloft's own, where its agents work) are left out.
pub(crate) fn read(text: &str, skip: &[String]) -> Vec<Found> {
    let Ok(Value::Object(root)) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    let mut found = servers_in(&root, None);
    let skip: Vec<String> = skip.iter().map(|folder| plain(folder)).collect();
    let mut folders: Vec<(&String, &Value)> = root
        .get("projects")
        .and_then(Value::as_object)
        .map(|projects| projects.iter().collect())
        .unwrap_or_default();
    folders.sort_by_key(|(folder, _)| plain(folder));
    for (folder, project) in folders {
        let inside = plain(folder);
        if skip
            .iter()
            .any(|own| inside == *own || inside.starts_with(&format!("{own}/")))
        {
            continue;
        }
        if let Value::Object(project) = project {
            found.extend(servers_in(project, Some(folder)));
        }
    }
    found
}

fn servers_in(holder: &Map<String, Value>, folder: Option<&String>) -> Vec<Found> {
    let Some(Value::Object(servers)) = holder.get("mcpServers") else {
        return Vec::new();
    };
    let mut found: Vec<Found> = servers
        .iter()
        .map(|(name, config)| entry(name, folder.cloned(), config))
        .collect();
    found.sort_by(|a, b| a.listed.name.cmp(&b.listed.name));
    found
}

fn strings(value: Option<&Value>) -> Option<Vec<String>> {
    match value {
        None => Some(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| item.as_str().map(str::to_owned))
            .collect(),
        Some(_) => None,
    }
}

fn texts(value: Option<&Value>) -> Option<std::collections::BTreeMap<String, String>> {
    match value {
        None => Some(Default::default()),
        Some(Value::Object(map)) => map
            .iter()
            .map(|(key, value)| value.as_str().map(|value| (key.clone(), value.to_owned())))
            .collect(),
        Some(_) => None,
    }
}

fn entry(name: &str, folder: Option<String>, config: &Value) -> Found {
    let mut listed = ClaudeMcpServer {
        name: name.to_owned(),
        folder,
        kind: None,
        url: None,
        command: None,
        args: Vec::new(),
        header_names: Vec::new(),
        env_names: Vec::new(),
        saved: false,
        problem: None,
    };
    let refuse = |mut listed: ClaudeMcpServer, problem| {
        listed.problem = Some(problem);
        Found { listed, save: None }
    };
    let field = |field: &str| ClaudeMcpProblem::Field {
        field: field.to_owned(),
    };
    let Value::Object(config) = config else {
        return refuse(listed, field(name));
    };
    let kind = match config.get("type").and_then(Value::as_str) {
        Some(kind) => kind.to_owned(),
        None if config.contains_key("command") => "stdio".to_owned(),
        None => "http".to_owned(),
    };
    let Some(kind) = McpKind::parse(&kind) else {
        return refuse(listed, ClaudeMcpProblem::Kind { kind });
    };
    listed.kind = Some(kind);
    listed.url = config.get("url").and_then(Value::as_str).map(str::to_owned);
    listed.command = config
        .get("command")
        .and_then(Value::as_str)
        .map(str::to_owned);
    if let Some(unknown) = config.keys().find(|key| !FIELDS.contains(&key.as_str())) {
        return refuse(listed, field(unknown));
    }
    let Some(args) = strings(config.get("args")) else {
        return refuse(listed, field("args"));
    };
    let Some(env) = texts(config.get("env")) else {
        return refuse(listed, field("env"));
    };
    let Some(headers) = texts(config.get("headers")) else {
        return refuse(listed, field("headers"));
    };
    listed.args = args.clone();
    listed.env_names = env.keys().cloned().collect();
    listed.header_names = headers.keys().cloned().collect();
    let save = match kind {
        McpKind::Stdio if listed.command.is_some() => McpSaveParams {
            server_id: None,
            name: name.to_owned(),
            kind,
            url: None,
            command: listed.command.clone(),
            args,
            headers: Default::default(),
            env,
            description: String::new(),
        },
        McpKind::Http if listed.url.is_some() => McpSaveParams {
            server_id: None,
            name: name.to_owned(),
            kind,
            url: listed.url.clone(),
            command: None,
            args: Vec::new(),
            headers,
            env: Default::default(),
            description: String::new(),
        },
        McpKind::Stdio => return refuse(listed, field("command")),
        McpKind::Http => return refuse(listed, field("url")),
    };
    Found {
        listed,
        save: Some(save),
    }
}

/// What Claude Code has, read now, with Botloft's own folders left out.
fn found(daemon: &Daemon) -> Option<Vec<Found>> {
    let file = config_file(&import_sources::environment(), dirs::home_dir());
    let text = match std::fs::read_to_string(&file) {
        Ok(text) => text,
        Err(err) => {
            if err.kind() != std::io::ErrorKind::NotFound {
                warn!("could not read Claude Code's settings: {err}");
            }
            return None;
        }
    };
    let skip = [
        daemon.paths.home.to_string_lossy().into_owned(),
        daemon.paths.workspaces_root.to_string_lossy().into_owned(),
    ];
    Some(read(&text, &skip))
}

fn connected(daemon: &Daemon) -> ApiResult<BTreeSet<String>> {
    Ok(daemon
        .store()
        .mcp_servers()?
        .into_iter()
        .map(|server| server.slug)
        .collect())
}

/// `mcp.fromClaude`: the servers of the owner's Claude Code, and which of
/// them are connected here already.
pub fn servers(daemon: &Daemon) -> ApiResult<ClaudeMcpServers> {
    let Some(found) = found(daemon) else {
        return Ok(ClaudeMcpServers::default());
    };
    let slugs = connected(daemon)?;
    Ok(ClaudeMcpServers {
        found: true,
        servers: found
            .into_iter()
            .map(|mut each| {
                each.listed.saved = slugs.contains(&mcp::slug_of(&each.listed.name));
                each.listed
            })
            .collect(),
    })
}

/// `mcp.importClaude`: copies the picked servers. Nothing is saved when one
/// cannot come, or when its name is taken here or twice in the picks.
pub fn import(daemon: &Daemon, params: McpImportParams) -> ApiResult<McpOverview> {
    let found = found(daemon)
        .ok_or_else(|| ApiError::validation("Claude Code's settings were not found"))?;
    let mut slugs = connected(daemon)?;
    let mut saves = Vec::new();
    for pick in &params.picks {
        let each = found
            .iter()
            .find(|each| each.listed.name == pick.name && each.listed.folder == pick.folder)
            .ok_or_else(|| ApiError::NotFound(format!("Claude Code tool {}", pick.name)))?;
        let save = each.save.clone().ok_or_else(|| {
            ApiError::validation(format!("{} cannot be brought as it is", pick.name))
        })?;
        if !slugs.insert(mcp::slug_of(&save.name)) {
            return Err(ApiError::validation(format!(
                "a tool named {} is connected already",
                save.name
            )));
        }
        saves.push(save);
    }
    let mut overview = None;
    for save in saves {
        overview = Some(mcp::save(daemon, save)?);
    }
    info!(
        count = params.picks.len(),
        "the owner brought tools from Claude Code"
    );
    match overview {
        Some(overview) => Ok(overview),
        None => mcp::overview(daemon),
    }
}

#[cfg(test)]
#[path = "mcp_claude_tests.rs"]
mod tests;
