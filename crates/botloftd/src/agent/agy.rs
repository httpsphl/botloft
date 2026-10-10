//! Antigravity's `agy` as an agent (spec 30). Experimental: in `-p` it asks
//! the owner for nothing, so Botloft fences it with deny rules and lets it use
//! only its own tools (spec 30.4).
//!
//! The process gets a home of its own, `<bot folder>/.botloft/agy-home`, as
//! `USERPROFILE`: `agy` reads `.gemini/config/mcp_config.json` and
//! `.gemini/antigravity-cli/settings.json` from there, so the owner's own
//! `GEMINI.md`, settings and MCP servers stay out, and the bot's token and
//! rules are its own. The login does not live there and carries over.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use botloft_core::ids::BotId;
use botloft_core::protocol::{AgentKind, TokenUsage};
use bytes::Bytes;
use serde_json::{Map, Value, json};

use super::{Agent, LaunchFiles, LaunchPlan, OutputDecoder, Turn};
use crate::chat::sink;
use crate::state::Daemon;

/// The bot's home, inside its folder.
fn home(workspace: &Path) -> PathBuf {
    workspace.join(".botloft").join("agy-home")
}

pub struct AgyAgent;

impl Agent for AgyAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Agy
    }

    fn locate(&self, configured: &str, _claude: Option<&Path>) -> io::Result<PathBuf> {
        let configured = configured.trim();
        if !configured.is_empty() {
            return Ok(PathBuf::from(configured));
        }
        let mut candidates = Vec::new();
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            candidates.push(PathBuf::from(local).join("agy").join("bin").join("agy.exe"));
        }
        if let Some(path) = std::env::var_os("PATH") {
            for dir in std::env::split_paths(&path) {
                candidates.push(dir.join("agy.exe"));
                candidates.push(dir.join("agy"));
            }
        }
        candidates
            .into_iter()
            .find(|candidate| candidate.is_file())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "agy was not found; install Antigravity or set agy_path in config.toml",
                )
            })
    }

    fn args(&self, plan: &LaunchPlan<'_>) -> Vec<OsString> {
        let mut args: Vec<OsString> = [
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        // A new conversation takes the id `agy` gives it (in `init`).
        if plan.resumed {
            args.push("--conversation".into());
            args.push(plan.session.into());
        }
        args.push("--add-dir".into());
        args.push(plan.work_folder.as_os_str().to_owned());
        // The effort is part of the model's id (`gemini-3.8-flash-low`).
        if let Some(model) = plan.agent_model {
            args.push("--model".into());
            args.push(model.into());
        }
        // `-p` takes the next argument as its prompt unless the value is
        // attached; there is none, the prompts come on stdin.
        args.push("-p=".into());
        args
    }

    fn extra_env(&self, workspace: &Path) -> Vec<(OsString, OsString)> {
        let home = home(workspace).into_os_string();
        vec![
            (OsString::from("USERPROFILE"), home.clone()),
            (OsString::from("HOME"), home),
        ]
    }

    fn write_launch_files(&self, files: &LaunchFiles<'_>) -> io::Result<()> {
        let home = home(files.workspace);
        let config = home.join(".gemini").join("config");
        let cli = home.join(".gemini").join("antigravity-cli");
        std::fs::create_dir_all(&config)?;
        std::fs::create_dir_all(&cli)?;
        let mcp = json!({
            "mcpServers": {
                "botloft": {
                    "disabled": false,
                    "serverUrl": format!("http://127.0.0.1:{}/mcp", files.port),
                    "headers": { "Authorization": format!("Bearer {}", files.token) },
                },
            },
        });
        std::fs::write(config.join("mcp_config.json"), pretty(&mcp))?;
        std::fs::write(
            cli.join("settings.json"),
            pretty(&settings(files.fenced, files.allowed_commands)),
        )?;
        // The rules `prepare_bot` wrote for Claude Code are the bot's rules;
        // `agy` reads `AGENTS.md` from its folder.
        let rules = files
            .workspace
            .join(".claude")
            .join("rules")
            .join("botloft.md");
        if let Ok(text) = std::fs::read_to_string(rules) {
            std::fs::write(files.workspace.join("AGENTS.md"), text)?;
        }
        Ok(())
    }

    fn speaks_control(&self) -> bool {
        // Not seen in `agy 1.3.1`; a Claude-format line is refused with an
        // error in the turn (spec 30.4).
        false
    }

    fn encode_turn(&self, _uuid: &str, turn: &Turn) -> Bytes {
        // The input takes text blocks only (a block of another type fails
        // the turn, spec 30.4). An image reaches the bot as the path the
        // courier lists in the text; the bot opens the file itself.
        let event = json!({
            "event": "user",
            "message": {
                "role": "user",
                "content": [{ "type": "text", "text": turn.text }],
            },
        });
        let mut line = event.to_string().into_bytes();
        line.push(b'\n');
        Bytes::from(line)
    }

    fn decoder(&self) -> Box<dyn OutputDecoder> {
        Box::new(AgyDecoder::default())
    }
}

fn pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

/// A path as `agy`'s rules match it: forward slashes, drive letter kept
/// (seen with 1.3.1; without the drive the rule matched nothing).
fn rule_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    let text = text.strip_prefix(r"\\?\").unwrap_or(&text);
    text.replace('\\', "/")
}

/// `settings.json` (spec 30.3): only Botloft's MCP tools run without asking,
/// since in `-p` nothing can ask; whatever else needs a permission is denied
/// by `agy` itself. Reads and writes of the fenced folders are denied.
fn settings(fenced: &[PathBuf], commands: &[String]) -> Value {
    let mut deny = Vec::new();
    for folder in fenced {
        let folder = rule_path(folder);
        deny.push(format!("read_file({folder})"));
        deny.push(format!("write_file({folder})"));
    }
    let mut allow = vec!["mcp(botloft/*)".to_owned()];
    allow.extend(commands.iter().map(|command| format!("command({command})")));
    json!({
        "permissions": {
            "allow": allow,
            "deny": deny,
        },
    })
}

/// What one process of `agy` printed, as far as the next line needs it.
#[derive(Default)]
struct AgyDecoder {
    /// The reply text written so far in the current response.
    pending: String,
    /// The model of this process, from `init`; sizes the context window.
    model: String,
}

impl OutputDecoder for AgyDecoder {
    fn live_text<'a>(&mut self, event: &'a Value) -> Option<&'a str> {
        let update = step_update(event)?;
        if update["step_type"] != "agent_response" || update["state"] != "ACTIVE" {
            return None;
        }
        let text = update["text_delta"]
            .as_str()
            .filter(|text| !text.is_empty())?;
        self.pending.push_str(text);
        Some(text)
    }

    fn handle(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
        if !daemon.supervisor.is_current(bot, generation) {
            return;
        }
        match event["event"].as_str() {
            Some("init") => {
                if let Some(model) = event["init"]["model"].as_str() {
                    self.model = model.to_owned();
                    crate::service::models::reported(daemon, bot, model);
                }
            }
            Some("step_update") => self.step(daemon, bot, generation, &event["step_update"]),
            Some("result") => self.result(daemon, bot, generation, &event["result"]),
            _ => {}
        }
    }
}

fn step_update(event: &Value) -> Option<&Value> {
    (event["event"] == "step_update").then(|| &event["step_update"])
}

impl AgyDecoder {
    fn step(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, update: &Value) {
        let conversation = update["conversation_id"].as_str().unwrap_or_default();
        match (update["step_type"].as_str(), update["state"].as_str()) {
            // The message was taken up: `agy` gives no id back, so it was the
            // oldest one written.
            (Some("user_input"), Some("DONE")) => {
                if !conversation.is_empty() {
                    sink::session_started(daemon, bot, conversation);
                }
                if let Some(uuid) = daemon.supervisor.oldest_message_began(bot, generation) {
                    sink::message_read(daemon, bot, &uuid);
                }
            }
            (Some("agent_response"), Some("DONE")) => {
                self.flush(daemon, bot);
                self.context(daemon, bot, &update["usage"]);
            }
            (Some("tool"), Some("ACTIVE")) => {
                // What came before the tool is a reply of its own.
                self.flush(daemon, bot);
                let (name, input) = vocabulary(
                    update["tool_name"].as_str().unwrap_or("tool"),
                    &update["tool_info"]["parameters"],
                );
                sink::tool_started(daemon, bot, &tool_id(conversation, update), &name, &input);
            }
            (Some("tool"), Some(state @ ("DONE" | "ERROR"))) => {
                let failed = state == "ERROR";
                let output = update["tool_info"]["error"]["message"]
                    .as_str()
                    .unwrap_or_default();
                sink::tool_finished(daemon, bot, &tool_id(conversation, update), failed, output);
            }
            _ => {}
        }
    }

    fn result(&mut self, daemon: &Daemon, bot: &BotId, generation: u64, result: &Value) {
        self.flush(daemon, bot);
        let failed = result["status"].as_str() != Some("SUCCESS");
        let error = failed.then(|| {
            result["error"]
                .as_str()
                .or_else(|| result["status"].as_str())
                .unwrap_or("error")
                .to_owned()
        });
        let count = |key: &str| result["usage"][key].as_u64().unwrap_or_default();
        let tokens = result["usage"].is_object().then(|| TokenUsage {
            input: count("input_tokens"),
            cache_write: 0,
            reloaded: 0,
            cache_read: count("cache_read_tokens"),
            output: count("output_tokens") + count("thinking_tokens"),
        });
        // `duration_seconds` is a float.
        let seconds = result["duration_seconds"].as_f64().unwrap_or_default();
        sink::turn_finished(daemon, bot, (seconds * 1000.0) as u64, tokens, error);
        sink::finish_turn(daemon, bot, generation, failed);
    }

    /// What the last request held is what the conversation holds: its
    /// prompt, cached or not, and what it wrote.
    fn context(&self, daemon: &Daemon, bot: &BotId, usage: &Value) {
        let count = |key: &str| usage[key].as_u64().unwrap_or_default();
        let used = count("input_tokens") + count("cache_read_tokens") + count("output_tokens");
        if used > 0 {
            crate::context::report(daemon, bot, used, window_of(&self.model));
        }
    }

    /// The reply written so far becomes a chat item.
    fn flush(&mut self, daemon: &Daemon, bot: &BotId) {
        let text = std::mem::take(&mut self.pending);
        sink::reply(daemon, bot, &text);
    }
}

/// How many tokens a conversation of `model` holds at most, by the family in
/// its id: `agy` does not say (spec 30.4). A round figure, not a promise.
fn window_of(model: &str) -> u64 {
    if model.starts_with("gemini") {
        1_000_000
    } else if model.starts_with("claude") {
        200_000
    } else {
        128_000
    }
}

/// The models `agy models` lists: an id and a name for people, tab apart.
pub fn models(program: &Path) -> io::Result<Vec<botloft_core::protocol::AgentModel>> {
    let output = std::process::Command::new(program).arg("models").output()?;
    if !output.status.success() {
        return Err(io::Error::other("agy models failed"));
    }
    Ok(parse_models(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_models(text: &str) -> Vec<botloft_core::protocol::AgentModel> {
    text.lines()
        .filter_map(|line| {
            let (id, name) = line.split_once('\t')?;
            let id = id.trim();
            // The line "Fetching available models..." has no tab.
            (!id.is_empty() && !id.contains(' ')).then(|| botloft_core::protocol::AgentModel {
                id: id.to_owned(),
                name: name.trim().to_owned(),
            })
        })
        .collect()
}

/// An id for a tool step, unique among the bot's conversations.
fn tool_id(conversation: &str, update: &Value) -> String {
    format!(
        "{conversation}-{}",
        update["step_index"].as_u64().unwrap_or_default()
    )
}

/// The tool in the names the chat knows (spec 8.4), so that it shows as a
/// command or a file edit. Others keep their own name and parameters.
fn vocabulary(name: &str, parameters: &Value) -> (String, Value) {
    let text = |key: &str| parameters[key].clone();
    let mut input = Map::new();
    let mapped = match name {
        "write_to_file" => {
            input.insert("file_path".into(), text("TargetFile"));
            "Write"
        }
        "replace_file_content" | "multi_replace_file_content" | "sed_file" => {
            input.insert("file_path".into(), text("TargetFile"));
            "Edit"
        }
        "run_command" => {
            input.insert("command".into(), text("CommandLine"));
            "Bash"
        }
        "view_file" => {
            input.insert("file_path".into(), text("AbsolutePath"));
            "Read"
        }
        "search_web" => {
            input.insert("query".into(), text("query"));
            "WebSearch"
        }
        "read_url_content" => {
            input.insert("url".into(), text("Url"));
            "WebFetch"
        }
        _ => return (name.to_owned(), parameters.clone()),
    };
    (mapped.to_owned(), Value::Object(input))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::TurnImage;

    #[test]
    fn a_turn_is_one_user_event_with_text_only() {
        let turn = Turn {
            text: "hi".into(),
            images: vec![TurnImage {
                media_type: "image/png".into(),
                data: "AAAA".into(),
            }],
        };
        let line = AgyAgent.encode_turn("ignored", &turn);
        assert!(line.ends_with(b"\n"));
        let json: Value = serde_json::from_slice(&line).expect("json");
        assert_eq!(json["event"], "user");
        assert_eq!(json["message"]["role"], "user");
        assert_eq!(json["message"]["content"][0]["text"], "hi");
        // An image block would fail the turn: `agy` takes text only.
        assert_eq!(json["message"]["content"].as_array().map(Vec::len), Some(1));
    }

    #[test]
    fn the_model_list_skips_the_line_that_is_not_one() {
        let models = parse_models(
            "Fetching available models...\ngemini-3.8-flash-low\tGemini 3.8 Flash (Low)\n\
             claude-opus-5-5-high\tClaude Opus 5.5 (High)\n",
        );
        assert_eq!(models.len(), 2);
        assert_eq!(models[0].id, "gemini-3.8-flash-low");
        assert_eq!(models[1].name, "Claude Opus 5.5 (High)");
        assert_eq!(window_of("gemini-3.8-flash-low"), 1_000_000);
        assert_eq!(window_of("claude-opus-5-5-high"), 200_000);
    }

    #[test]
    fn a_new_conversation_has_no_id_and_a_resumed_one_names_it() {
        let plan = |resumed| LaunchPlan {
            session: "conv-1",
            resumed,
            mcp_config: Path::new("m.json"),
            work_folder: Path::new("w"),
            permission_mode: botloft_core::protocol::PermissionMode::Default,
            model: botloft_core::protocol::BotModel::Default,
            effort: botloft_core::protocol::BotEffort::Default,
            agent_model: Some("gemini-3.8-flash-low"),
        };
        let text = |args: Vec<OsString>| -> Vec<String> {
            args.into_iter()
                .map(|a| a.to_string_lossy().into_owned())
                .collect()
        };
        let fresh = text(AgyAgent.args(&plan(false)));
        assert!(!fresh.contains(&"--conversation".to_owned()));
        assert_eq!(fresh.last().map(String::as_str), Some("-p="));
        let at = fresh.iter().position(|a| a == "--model").expect("model");
        assert_eq!(fresh[at + 1], "gemini-3.8-flash-low");
        let resumed = text(AgyAgent.args(&plan(true)));
        let at = resumed
            .iter()
            .position(|a| a == "--conversation")
            .expect("flag");
        assert_eq!(resumed[at + 1], "conv-1");
    }

    #[test]
    fn rules_name_folders_with_the_drive_and_forward_slashes() {
        assert_eq!(
            rule_path(Path::new(r"C:\Users\x\Botloft\a")),
            "C:/Users/x/Botloft/a"
        );
        assert_eq!(rule_path(Path::new(r"\\?\C:\Users\x")), "C:/Users/x");
        let settings = settings(&[PathBuf::from(r"C:\Data")], &["git status".to_owned()]);
        let deny = settings["permissions"]["deny"].as_array().expect("deny");
        assert!(deny.contains(&json!("write_file(C:/Data)")));
        assert!(deny.contains(&json!("read_file(C:/Data)")));
        assert_eq!(settings["permissions"]["allow"][0], "mcp(botloft/*)");
        assert_eq!(settings["permissions"]["allow"][1], "command(git status)");
    }

    #[test]
    fn tools_take_the_names_the_chat_knows() {
        let (name, input) = vocabulary("run_command", &json!({ "CommandLine": "echo hi" }));
        assert_eq!(
            (name.as_str(), input["command"].as_str()),
            ("Bash", Some("echo hi"))
        );
        let (name, input) = vocabulary("write_to_file", &json!({ "TargetFile": "C:/a/b.txt" }));
        assert_eq!(
            (name.as_str(), input["file_path"].as_str()),
            ("Write", Some("C:/a/b.txt"))
        );
        let (name, _) = vocabulary("browser_get_dom", &json!({}));
        assert_eq!(name, "browser_get_dom");
    }

    #[test]
    fn reply_text_gathers_until_the_response_is_done() {
        let mut decoder = AgyDecoder::default();
        let delta = |text: &str| {
            json!({ "event": "step_update", "step_update": {
                "step_type": "agent_response", "state": "ACTIVE", "text_delta": text } })
        };
        assert_eq!(decoder.live_text(&delta("po")), Some("po"));
        assert_eq!(decoder.live_text(&delta("ng")), Some("ng"));
        assert_eq!(decoder.pending, "pong");
        // A line of another kind is not live text.
        let done = json!({ "event": "result", "result": {} });
        assert_eq!(decoder.live_text(&done), None);
    }
}
