//! The agent behind a bot (spec 30): Claude Code, and Antigravity (`agy`) as an
//! experimental one; Codex later. What is specific to one agent lives behind
//! [`Agent`]; the rest of the daemon asks it and never writes an agent's
//! command line, stdin framing or output format itself.

mod agy;
mod claude;

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use botloft_core::ids::BotId;
use botloft_core::protocol::{AgentKind, BotEffort, BotModel, PermissionMode};
use bytes::Bytes;
use serde_json::Value;

pub use agy::{AgyAgent, models as agy_models};
pub use claude::ClaudeAgent;

use crate::state::Daemon;

/// One image that goes in the turn, already read and encoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnImage {
    pub media_type: String,
    /// The file's bytes, base64.
    pub data: String,
}

/// What a bot is told in one turn, whatever the agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Turn {
    pub text: String,
    pub images: Vec<TurnImage>,
}

/// What an agent needs to build the command line of a bot's process.
pub struct LaunchPlan<'a> {
    /// A new conversation's id (Claude Code takes the one given) or, when
    /// `resumed`, the one to resume.
    pub session: &'a str,
    pub resumed: bool,
    /// The bot's MCP configuration (the Botloft server and the owner's).
    pub mcp_config: &'a Path,
    /// The crew's work folder, which the bot edits like its own (spec 5).
    pub work_folder: &'a Path,
    pub permission_mode: PermissionMode,
    pub model: BotModel,
    pub effort: BotEffort,
    /// The model of a bot not on Claude Code, as its agent names it.
    pub agent_model: Option<&'a str>,
}

/// What an agent needs to write the files it reads at start, once the bot
/// has its token for this process.
pub struct LaunchFiles<'a> {
    pub workspace: &'a Path,
    pub port: u16,
    pub token: &'a str,
    /// Folders the bot must not read or edit with its own tools (spec 7.5).
    pub fenced: &'a [PathBuf],
    /// Command prefixes the owner lets the bot run without asking.
    pub allowed_commands: &'a [String],
}

/// Reads one process's output, line by line (already JSON). One per process,
/// so it may keep what it needs between lines.
pub trait OutputDecoder: Send {
    /// The piece of reply text this line carries while the reply is still
    /// being written, if that is what it is (spec 8.3).
    fn live_text<'a>(&mut self, event: &'a Value) -> Option<&'a str>;

    /// What the line means for the chat and the bot's state: it saves items
    /// and tells the supervisor, through `chat::sink` for what every agent has
    /// and its own code for the rest. Lines from a process that was replaced
    /// say nothing about the new one.
    fn handle(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, event: &Value);
}

pub trait Agent: Send + Sync + 'static {
    fn kind(&self) -> AgentKind;

    /// The executable, from the configured path or the usual places.
    /// `claude` is the Claude Code the supervisor found, when it found one.
    fn locate(&self, configured: &str, claude: Option<&Path>) -> io::Result<PathBuf>;

    /// The arguments of the process, after the program.
    fn args(&self, plan: &LaunchPlan<'_>) -> Vec<OsString>;

    /// Variables the agent needs beyond the ones every bot gets.
    fn extra_env(&self, workspace: &Path) -> Vec<(OsString, OsString)>;

    /// Writes what the agent reads from the bot's folder at start and that
    /// `workspace::prepare_bot` does not (spec 30.2).
    fn write_launch_files(&self, files: &LaunchFiles<'_>) -> io::Result<()>;

    /// Whether the process answers Botloft's own requests on stdin (its
    /// settings, how full its conversation is, the state of its MCP servers,
    /// `/compact`). An agent that does not shows none of those (spec 30.2).
    fn speaks_control(&self) -> bool;

    /// One turn as the bytes written to the process's stdin; `uuid` is what
    /// the agent gives back when the turn begins, for those that do.
    fn encode_turn(&self, uuid: &str, turn: &Turn) -> Bytes;

    /// A reader for the output of one process.
    fn decoder(&self) -> Box<dyn OutputDecoder>;
}

static CLAUDE: ClaudeAgent = ClaudeAgent;
static AGY: AgyAgent = AgyAgent;

/// The agent that runs bots of `kind`; `None` for one the daemon does not
/// have yet (spec 30.1).
pub fn of(kind: AgentKind) -> Option<&'static dyn Agent> {
    match kind {
        AgentKind::Claude => Some(&CLAUDE),
        AgentKind::Agy => Some(&AGY),
        AgentKind::Codex => None,
    }
}
