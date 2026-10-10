//! The agent behind a bot (spec 30): Claude Code, and Antigravity (`agy`) as an
//! experimental one; Codex later. What is specific to one agent lives behind
//! [`Agent`]; the rest of the daemon asks it and never writes an agent's
//! command line, stdin framing or output format itself.

mod agy;
mod claude;
mod codex;
mod codex_decoder;
mod codex_wire;

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::{AgentKind, BotEffort, BotModel, PermissionMode};
use bytes::Bytes;
use serde_json::Value;

pub use agy::{AgyAgent, models as agy_models};
pub use claude::ClaudeAgent;
pub use codex::CodexAgent;

use crate::runtime::ProcessControl;
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

/// What an agent that keeps a conversation with its process needs once the
/// process exists (spec 30): where the bot works, its token, what to resume.
#[derive(Debug, Clone)]
pub struct AttachInput {
    pub workspace: PathBuf,
    pub port: u16,
    /// The token of this process, in the clear: it goes to the agent's MCP
    /// configuration.
    pub token: String,
    /// The conversation to resume; `None` starts a new one.
    pub resume: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    /// Folders its tools must not touch (spec 7.5): a request that does is
    /// refused without asking the owner.
    pub fenced: Vec<PathBuf>,
    /// Whether it may ask the owner at all; a plan only reads.
    pub may_ask: bool,
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
    fn handle(&mut self, daemon: &Arc<Daemon>, bot: &BotId, generation: u64, event: &Value);
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

    /// What the supervisor writes to and what reads the output, for a
    /// process that was just created. An agent that has a conversation with
    /// its process (Codex) wraps `control`; the others leave it as it is.
    fn attach(
        &self,
        control: Box<dyn ProcessControl>,
        _input: AttachInput,
    ) -> (Box<dyn ProcessControl>, Box<dyn OutputDecoder>) {
        (control, self.decoder())
    }
}

/// Botloft's own line for a turn, for agents whose stdin is a protocol of
/// their own that a wrapper speaks: text only, as `agy` takes it.
pub(crate) fn neutral_turn(turn: &Turn) -> Bytes {
    let event = serde_json::json!({
        "event": "user",
        "message": { "role": "user", "content": [{ "type": "text", "text": turn.text }] },
    });
    let mut line = event.to_string().into_bytes();
    line.push(b'\n');
    Bytes::from(line)
}

/// The rules `prepare_bot` wrote for Claude Code are the bot's rules; the
/// other agents read `AGENTS.md` from the bot's folder.
pub(crate) fn copy_rules_to_agents_md(workspace: &Path) -> io::Result<()> {
    let rules = workspace.join(".claude").join("rules").join("botloft.md");
    if let Ok(text) = std::fs::read_to_string(rules) {
        std::fs::write(workspace.join("AGENTS.md"), text)?;
    }
    Ok(())
}

/// The first line `program --version` prints; `None` if it cannot run.
pub fn program_version(program: &Path) -> Option<String> {
    let output = std::process::Command::new(program)
        .arg("--version")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next()?.trim();
    (!line.is_empty()).then(|| line.to_owned())
}

static CLAUDE: ClaudeAgent = ClaudeAgent;
static AGY: AgyAgent = AgyAgent;
static CODEX: CodexAgent = CodexAgent;

/// The agent that runs bots of `kind`.
pub fn of(kind: AgentKind) -> Option<&'static dyn Agent> {
    // Every agent has an implementation; whether it may run is the daemon's
    // setting (`Supervisor::agent_enabled`).
    match kind {
        AgentKind::Claude => Some(&CLAUDE),
        AgentKind::Agy => Some(&AGY),
        AgentKind::Codex => Some(&CODEX),
    }
}
