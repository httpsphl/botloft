//! The agent behind a bot (spec 30): Claude Code today, Codex and Antigravity
//! later. What is specific to one agent lives behind [`Agent`]; the rest of the
//! daemon asks it and never writes an agent's command line or stdin framing
//! itself. This first slice owns the process arguments and the encoding of a
//! turn; reading the output and the approvals follow (spec 30.1, A1).

mod claude;

use std::ffi::OsString;
use std::path::Path;

use botloft_core::protocol::{AgentKind, BotEffort, BotModel, PermissionMode};
use bytes::Bytes;

pub use claude::ClaudeAgent;

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
    /// The conversation: resumed when `resumed`, new with this id otherwise.
    pub session: &'a str,
    pub resumed: bool,
    /// The bot's MCP configuration (the Botloft server and the owner's).
    pub mcp_config: &'a Path,
    /// The crew's work folder, which the bot edits like its own (spec 5).
    pub work_folder: &'a Path,
    pub permission_mode: PermissionMode,
    pub model: BotModel,
    pub effort: BotEffort,
}

pub trait Agent: Send + Sync + 'static {
    fn kind(&self) -> AgentKind;

    /// The arguments of the process, after the program.
    fn args(&self, plan: &LaunchPlan<'_>) -> Vec<OsString>;

    /// Variables the agent needs beyond the ones every bot gets.
    fn extra_env(&self) -> Vec<(OsString, OsString)>;

    /// One turn as the bytes written to the process's stdin; `uuid` is what
    /// the agent gives back when the turn begins.
    fn encode_turn(&self, uuid: &str, turn: &Turn) -> Bytes;
}

static CLAUDE: ClaudeAgent = ClaudeAgent;

/// The agent that runs bots of `kind`; `None` while the daemon cannot run
/// that agent yet (spec 30.1).
pub fn of(kind: AgentKind) -> Option<&'static dyn Agent> {
    match kind {
        AgentKind::Claude => Some(&CLAUDE),
        AgentKind::Codex | AgentKind::Agy => None,
    }
}
