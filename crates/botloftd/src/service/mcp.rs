//! Connected tools (spec 25): the MCP servers the owner registers and the
//! bots that use them. Only the owner, through the app, gets here: no tool
//! of a bot reaches these operations.

use std::collections::BTreeMap;
use std::io;

use botloft_core::ids::{BotId, McpServerId};
use botloft_core::protocol::{
    BotMcp, BotMcpSetParams, BotRules, McpKind, McpOverview, McpSaveParams, McpServer,
    McpServerIdParams,
};
use botloft_core::validate;
use botloft_store::{BotRecord, Store};
use tracing::{info, warn};

use super::{ApiError, ApiResult, bots, mcp_state};
use crate::mcp_secrets::{self, McpSecrets};
use crate::state::{Daemon, Event};
use crate::workspace;

/// Entries of one server, so a mistake or a pasted file cannot grow the
/// generated `mcp.json` without limit.
const MAX_ENTRIES: usize = 32;
const MAX_TEXT: usize = 2000;
const MAX_VALUE: usize = 8000;

fn overview_of(daemon: &Daemon, store: &Store) -> ApiResult<McpOverview> {
    let mut bots = store.mcp_links()?;
    for link in &mut bots {
        mcp_state::fill(daemon, link);
    }
    Ok(McpOverview {
        servers: store.mcp_servers()?,
        bots,
    })
}

pub fn overview(daemon: &Daemon) -> ApiResult<McpOverview> {
    overview_of(daemon, &daemon.store())
}

/// A tool that goes away, or comes back under another name, is not the one
/// the owner allowed for good (spec 25.4): the rules for its name go.
fn forget_allowed(daemon: &Daemon, slug: &str) {
    let store = daemon.store();
    let bots = match store.delete_mcp_allow_rules(slug) {
        Ok(bots) => bots,
        Err(err) => {
            warn!("could not forget what bots were allowed to call: {err}");
            return;
        }
    };
    for bot_id in bots {
        match store.allow_rules(&bot_id) {
            Ok(rules) => daemon.emit(Event::BotRules(BotRules { bot_id, rules })),
            Err(err) => warn!(bot = %bot_id, "could not read the allow rules: {err}"),
        }
    }
}

/// Rewrites the rules of `bot` with the tools it has now.
pub(crate) fn write_rules(
    daemon: &Daemon,
    store: &Store,
    crew: &botloft_core::protocol::Crew,
    bot: &BotRecord,
) -> io::Result<()> {
    let servers = store.bot_mcp_servers(&bot.id).map_err(io::Error::other)?;
    workspace::write_rules(&daemon.paths, crew, bot, &servers)
}

/// What a name becomes in the tools Claude Code shows: lowercase letters,
/// digits and `_`, so `mcp__<slug>__<tool>` stays one token.
fn slug_of(name: &str) -> String {
    let mut slug = String::new();
    for ch in name.chars().flat_map(char::to_lowercase) {
        let ch = if ch.is_ascii_alphanumeric() { ch } else { '_' };
        if ch != '_' || !slug.is_empty() && !slug.ends_with('_') {
            slug.push(ch);
        }
    }
    slug.truncate(32);
    slug.trim_end_matches('_').to_owned()
}

fn text(field: &str, value: &str) -> ApiResult<String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > MAX_TEXT || value.contains(['\r', '\n']) {
        return Err(ApiError::validation(format!(
            "{field} must be one line of up to {MAX_TEXT} characters"
        )));
    }
    Ok(value.to_owned())
}

/// Names of headers and variables go into `mcp.json` and the environment.
fn names(field: &str, map: &BTreeMap<String, String>, valid: fn(&str) -> bool) -> ApiResult<()> {
    if map.len() > MAX_ENTRIES {
        return Err(ApiError::validation(format!(
            "{field} has too many entries"
        )));
    }
    for (name, value) in map {
        if !valid(name) {
            return Err(ApiError::validation(format!(
                "{field}: \"{name}\" is not a valid name"
            )));
        }
        if value.len() > MAX_VALUE || value.contains('\0') {
            return Err(ApiError::validation(format!(
                "{field}: the value of \"{name}\" is not valid"
            )));
        }
    }
    Ok(())
}

fn header_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 100
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn variable_name(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.len() <= 100
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && !name.to_ascii_uppercase().starts_with("BOTLOFT_")
}

/// Checks `params` and builds the server they describe, with the secrets
/// to keep for it. An empty value keeps what `old` stored under that name.
fn build(
    params: &McpSaveParams,
    id: McpServerId,
    created_at: i64,
    old: &McpSecrets,
) -> ApiResult<(McpServer, McpSecrets)> {
    let name = validate::name("name", &params.name)?;
    let slug = slug_of(&name);
    if slug.is_empty() || slug == "botloft" {
        return Err(ApiError::validation("that name cannot be used for a tool"));
    }
    if params.args.len() > MAX_ENTRIES {
        return Err(ApiError::validation("args has too many entries"));
    }
    let args = params
        .args
        .iter()
        .map(|arg| (arg.len() <= MAX_TEXT && !arg.contains('\0')).then(|| arg.clone()))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| ApiError::validation("an argument is too long"))?;
    names("headers", &params.headers, header_name)?;
    names("env", &params.env, variable_name)?;

    let (url, command) = match params.kind {
        McpKind::Http => {
            let url = text("url", params.url.as_deref().unwrap_or_default())?;
            if !(url.starts_with("http://") || url.starts_with("https://")) {
                return Err(ApiError::validation(
                    "url must start with http:// or https://",
                ));
            }
            if params.command.is_some() || !args.is_empty() || !params.env.is_empty() {
                return Err(ApiError::validation(
                    "a server at an address has no command or variables",
                ));
            }
            (Some(url), None)
        }
        McpKind::Stdio => {
            let command = text("command", params.command.as_deref().unwrap_or_default())?;
            if params.url.is_some() || !params.headers.is_empty() {
                return Err(ApiError::validation("a program has no address or headers"));
            }
            (None, Some(command))
        }
    };
    let keep = |new: &BTreeMap<String, String>, stored: &BTreeMap<String, String>| {
        new.iter()
            .map(|(name, value)| {
                let value = if value.is_empty() {
                    stored.get(name).cloned().unwrap_or_default()
                } else {
                    value.clone()
                };
                (name.clone(), value)
            })
            .collect::<BTreeMap<_, _>>()
    };
    let secrets = McpSecrets {
        headers: keep(&params.headers, &old.headers),
        env: keep(&params.env, &old.env),
    };
    let server = McpServer {
        id,
        name,
        slug,
        kind: params.kind,
        url,
        command,
        args,
        header_names: params.headers.keys().cloned().collect(),
        env_names: params.env.keys().cloned().collect(),
        description: validate::role(&params.description)?,
        created_at,
    };
    Ok((server, secrets))
}

/// Tells the apps, and the running bots that use `bots` to start again with
/// the new tools (spec 25.3).
fn applied(daemon: &Daemon, bots: &[BotId]) -> ApiResult<McpOverview> {
    let store = daemon.store();
    for id in bots {
        match bots::active(&store, id) {
            Ok((crew, bot)) => {
                if let Err(err) = write_rules(daemon, &store, &crew, &bot) {
                    warn!(bot = %id, "could not refresh the rules after a tool changed: {err}");
                }
                daemon.supervisor.launch_settings_changed(id);
            }
            Err(ApiError::NotFound(_)) => {}
            Err(err) => return Err(err),
        }
    }
    let overview = overview_of(daemon, &store)?;
    drop(store);
    daemon.emit(Event::McpServers(overview.clone()));
    Ok(overview)
}

pub fn save(daemon: &Daemon, params: McpSaveParams) -> ApiResult<McpOverview> {
    let secrets_dir = daemon.paths.secrets();
    let (users, renamed_from) = {
        let store = daemon.store();
        let (id, created_at, old, old_slug) = match &params.server_id {
            Some(id) => {
                let old = store
                    .mcp_server(id)?
                    .ok_or_else(|| ApiError::NotFound(format!("tool {id}")))?;
                let stored = mcp_secrets::load(&secrets_dir, id).map_err(ApiError::Workspace)?;
                (id.clone(), old.created_at, stored, Some(old.slug))
            }
            None => (
                McpServerId::generate(),
                daemon.clock.now_ms(),
                McpSecrets::default(),
                None,
            ),
        };
        let (server, secrets) = build(&params, id, created_at, &old)?;
        // The secrets first: a server never exists without them.
        mcp_secrets::save(&secrets_dir, &server.id, &secrets).map_err(ApiError::Workspace)?;
        if params.server_id.is_some() {
            store.update_mcp_server(&server)?;
        } else {
            store.insert_mcp_server(&server)?;
        }
        info!(
            kind = server.kind.as_str(),
            "the owner saved a connected tool"
        );
        let users = store
            .mcp_links()?
            .into_iter()
            .filter(|link| link.server_ids.contains(&server.id))
            .map(|link| link.bot_id)
            .collect::<Vec<_>>();
        (users, old_slug.filter(|slug| *slug != server.slug))
    };
    if let Some(slug) = renamed_from {
        forget_allowed(daemon, &slug);
    }
    applied(daemon, &users)
}

pub fn delete(daemon: &Daemon, params: McpServerIdParams) -> ApiResult<McpOverview> {
    let slug = daemon
        .store()
        .mcp_server(&params.server_id)?
        .ok_or_else(|| ApiError::NotFound(format!("tool {}", params.server_id)))?
        .slug;
    let users = daemon
        .store()
        .delete_mcp_server(&params.server_id)?
        .ok_or_else(|| ApiError::NotFound(format!("tool {}", params.server_id)))?;
    forget_allowed(daemon, &slug);
    if let Err(err) = mcp_secrets::remove(&daemon.paths.secrets(), &params.server_id) {
        warn!("could not remove the secrets of a deleted tool: {err}");
    }
    info!("the owner deleted a connected tool");
    applied(daemon, &users)
}

pub fn set_bot(daemon: &Daemon, params: BotMcpSetParams) -> ApiResult<BotMcp> {
    let result = {
        let store = daemon.store();
        bots::active(&store, &params.bot_id)?;
        let mut ids = Vec::new();
        for id in &params.server_ids {
            if store.mcp_server(id)?.is_none() {
                return Err(ApiError::NotFound(format!("tool {id}")));
            }
            if !ids.contains(id) {
                ids.push(id.clone());
            }
        }
        store.set_bot_mcp_servers(&params.bot_id, &ids)?;
        let mut result = store.bot_mcp(&params.bot_id)?;
        mcp_state::fill(daemon, &mut result);
        result
    };
    applied(daemon, std::slice::from_ref(&params.bot_id))?;
    daemon.emit(Event::BotMcp(result.clone()));
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_becomes_a_slug_of_one_token() {
        assert_eq!(slug_of("LinkedIn"), "linkedin");
        assert_eq!(slug_of("  My CRM / v2! "), "my_crm_v2");
        assert_eq!(slug_of("---"), "");
        assert_eq!(slug_of(&"a".repeat(80)).len(), 32);
    }

    #[test]
    fn variables_cannot_shadow_the_ones_of_botloft() {
        assert!(variable_name("LI_COOKIE"));
        assert!(!variable_name("botloft_bot_token"));
        assert!(!variable_name("1ST"));
        assert!(!variable_name("A B"));
        assert!(header_name("X-Api-Key"));
        assert!(!header_name("X Api"));
    }
}
