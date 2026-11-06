//! The owner's claude.ai connectors in the agents (spec 34): which the
//! account has, which each agent may use, and what that changes in how the
//! agent runs.

mod discover;
pub mod names;
mod rules;
mod seen;

pub(crate) use seen::seen;

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::ids::{BotId, MessageId};
use botloft_core::protocol::{
    AgentKind, Bot, BotsSetConnectorsParams, ConnectorList, ConnectorsListParams, Message,
    MessageKind, SenderKind,
};
use tracing::{info, warn};

use crate::platform;
use crate::service::bots::{active, changed};
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

#[derive(Default)]
pub struct Connectors {
    /// What the last look found; `None` before the first in this daemon.
    known: Mutex<Option<ConnectorList>>,
    /// Held while a look runs, so two never run at once.
    looking: Mutex<()>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Where the connectors are looked for, and the last list kept.
fn dir(daemon: &Daemon) -> PathBuf {
    daemon.paths.home.join("connectors")
}

fn kept(daemon: &Daemon) -> PathBuf {
    dir(daemon).join("known.json")
}

/// The connectors as last found: in memory, or as kept on disk by an earlier
/// daemon, so an agent that starts before a new look still has the others
/// blocked (spec 34.5).
fn known(daemon: &Daemon) -> Option<ConnectorList> {
    let mut known = lock(&daemon.connectors.known);
    if known.is_none() {
        *known = std::fs::read(kept(daemon))
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok());
    }
    known.clone()
}

/// `connectors.list {refresh}`: the connectors as last found, looked for
/// again when asked or when never looked for. A look takes a few seconds.
pub fn list(daemon: &Daemon, params: ConnectorsListParams) -> ApiResult<ConnectorList> {
    if !params.refresh
        && let Some(list) = known(daemon)
    {
        return Ok(list);
    }
    let _looking = lock(&daemon.connectors.looking);
    // Another call may have looked while this one waited.
    if !params.refresh
        && let Some(list) = known(daemon)
    {
        return Ok(list);
    }
    let program = daemon
        .supervisor
        .claude_path()
        .ok_or_else(|| ApiError::Conflict("Claude Code was not found".into()))?;
    let mut env = platform::user_environment().map_err(ApiError::Workspace)?;
    env.retain(|(name, _)| {
        !name
            .to_string_lossy()
            .to_ascii_uppercase()
            .starts_with("BOTLOFT_")
    });
    let connectors =
        discover::discover(&program, &env, &dir(daemon)).map_err(ApiError::Workspace)?;
    info!(found = connectors.len(), "claude.ai connectors looked for");
    let list = ConnectorList {
        connectors,
        checked_at: daemon.clock.now_ms(),
    };
    keep(daemon, &list);
    *lock(&daemon.connectors.known) = Some(list.clone());
    Ok(list)
}

/// Writes `list` to disk, for the next daemon.
fn keep(daemon: &Daemon, list: &ConnectorList) {
    if let Ok(text) = serde_json::to_vec(list)
        && let Err(err) = std::fs::write(kept(daemon), text)
    {
        warn!("could not keep the claude.ai connectors: {err}");
    }
}

/// Looks for the connectors once after the daemon starts, when Claude Code
/// is found, so the agents' rules name them before the owner opens the page.
pub async fn look_at_start(daemon: Arc<Daemon>) {
    for _ in 0..30 {
        if daemon.supervisor.claude_path().is_some() {
            break;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    let looked =
        tokio::task::spawn_blocking(move || list(&daemon, ConnectorsListParams { refresh: true }))
            .await;
    if let Ok(Err(err)) = looked {
        warn!("could not look for the claude.ai connectors: {err}");
    }
}

/// `bots.setConnectors`: which connectors the agent may use. It starts again
/// with them when nothing is in progress (spec 34.4).
pub fn set_bot(daemon: &Daemon, params: BotsSetConnectorsParams) -> ApiResult<Bot> {
    let mut chosen: Vec<String> = Vec::new();
    for name in params.names {
        let name = name.trim().to_owned();
        if name.is_empty() || name.chars().count() > 100 {
            return Err(ApiError::Conflict(format!(
                "not a connector name: {name:?}"
            )));
        }
        if !chosen.contains(&name) {
            chosen.push(name);
        }
    }
    let bot = {
        let store = daemon.store();
        let (crew, record) = active(&store, &params.bot_id)?;
        if record.agent != AgentKind::Claude {
            return Err(ApiError::Conflict(
                "only agents on Claude Code use claude.ai connectors".into(),
            ));
        }
        store.set_bot_connectors(&record.id, &chosen)?;
        let workspace = daemon.paths.bot_workspace(&crew.slug, &record.slug);
        if let Err(err) = rules::write_rules(&workspace, &chosen, &others(daemon, &chosen)) {
            warn!(bot = %record.id, "could not write the connectors' rules: {err}");
        }
        changed(daemon, &store, &crew, record)
    };
    daemon.supervisor.launch_settings_changed(&bot.id);
    Ok(bot)
}

/// What an agent starts with (spec 34.5): the connectors it may use, and
/// the servers whose tools it must not get.
pub struct Launch {
    pub chosen: Vec<String>,
    /// Tool-name servers for `--disallowedTools mcp__<server>`.
    pub blocked: Vec<String>,
}

/// The launch of `bot` from `workspace`, with its rules written. An agent
/// with no connector runs as before, fenced by `--strict-mcp-config`.
pub(crate) fn launch(daemon: &Daemon, bot: &BotId, workspace: &Path) -> io::Result<Launch> {
    let chosen = daemon
        .store()
        .bot_connectors(bot)
        .map_err(io::Error::other)?;
    rules::write_rules(workspace, &chosen, &others(daemon, &chosen))?;
    if chosen.is_empty() {
        return Ok(Launch {
            chosen,
            blocked: Vec::new(),
        });
    }
    let mut blocked: Vec<String> = known(daemon)
        .map(|list| list.connectors)
        .unwrap_or_default()
        .into_iter()
        .filter(|connector| !chosen.contains(&connector.name))
        .map(|connector| names::connector_server(&connector.name))
        .collect();
    let claude_json = std::env::home_dir().map(|home| home.join(".claude.json"));
    blocked.extend(
        names::planted(workspace, claude_json.as_deref())
            .iter()
            .map(|name| names::tool_server(name)),
    );
    blocked.sort();
    blocked.dedup();
    Ok(Launch { chosen, blocked })
}

/// Whether the agent may use a tool of `server` (a tool-name server). An
/// agent with no connector runs with `--strict-mcp-config`, where only the
/// Botloft's servers exist; one given connectors may use the Botloft's own,
/// its connected tools and those connectors, and nothing else (spec 34.5).
pub fn server_allowed(daemon: &Daemon, bot: &BotId, server: &str) -> bool {
    let store = daemon.store();
    let chosen = store.bot_connectors(bot).unwrap_or_default();
    // The Botloft's own MCP server (spec 10).
    if chosen.is_empty() || server == "botloft" {
        return true;
    }
    chosen
        .iter()
        .any(|name| names::connector_server(name) == server)
        || store
            .bot_mcp_servers(bot)
            .map(|servers| servers.iter().any(|tool| tool.slug == server))
            .unwrap_or(false)
}

/// The names of the account's tools the agent was not given.
fn others(daemon: &Daemon, chosen: &[String]) -> Vec<String> {
    known(daemon)
        .map(|list| list.connectors)
        .unwrap_or_default()
        .into_iter()
        .map(|connector| connector.name)
        .filter(|name| !chosen.contains(name))
        .collect()
}

/// The account's tool named `name`, written as the account writes it, or
/// `None` when the account has no such tool.
pub(crate) fn account_tool(daemon: &Daemon, name: &str) -> Option<String> {
    let wanted = name.trim().to_lowercase();
    known(daemon)?
        .connectors
        .into_iter()
        .map(|connector| connector.name)
        .find(|have| have.to_lowercase() == wanted)
}

/// Whether the agent runs on Claude Code, the only engine that gets them.
pub fn on_claude(daemon: &Daemon, bot: &BotId) -> bool {
    let store = daemon.store();
    active(&store, bot).is_ok_and(|(_, record)| record.agent == AgentKind::Claude)
}

/// Whether the agent was given `name`.
pub(crate) fn given(daemon: &Daemon, bot: &BotId, name: &str) -> bool {
    daemon
        .store()
        .bot_connectors(bot)
        .is_ok_and(|chosen| chosen.iter().any(|have| have == name))
}

/// The owner said yes to an agent's `ask_connector`: the tool joins its
/// choice, it starts again when its turn ends, and then a note tells it the
/// tool is there, so it goes on with what it was doing.
pub(crate) fn grant(daemon: &Daemon, bot: &BotId, name: &str) -> ApiResult<()> {
    let mut names = daemon.store().bot_connectors(bot)?;
    if !names.iter().any(|have| have == name) {
        names.push(name.to_owned());
    }
    set_bot(
        daemon,
        BotsSetConnectorsParams {
            bot_id: bot.clone(),
            names,
        },
    )?;
    let store = daemon.store();
    let (crew, _) = active(&store, bot)?;
    let note = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: bot.clone(),
        kind: MessageKind::System,
        body: format!(
            "The owner gave you {name} from their claude.ai account. Its tools \
             (mcp__{}__*) are ready now: go on with what you were doing.",
            names::connector_server(name)
        ),
        task_id: None,
        routine_id: None,
        question_id: None,
        reply_to: None,
        attachments: Vec::new(),
        created_at: daemon.clock.now_ms(),
    };
    if let Err(err) = crate::service::messages::post(daemon, &store, note, None) {
        warn!(bot = %bot, "could not tell an agent about its new tool: {err}");
    }
    Ok(())
}
