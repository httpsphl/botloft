//! Connected tools brought from the owner's own Claude Code (spec 25.8): the
//! servers in its `~/.claude.json`, which the owner picks to copy here.

use serde::{Deserialize, Serialize};

use super::McpKind;

/// Why a server of Claude Code cannot come as it is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum ClaudeMcpProblem {
    /// A way of reaching it Botloft does not take (`sse`, or one unknown).
    Kind { kind: String },
    /// A setting Botloft does not know, such as a sign-in of its own.
    Field { field: String },
}

/// One server of Claude Code, without its secrets: only the names of its
/// headers and variables come to the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ClaudeMcpServer {
    pub name: String,
    /// The folder it is set for; `None` when it is for every folder.
    pub folder: Option<String>,
    /// `None` when it cannot come ([`ClaudeMcpServer::problem`]).
    pub kind: Option<McpKind>,
    pub url: Option<String>,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub header_names: Vec<String>,
    pub env_names: Vec<String>,
    /// A tool with this name is connected in Botloft already.
    pub saved: bool,
    pub problem: Option<ClaudeMcpProblem>,
}

/// The answer of `mcp.fromClaude`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ClaudeMcpServers {
    /// Claude Code's settings file was there to read.
    pub found: bool,
    /// Those for every folder first, then by folder; by name in each.
    pub servers: Vec<ClaudeMcpServer>,
}

/// A server the owner picked, as `mcp.fromClaude` named it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ClaudeMcpPick {
    pub name: String,
    #[serde(default)]
    pub folder: Option<String>,
}

/// Copies the picked servers, secrets and all, read again from Claude
/// Code's file by the daemon: their values never pass through the app.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct McpImportParams {
    pub picks: Vec<ClaudeMcpPick>,
}
