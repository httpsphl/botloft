//! The owner's Claude Code plugins in the agents (spec 41): the plugins on
//! in the owner's Claude Code, what each brings, and which agents may use
//! each one.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

text_enum!(
    /// Where a plugin comes from.
    PluginSource, "plugin source" {
        /// Installed from a marketplace in the owner's Claude Code.
        ClaudeCode => "claude_code",
        /// The owner's claude.ai account, downloaded by Claude Code.
        ClaudeAi => "claude_ai",
        /// Only Botloft's copy is left: the original is gone.
        Kept => "kept",
    }
);

/// A plugin the owner can give to agents (spec 41.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Plugin {
    /// Its name, which is how Botloft and Claude Code know it.
    pub name: String,
    /// What it does, from its manifest; may be empty.
    pub description: String,
    /// Its version, from its manifest.
    pub version: Option<String>,
    pub source: PluginSource,
    /// Its skills and commands, as the agent calls them
    /// (`<plugin>:<name>`).
    pub skills: Vec<String>,
    /// Its subagents, as the agent calls them.
    pub agents: Vec<String>,
    /// When its hooks run commands by themselves: the hook events.
    pub hooks: Vec<String>,
    /// The language servers it runs by itself.
    pub language_servers: Vec<String>,
    /// Its MCP servers, which agents do not get (spec 41.4).
    pub mcp_servers: Vec<String>,
    /// It brings programs the agent can run in its terminal (`bin/`).
    pub programs: bool,
    /// It asks for settings (`userConfig`) the agents do not get.
    pub needs_settings: bool,
    /// The agents that have it: the ones chosen and, when it is for
    /// everyone, every agent on Claude Code.
    pub bot_ids: Vec<BotId>,
    /// Every agent on Claude Code uses it, the ones made later included
    /// (spec 41.3). `everyone` in the list gives every plugin to every
    /// agent.
    pub everyone: bool,
}

/// What `plugins.list` answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PluginList {
    /// By name.
    pub plugins: Vec<Plugin>,
    /// Every agent on Claude Code uses every plugin, the ones that come
    /// later included (spec 41.3).
    pub everyone: bool,
}

/// `plugins.share`: the agents that may use a plugin, in place of those
/// before.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PluginsShareParams {
    pub name: String,
    pub bot_ids: Vec<BotId>,
}

/// `plugins.everyone`: one plugin, or with no name every plugin, for every
/// agent on Claude Code, or not any more.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct PluginsEveryoneParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub name: Option<String>,
    pub on: bool,
}
