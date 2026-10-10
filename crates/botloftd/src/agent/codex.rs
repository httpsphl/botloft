//! OpenAI's Codex as an agent (spec 30), through `codex app-server`: one
//! process per bot, JSON-RPC on stdin and stdout. Experimental. The thread
//! runs in a read-only sandbox; whatever needs more is asked of the owner in
//! the chat (`codex_decoder.rs`).

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::AgentKind;
use bytes::Bytes;
use serde_json::Value;

use super::codex_decoder::CodexDecoder;
use super::codex_wire::{CodexControl, Shared};
use super::{Agent, AttachInput, LaunchFiles, LaunchPlan, OutputDecoder, Turn};
use crate::runtime::ProcessControl;
use crate::state::Daemon;

pub struct CodexAgent;

impl Agent for CodexAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Codex
    }

    fn locate(&self, configured: &str, _claude: Option<&Path>) -> io::Result<PathBuf> {
        let configured = configured.trim();
        if !configured.is_empty() {
            return Ok(PathBuf::from(configured));
        }
        let mut candidates = Vec::new();
        // The desktop app keeps the executable in a folder named by a hash.
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let bin = PathBuf::from(local)
                .join("OpenAI")
                .join("Codex")
                .join("bin");
            let mut found: Vec<_> = std::fs::read_dir(bin)
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path().join("codex.exe"))
                .filter(|path| path.is_file())
                .collect();
            found.sort_by_key(|path| std::fs::metadata(path).and_then(|m| m.modified()).ok());
            candidates.extend(found.into_iter().rev());
        }
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path) {
                candidates.push(dir.join("codex.exe"));
                candidates.push(dir.join("codex"));
            }
        }
        candidates
            .into_iter()
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "codex was not found; install Codex or set codex_path in config.toml",
                )
            })
    }

    fn args(&self, _plan: &LaunchPlan<'_>) -> Vec<OsString> {
        let mut overrides = vec![
            // The owner's own apps, hooks, plugins and notifier are theirs
            // (spec 30.3); their MCP servers go by name below.
            "features.apps=false".to_owned(),
            "features.hooks=false".to_owned(),
            "features.plugins=false".to_owned(),
            "notify=[]".to_owned(),
        ];
        overrides.extend(
            owner_mcp_servers()
                .into_iter()
                .map(|name| format!("mcp_servers.{name}.enabled=false")),
        );
        let mut args = vec![OsString::from("app-server")];
        for value in overrides {
            args.push("-c".into());
            args.push(value.into());
        }
        args
    }

    fn extra_env(&self, _workspace: &Path) -> Vec<(OsString, OsString)> {
        Vec::new()
    }

    fn write_launch_files(&self, files: &LaunchFiles<'_>) -> io::Result<()> {
        super::copy_rules_to_agents_md(files.workspace)
    }

    fn speaks_control(&self) -> bool {
        false
    }

    fn encode_turn(&self, _uuid: &str, turn: &Turn) -> Bytes {
        // The wrapper turns this neutral line into a `turn/start`.
        super::neutral_turn(turn)
    }

    fn decoder(&self) -> Box<dyn OutputDecoder> {
        // Not used: `attach` makes the decoder, which needs the process.
        Box::new(Silent)
    }

    fn attach(
        &self,
        control: Box<dyn ProcessControl>,
        input: AttachInput,
    ) -> (Box<dyn ProcessControl>, Box<dyn OutputDecoder>) {
        let fenced = input.fenced.clone();
        let shared = Shared::begin(control, input);
        let decoder = CodexDecoder::new(Arc::clone(&shared), fenced);
        (Box::new(CodexControl { shared }), Box::new(decoder))
    }
}

struct Silent;

impl OutputDecoder for Silent {
    fn live_text<'a>(&mut self, _event: &'a Value) -> Option<&'a str> {
        None
    }

    fn handle(&mut self, _daemon: &Arc<Daemon>, _bot: &BotId, _generation: u64, _event: &Value) {}
}

/// The names of the MCP servers in the owner's `config.toml`, to switch off.
fn owner_mcp_servers() -> Vec<String> {
    let home = std::env::var_os("CODEX_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("USERPROFILE").map(|p| PathBuf::from(p).join(".codex")));
    let Some(text) = home.and_then(|home| std::fs::read_to_string(home.join("config.toml")).ok())
    else {
        return Vec::new();
    };
    let Ok(document) = text.parse::<toml_edit::DocumentMut>() else {
        return Vec::new();
    };
    document
        .get("mcp_servers")
        .and_then(toml_edit::Item::as_table_like)
        .map(|servers| {
            servers
                .iter()
                .map(|(name, _)| name.to_owned())
                // A name that is not plain would need quoting in the flag.
                .filter(|name| {
                    name.chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                })
                .collect()
        })
        .unwrap_or_default()
}
