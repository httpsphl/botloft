//! OpenAI's Codex as an agent (spec 30), through `codex app-server`: one
//! process per bot, JSON-RPC on stdin and stdout. Experimental. The first
//! slice runs the thread in a read-only sandbox that asks for nothing, so the
//! bot reads and answers; approvals come next (spec 30.1).

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use botloft_core::ids::BotId;
use botloft_core::protocol::{AgentKind, TokenUsage};
use bytes::Bytes;
use serde_json::{Value, json};

use super::codex_wire::{Answer, CodexControl, Shared};
use super::{Agent, AttachInput, LaunchFiles, LaunchPlan, OutputDecoder, Turn};
use crate::chat::sink;
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
        let shared = Shared::begin(control, input);
        let decoder = CodexDecoder {
            shared: Arc::clone(&shared),
            thread: None,
            window: 0,
            base: (0, 0, 0, 0),
            usage: (0, 0, 0, 0),
        };
        (Box::new(CodexControl { shared }), Box::new(decoder))
    }
}

struct Silent;

impl OutputDecoder for Silent {
    fn live_text<'a>(&mut self, _event: &'a Value) -> Option<&'a str> {
        None
    }

    fn handle(&mut self, _daemon: &Daemon, _bot: &BotId, _generation: u64, _event: &Value) {}
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

/// Reads what the server prints and tells the daemon.
struct CodexDecoder {
    shared: Arc<Shared>,
    thread: Option<String>,
    /// The model's context window, from `thread/tokenUsage/updated`.
    window: u64,
    /// What the thread has used (input, cached, output, reasoning), and what
    /// it had used when this turn began.
    usage: (u64, u64, u64, u64),
    base: (u64, u64, u64, u64),
}

impl OutputDecoder for CodexDecoder {
    fn live_text<'a>(&mut self, event: &'a Value) -> Option<&'a str> {
        if event["method"] != "item/agentMessage/delta" {
            return None;
        }
        event["params"]["delta"]
            .as_str()
            .filter(|text| !text.is_empty())
    }

    fn handle(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
        if !daemon.supervisor.is_current(bot, generation) {
            return;
        }
        let method = event["method"].as_str();
        match (method, event.get("id")) {
            (None, Some(_)) => self.response(daemon, bot, generation, event),
            (Some(method), Some(id)) => self.server_request(method, id, &event["params"]),
            (Some(method), None) => {
                self.notification(daemon, bot, generation, method, &event["params"])
            }
            (None, None) => {}
        }
    }
}

impl CodexDecoder {
    fn response(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
        match self.shared.answered(event) {
            Answer::Thread { thread, model } => {
                self.thread = Some(thread);
                if let Some(model) = model {
                    crate::service::models::reported(daemon, bot, &model);
                }
            }
            Answer::TurnRefused(message) => {
                // No turn began, so the message is read and the turn failed.
                if let Some(uuid) = daemon.supervisor.oldest_message_began(bot, generation) {
                    sink::message_read(daemon, bot, &uuid);
                }
                let error = message.unwrap_or_else(|| "the turn was refused".to_owned());
                sink::turn_finished(daemon, bot, 0, None, Some(error));
                sink::finish_turn(daemon, bot, generation, true);
            }
            Answer::Failed(message) => {
                tracing::warn!(bot = %bot, "codex could not start a thread: {message:?}");
            }
            Answer::Lost | Answer::None => {}
        }
    }

    /// Requests the daemon does not take yet (A3a) are declined, except the
    /// call of Botloft's own tools, which the bot needs to work in the crew.
    fn server_request(&self, method: &str, id: &Value, params: &Value) {
        let result = match method {
            "mcpServer/elicitation/request" if params["serverName"] == "botloft" => {
                json!({ "action": "accept", "content": null })
            }
            "mcpServer/elicitation/request" => json!({ "action": "decline", "content": null }),
            "item/commandExecution/requestApproval" | "item/fileChange/requestApproval" => {
                json!({ "decision": "decline" })
            }
            "applyPatchApproval" | "execCommandApproval" => json!({ "decision": "denied" }),
            _ => {
                self.shared.refuse(id, "not supported by Botloft yet");
                return;
            }
        };
        self.shared.respond(id, result);
    }

    fn notification(
        &mut self,
        daemon: &Daemon,
        bot: &BotId,
        generation: u64,
        method: &str,
        params: &Value,
    ) {
        match method {
            "turn/started" => {
                self.base = self.usage;
                daemon.supervisor.turn_began(bot, generation);
            }
            "item/started" => self.item_started(daemon, bot, generation, params),
            "item/completed" => self.item_completed(daemon, bot, &params["item"]),
            "thread/tokenUsage/updated" => self.token_usage(daemon, bot, &params["tokenUsage"]),
            "turn/completed" => self.turn_completed(daemon, bot, generation, &params["turn"]),
            _ => {}
        }
    }

    fn item_started(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, params: &Value) {
        let item = &params["item"];
        let id = item["id"].as_str().unwrap_or_default();
        match item["type"].as_str() {
            // The message was taken up: the oldest one written.
            Some("userMessage") => {
                if let Some(thread) = params["threadId"].as_str().or(self.thread.as_deref()) {
                    sink::session_started(daemon, bot, thread);
                }
                if let Some(uuid) = daemon.supervisor.oldest_message_began(bot, generation) {
                    sink::message_read(daemon, bot, &uuid);
                }
            }
            Some("commandExecution") => {
                let command = item["commandActions"][0]["command"]
                    .as_str()
                    .or_else(|| item["command"].as_str())
                    .unwrap_or_default();
                sink::tool_started(daemon, bot, id, "Bash", &json!({ "command": command }));
            }
            Some("fileChange") => {
                let change = &item["changes"][0];
                let name = if change["kind"]["type"] == "add" {
                    "Write"
                } else {
                    "Edit"
                };
                sink::tool_started(
                    daemon,
                    bot,
                    id,
                    name,
                    &json!({ "file_path": change["path"] }),
                );
            }
            Some("mcpToolCall") => {
                let name = format!(
                    "mcp__{}__{}",
                    item["server"].as_str().unwrap_or("mcp"),
                    item["tool"].as_str().unwrap_or("tool")
                );
                sink::tool_started(daemon, bot, id, &name, &item["arguments"]);
            }
            _ => {}
        }
    }

    fn item_completed(&mut self, daemon: &Daemon, bot: &BotId, item: &Value) {
        let id = item["id"].as_str().unwrap_or_default();
        let ok = item["status"].as_str() == Some("completed");
        match item["type"].as_str() {
            Some("agentMessage") => {
                sink::reply(daemon, bot, item["text"].as_str().unwrap_or_default());
            }
            Some("commandExecution") => {
                let failed = !ok || item["exitCode"].as_i64().is_some_and(|code| code != 0);
                let output = item["aggregatedOutput"].as_str().unwrap_or_default();
                sink::tool_finished(daemon, bot, id, failed, output);
            }
            Some("fileChange") => sink::tool_finished(daemon, bot, id, !ok, ""),
            Some("mcpToolCall") => {
                let output = item["result"]["content"]
                    .as_array()
                    .map(|parts| {
                        parts
                            .iter()
                            .filter_map(|part| part["text"].as_str())
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .or_else(|| item["error"]["message"].as_str().map(str::to_owned))
                    .unwrap_or_default();
                sink::tool_finished(daemon, bot, id, !ok, &output);
            }
            _ => {}
        }
    }

    fn token_usage(&mut self, daemon: &Daemon, bot: &BotId, usage: &Value) {
        let count = |value: &Value, key: &str| value[key].as_u64().unwrap_or_default();
        self.window = count(usage, "modelContextWindow");
        let last = &usage["last"];
        self.usage = (
            count(&usage["total"], "inputTokens"),
            count(&usage["total"], "cachedInputTokens"),
            count(&usage["total"], "outputTokens"),
            count(&usage["total"], "reasoningOutputTokens"),
        );
        // The last request held the conversation up to that point.
        let held = count(last, "inputTokens") + count(last, "outputTokens");
        if held > 0 && self.window > 0 {
            crate::context::report(daemon, bot, held, self.window);
        }
    }

    fn turn_completed(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, turn: &Value) {
        let failed = turn["status"].as_str() != Some("completed");
        let error = failed.then(|| {
            turn["error"]["message"]
                .as_str()
                .or_else(|| turn["status"].as_str())
                .unwrap_or("error")
                .to_owned()
        });
        let (input, cached, output, reasoning) = (
            self.usage.0.saturating_sub(self.base.0),
            self.usage.1.saturating_sub(self.base.1),
            self.usage.2.saturating_sub(self.base.2),
            self.usage.3.saturating_sub(self.base.3),
        );
        let tokens = (self.usage != self.base).then(|| TokenUsage {
            input: input.saturating_sub(cached),
            cache_write: 0,
            reloaded: 0,
            cache_read: cached,
            output: output + reasoning,
        });
        let duration = turn["durationMs"].as_u64().unwrap_or_default();
        sink::turn_finished(daemon, bot, duration, tokens, error);
        self.shared.turn_done();
        sink::finish_turn(daemon, bot, generation, failed);
    }
}
