//! Connected tools: MCP servers the owner registered and attached to chosen
//! bots (spec 25).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ids::{BotId, McpServerId};

/// How Claude Code reaches the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum McpKind {
    /// A program Claude Code starts, with the owner's powers.
    Stdio,
    /// A server at an address.
    Http,
}

impl McpKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stdio => "stdio",
            Self::Http => "http",
        }
    }

    pub fn parse(text: &str) -> Option<Self> {
        Some(match text {
            "stdio" => Self::Stdio,
            "http" => Self::Http,
            _ => return None,
        })
    }
}

/// A registered server, without its secrets: the values of headers and
/// environment variables stay in `secrets\mcp` (spec 25.2), and only their
/// names come here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct McpServer {
    pub id: McpServerId,
    pub name: String,
    /// The name Claude Code puts in the tools: `mcp__<slug>__<tool>`.
    pub slug: String,
    pub kind: McpKind,
    /// `http` only.
    pub url: Option<String>,
    /// `stdio` only.
    pub command: Option<String>,
    pub args: Vec<String>,
    /// Names of the headers (`http`); their values are secret.
    pub header_names: Vec<String>,
    /// Names of the environment variables (`stdio`); their values are secret.
    pub env_names: Vec<String>,
    /// What the server is for, in the owner's words. Goes into the rules of
    /// the bots that use it (spec 25.3).
    pub description: String,
    pub created_at: i64,
}

/// Whether Claude Code reached a server when the bot started (spec 25.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum McpState {
    Connected,
    Pending,
    /// The server wants the owner to sign in first.
    NeedsAuth,
    Failed,
}

/// What a bot's process reported about one of its servers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct McpServerState {
    pub server_id: McpServerId,
    pub state: McpState,
    /// Why it failed, as Claude Code put it: for "Details", never the log.
    pub error: Option<String>,
}

/// The servers a bot uses, and how they stand since it started.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotMcp {
    pub bot_id: BotId,
    pub server_ids: Vec<McpServerId>,
    /// Empty until the process says; only for servers it was started with.
    pub states: Vec<McpServerState>,
}

/// Every server and who uses it: the result of `mcp.servers` and the params
/// of the `mcp.servers` notification.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct McpOverview {
    /// Oldest first.
    pub servers: Vec<McpServer>,
    /// Only the bots that use some server.
    pub bots: Vec<BotMcp>,
}

/// Registers a server, or changes one when `server_id` is set. The header
/// and variable values are the secrets: when changing a server, an empty
/// value keeps the one already stored under that name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct McpSaveParams {
    #[serde(default)]
    pub server_id: Option<McpServerId>,
    pub name: String,
    pub kind: McpKind,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct McpServerIdParams {
    pub server_id: McpServerId,
}

/// Replaces the servers a bot uses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotMcpSetParams {
    pub bot_id: BotId,
    pub server_ids: Vec<McpServerId>,
}
