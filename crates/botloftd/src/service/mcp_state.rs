//! How each bot's connected tools stand (spec 25.5). Claude Code answers
//! the `mcp_status` request the daemon sends when the process starts; the
//! answer is kept in memory only and goes to the app as `bot.mcp`.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use botloft_core::ids::BotId;
use botloft_core::protocol::{BotMcp, McpServerState, McpState};
use serde_json::Value;
use tracing::warn;

use crate::state::{Daemon, Event};

/// Longest reason kept, in characters.
const ERROR_MAX: usize = 300;

#[derive(Default)]
pub struct McpStates(Mutex<HashMap<BotId, Vec<McpServerState>>>);

impl McpStates {
    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<BotId, Vec<McpServerState>>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub fn get(&self, bot: &BotId) -> Vec<McpServerState> {
        self.lock().get(bot).cloned().unwrap_or_default()
    }

    /// Keeps `states`; `true` if they differ from what was there.
    fn set(&self, bot: &BotId, states: Vec<McpServerState>) -> bool {
        let mut all = self.lock();
        if all.get(bot).map_or(states.is_empty(), |old| *old == states) {
            return false;
        }
        if states.is_empty() {
            all.remove(bot);
        } else {
            all.insert(bot.clone(), states);
        }
        true
    }
}

/// What the servers of `links` report now.
pub(crate) fn fill(daemon: &Daemon, link: &mut BotMcp) {
    link.states = daemon.mcp_states.get(&link.bot_id);
}

fn state_of(status: &str) -> McpState {
    match status {
        "connected" => McpState::Connected,
        "pending" => McpState::Pending,
        "needs-auth" => McpState::NeedsAuth,
        _ => McpState::Failed,
    }
}

/// One line, short: the reason is shown under "Details" and nowhere else.
fn clean(error: &str) -> Option<String> {
    let line = error.split_whitespace().collect::<Vec<_>>().join(" ");
    (!line.is_empty()).then(|| line.chars().take(ERROR_MAX).collect())
}

fn announce(daemon: &Daemon, bot: &BotId) {
    match daemon.store().bot_mcp(bot) {
        Ok(mut link) => {
            fill(daemon, &mut link);
            daemon.emit(Event::BotMcp(link));
        }
        Err(err) => warn!(bot = %bot, "could not read the tools of a bot: {err}"),
    }
}

/// The process answered `mcp_status`: `servers` is its `mcpServers` list.
/// Only the servers the owner attached to the bot count; the answer also
/// carries their configuration, which is never read here (it may hold the
/// secrets Claude Code expanded).
pub(crate) fn reported(daemon: &Daemon, bot: &BotId, servers: &Value) {
    let Some(list) = servers.as_array() else {
        return;
    };
    let attached = match daemon.store().bot_mcp_servers(bot) {
        Ok(attached) => attached,
        Err(err) => {
            warn!(bot = %bot, "could not read the tools of a bot: {err}");
            return;
        }
    };
    let states: Vec<McpServerState> = attached
        .iter()
        .filter_map(|server| {
            let entry = list
                .iter()
                .find(|entry| entry["name"].as_str() == Some(server.slug.as_str()))?;
            Some(McpServerState {
                server_id: server.id.clone(),
                state: state_of(entry["status"].as_str().unwrap_or_default()),
                error: entry["error"].as_str().and_then(clean),
            })
        })
        .collect();
    if daemon.mcp_states.set(bot, states) {
        announce(daemon, bot);
    }
}

/// A new process starts: what the old one said no longer holds.
pub(crate) fn forget(daemon: &Daemon, bot: &BotId) {
    if daemon.mcp_states.set(bot, Vec::new()) {
        announce(daemon, bot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_map_and_unknown_ones_count_as_failed() {
        assert_eq!(state_of("connected"), McpState::Connected);
        assert_eq!(state_of("needs-auth"), McpState::NeedsAuth);
        assert_eq!(state_of("pending"), McpState::Pending);
        assert_eq!(state_of("disabled"), McpState::Failed);
    }

    #[test]
    fn a_reason_is_one_short_line() {
        assert_eq!(clean("  \n "), None);
        assert_eq!(clean("a\n  b\tc").as_deref(), Some("a b c"));
        assert_eq!(
            clean(&"x".repeat(900)).map(|text| text.len()),
            Some(ERROR_MAX)
        );
    }
}
