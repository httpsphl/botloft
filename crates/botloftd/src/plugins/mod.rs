//! The owner's Claude Code plugins in the agents (spec 41): the plugins on
//! in the owner's Claude Code, which agents may use each one (chosen one by
//! one, or every agent), and the copy each agent loads with `--plugin-dir`
//! from its `.claude\plugins`. Only agents on Claude Code get them.

mod copy;
mod found;
mod give;
mod meta;

pub use give::{everyone, share};

use std::collections::BTreeMap;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use botloft_core::ids::BotId;
use botloft_core::protocol::{AgentKind, Plugin, PluginList, PluginSource};
use botloft_store::BotRecord;
use serde_json::Value;
use tracing::warn;

use crate::seat::Seat;
use crate::service::ApiResult;
use crate::state::Daemon;
use found::Found;
use meta::Meta;

/// Where the owner's Claude Code keeps its plugins and settings, and the
/// lock on Botloft's copies, which agents starting together share.
#[derive(Default)]
pub struct Plugins {
    root: Mutex<Option<PathBuf>>,
    copies: Mutex<()>,
}

impl Plugins {
    /// Looks for the owner's plugins in `claude`, a folder laid out like
    /// `~/.claude`, instead of the owner's own.
    pub fn use_root(&self, claude: PathBuf) {
        *self.root.lock().unwrap_or_else(|p| p.into_inner()) = Some(claude);
    }

    fn root(&self) -> Option<PathBuf> {
        let set = self.root.lock().unwrap_or_else(|p| p.into_inner()).clone();
        set.or_else(|| std::env::home_dir().map(|home| home.join(".claude")))
    }

    fn copies(&self) -> MutexGuard<'_, ()> {
        self.copies.lock().unwrap_or_else(|p| p.into_inner())
    }
}

/// A name Botloft accepts as a plugin's: what Claude Code would load, and
/// safe to join to a path.
fn valid(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && !name.starts_with('.')
        && name
            .chars()
            .all(|ch| ch.is_alphanumeric() || matches!(ch, '-' | '_' | '.'))
}

fn found(daemon: &Daemon) -> BTreeMap<String, Found> {
    daemon
        .plugins
        .root()
        .map(|claude| found::found(&claude))
        .unwrap_or_default()
}

/// Botloft's own copies of the plugins it gave (spec 41.3).
fn library(daemon: &Daemon) -> PathBuf {
    daemon.paths.home.join("plugins")
}

/// The plugins there are, by name: the owner's, and the ones only Botloft's
/// copy is left of that an agent still has.
fn names(daemon: &Daemon, found: &BTreeMap<String, Found>) -> botloft_store::Result<Vec<String>> {
    let (given, everyone) = {
        let store = daemon.store();
        (store.given_plugins()?, store.everyone_plugins()?)
    };
    let mut names: Vec<String> = found.keys().cloned().collect();
    for name in given.into_iter().chain(everyone) {
        if !names.contains(&name) && valid(&name) && library(daemon).join(&name).is_dir() {
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

/// The plugins an agent on Claude Code uses, out of `names` (spec 41.3):
/// the ones chosen for it, the ones for every agent, and all of them when
/// the owner gave every plugin to every agent.
fn chosen(daemon: &Daemon, bot: &BotId, names: &[String]) -> botloft_store::Result<Vec<String>> {
    let store = daemon.store();
    if store.all_plugins_everyone()? {
        return Ok(names.to_vec());
    }
    let mut chosen = store.bot_plugins(bot)?;
    chosen.extend(store.everyone_plugins()?);
    chosen.retain(|name| names.contains(name));
    chosen.sort();
    chosen.dedup();
    Ok(chosen)
}

fn on_claude_code(bot: &BotRecord) -> bool {
    bot.agent == AgentKind::Claude
}

fn plugin(name: String, meta: Meta, source: PluginSource) -> Plugin {
    Plugin {
        name,
        description: meta.description,
        version: meta.version,
        source,
        skills: meta.skills,
        agents: meta.agents,
        hooks: meta.hooks,
        language_servers: meta.language_servers,
        mcp_servers: meta.mcp_servers,
        programs: meta.programs,
        needs_settings: meta.needs_settings,
        bot_ids: Vec::new(),
        everyone: false,
    }
}

/// `plugins.list`: the plugins on in the owner's Claude Code, and the ones
/// only Botloft's copy is left of that an agent still has.
pub fn list(daemon: &Daemon) -> ApiResult<PluginList> {
    let found = found(daemon);
    let names = names(daemon, &found)?;
    let store = daemon.store();
    let all = store.all_plugins_everyone()?;
    let everyone = store.everyone_plugins()?;
    let claude: Vec<BotId> = store
        .bots(None, false)?
        .into_iter()
        .filter(on_claude_code)
        .map(|bot| bot.id)
        .collect();
    let mut plugins = Vec::new();
    for name in names {
        let (dir, source) = match found.get(&name) {
            Some(found) => (found.dir.clone(), found.source),
            None => (library(daemon).join(&name), PluginSource::Kept),
        };
        let Some(meta) = meta::read(&dir, &name) else {
            continue;
        };
        let for_everyone = everyone.contains(&name);
        let bot_ids = if all || for_everyone {
            claude.clone()
        } else {
            store
                .plugin_bots(&name)?
                .into_iter()
                .filter(|bot| claude.contains(bot))
                .collect()
        };
        plugins.push(Plugin {
            bot_ids,
            everyone: for_everyone,
            ..plugin(name, meta, source)
        });
    }
    Ok(PluginList {
        plugins,
        everyone: all,
    })
}

/// Writes the agent's plugins before it starts (spec 41.3): Botloft's copy
/// of each one it has, fresh from the owner's Claude Code when the plugin
/// is still there, in its `.claude\plugins`. Answers the folders it loads
/// with `--plugin-dir`.
pub(crate) fn launch(daemon: &Daemon, bot: &BotId, workspace: &Path) -> io::Result<Vec<PathBuf>> {
    let _copies = daemon.plugins.copies();
    if !crate::connectors::on_claude(daemon, bot) {
        return copy::sync(&library(daemon), workspace, &[]);
    }
    let found = found(daemon);
    let chosen = names(daemon, &found)
        .and_then(|names| chosen(daemon, bot, &names))
        .map_err(io::Error::other)?;
    for name in &chosen {
        if let Some(found) = found.get(name) {
            std::fs::create_dir_all(library(daemon))?;
            if let Err(err) = copy::copy_plugin(&found.dir, &library(daemon).join(name)) {
                warn!(bot = %bot, "could not copy a plugin again: {err}");
            }
        }
    }
    copy::sync(&library(daemon), workspace, &chosen)
}

/// The plugins Claude Code could not load, from its `system/init` (spec
/// 41.4): their names and what kind of error, never more.
pub(crate) fn not_loaded(bot: &Seat, init: &Value) {
    for error in init["plugin_errors"].as_array().into_iter().flatten() {
        let plugin = error["plugin"].as_str().unwrap_or_default();
        let kind = error["type"].as_str().unwrap_or_default();
        warn!(bot = %bot, plugin, kind, "a plugin did not load");
    }
}
