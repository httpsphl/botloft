//! Claude Code as an agent (spec 7.4, 9.2): `claude -p` with `stream-json` in
//! both directions.

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use botloft_core::protocol::AgentKind;
use bytes::Bytes;
use serde_json::{Value, json};

use super::{Agent, LaunchFiles, LaunchPlan, OutputDecoder, Turn};
use crate::chat::claude;
use crate::seat::Seat;
use crate::state::Daemon;

/// Tools the bot uses without asking: its crew tools (spec 7.4).
const ALLOWED_TOOLS: &str = "mcp__botloft";
/// Claude Code's own schedulers: they die with the process and Botloft never
/// sees them. Work at set times is a routine (spec 7.4, 20).
const DISALLOWED_TOOLS: &str = "CronCreate,CronDelete,CronList,ScheduleWakeup,RemoteTrigger";
/// Makes Claude Code load `CLAUDE.md` from `--add-dir` folders (spec 5).
const ADDITIONAL_MEMORY: &str = "CLAUDE_CODE_ADDITIONAL_DIRECTORIES_CLAUDE_MD";
/// Where permission requests go (spec 10.1).
const PERMISSION_TOOL: &str = "mcp__botloft__permission_prompt";

pub struct ClaudeAgent;

impl Agent for ClaudeAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Claude
    }

    fn locate(&self, _configured: &str, claude: Option<&Path>) -> io::Result<PathBuf> {
        // Found and checked by the supervisor, which keeps it up to date.
        claude
            .map(Path::to_path_buf)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Claude Code was not found"))
    }

    fn args(&self, plan: &LaunchPlan<'_>) -> Vec<OsString> {
        let mut args: Vec<OsString> = [
            "-p",
            "--input-format",
            "stream-json",
            "--output-format",
            "stream-json",
            "--verbose",
            "--include-partial-messages",
            "--replay-user-messages",
        ]
        .into_iter()
        .map(OsString::from)
        .collect();
        args.push(
            if plan.resumed {
                "--resume"
            } else {
                "--session-id"
            }
            .into(),
        );
        args.push(plan.session.into());
        args.push("--setting-sources".into());
        args.push("project,local".into());
        // The owner's own Chrome, with their accounts signed in, comes only
        // to an agent the owner gave it to, never with the owner's
        // environment or their "enabled by default": the flag beats both
        // (spec 7.4, 37.2).
        args.push(
            if plan.chrome {
                "--chrome"
            } else {
                "--no-chrome"
            }
            .into(),
        );
        // Claude Code fetches the claude.ai connectors only without it: an
        // agent given some runs without, with every other server blocked by
        // name (spec 34.5).
        if plan.connectors.is_empty() {
            args.push("--strict-mcp-config".into());
        }
        let mut disallowed = String::from(DISALLOWED_TOOLS);
        for server in plan.blocked_servers {
            disallowed.push_str(&format!(",mcp__{server}"));
        }
        for arg in [
            "--permission-mode",
            plan.permission_mode.cli_value(),
            "--permission-prompt-tool",
            PERMISSION_TOOL,
            "--allowedTools",
            ALLOWED_TOOLS,
            "--disallowedTools",
            disallowed.as_str(),
            "--mcp-config",
        ] {
            args.push(arg.into());
        }
        args.push(plan.mcp_config.as_os_str().to_owned());
        args.push("--add-dir".into());
        args.push(plan.work_folder.as_os_str().to_owned());
        // The owner's plugins it was given, each a copy in its folder (spec
        // 41.3).
        for dir in plan.plugin_dirs {
            args.push("--plugin-dir".into());
            args.push(dir.as_os_str().to_owned());
        }
        // Without the flag, Claude Code uses the default of the owner's plan.
        if let Some(model) = plan.model.cli_value() {
            args.push("--model".into());
            args.push(model.into());
        }
        // Without the flag, Claude Code uses the level it sets for the model.
        if let Some(effort) = plan.effort.cli_value() {
            args.push("--effort".into());
            args.push(effort.into());
        }
        args
    }

    fn extra_env(&self, _workspace: &Path) -> Vec<(OsString, OsString)> {
        // Loads the work folder's CLAUDE.md with the bot's memory.
        vec![(OsString::from(ADDITIONAL_MEMORY), OsString::from("1"))]
    }

    fn write_launch_files(&self, _files: &LaunchFiles<'_>) -> io::Result<()> {
        // `workspace::prepare_bot` writes everything Claude Code reads, and
        // it expands the token from the environment.
        Ok(())
    }

    fn speaks_control(&self) -> bool {
        true
    }

    fn encode_turn(&self, uuid: &str, turn: &Turn) -> Bytes {
        let mut content = vec![json!({ "type": "text", "text": turn.text })];
        content.extend(turn.images.iter().map(|image| {
            json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": image.media_type,
                    "data": image.data,
                },
            })
        }));
        let event = json!({
            "type": "user",
            "uuid": uuid,
            "message": { "role": "user", "content": content },
        });
        let mut line = event.to_string().into_bytes();
        line.push(b'\n');
        Bytes::from(line)
    }

    fn decoder(&self) -> Box<dyn OutputDecoder> {
        Box::new(ClaudeDecoder)
    }
}

/// Claude Code's stream keeps nothing between lines.
struct ClaudeDecoder;

impl OutputDecoder for ClaudeDecoder {
    fn live_text<'a>(&mut self, event: &'a Value) -> Option<&'a str> {
        claude::live_text(event)
    }

    fn handle(&mut self, daemon: &Arc<Daemon>, bot: &Seat, generation: u64, event: &Value) {
        claude::apply(daemon, bot, generation, event);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use botloft_core::protocol::{BotEffort, BotModel, PermissionMode};

    use super::*;
    use crate::agent::TurnImage;

    fn plan(resumed: bool, model: BotModel, effort: BotEffort) -> LaunchPlan<'static> {
        LaunchPlan {
            session: "abc",
            resumed,
            mcp_config: Path::new("w/.botloft/mcp.json"),
            work_folder: Path::new("w/shared"),
            permission_mode: PermissionMode::Default,
            model,
            effort,
            agent_model: None,
            connectors: &[],
            blocked_servers: &[],
            chrome: false,
            plugin_dirs: &[],
        }
    }

    fn strings(args: Vec<OsString>) -> Vec<String> {
        args.into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn a_new_bot_gets_a_session_id_and_the_defaults_leave_model_and_effort_out() {
        let args = strings(ClaudeAgent.args(&plan(false, BotModel::Default, BotEffort::Default)));
        let at = args.iter().position(|a| a == "--session-id").expect("flag");
        assert_eq!(args[at + 1], "abc");
        assert!(!args.contains(&"--resume".to_owned()));
        assert!(!args.contains(&"--model".to_owned()));
        assert!(!args.contains(&"--effort".to_owned()));
        assert!(args.contains(&"--permission-prompt-tool".to_owned()));
        assert!(args.contains(&"--no-chrome".to_owned()));
        assert!(!args.contains(&"--chrome".to_owned()));
        assert!(!args.contains(&"--plugin-dir".to_owned()));
    }

    #[test]
    fn each_plugin_given_is_one_plugin_dir() {
        let dirs = [
            PathBuf::from("w/.claude/plugins/data"),
            PathBuf::from("w/.claude/plugins/vercel"),
        ];
        let args = strings(ClaudeAgent.args(&LaunchPlan {
            plugin_dirs: &dirs,
            ..plan(false, BotModel::Default, BotEffort::Default)
        }));
        let given: Vec<_> = args
            .iter()
            .enumerate()
            .filter(|(_, arg)| *arg == "--plugin-dir")
            .map(|(at, _)| args[at + 1].replace('\\', "/"))
            .collect();
        assert_eq!(
            given,
            ["w/.claude/plugins/data", "w/.claude/plugins/vercel"]
        );
    }

    #[test]
    fn a_resumed_bot_with_a_model_and_effort_passes_them() {
        let args = strings(ClaudeAgent.args(&plan(true, BotModel::Sonnet, BotEffort::High)));
        assert!(args.contains(&"--resume".to_owned()));
        let at = args.iter().position(|a| a == "--model").expect("model");
        assert_eq!(args[at + 1], "sonnet");
        let at = args.iter().position(|a| a == "--effort").expect("effort");
        assert_eq!(args[at + 1], "high");
    }

    #[test]
    fn a_turn_is_one_user_line_with_its_images() {
        let turn = Turn {
            text: "hi".into(),
            images: vec![TurnImage {
                media_type: "image/png".into(),
                data: "AAAA".into(),
            }],
        };
        let line = ClaudeAgent.encode_turn("u-1", &turn);
        assert!(line.ends_with(b"\n"));
        let json: serde_json::Value = serde_json::from_slice(&line).expect("json");
        assert_eq!(json["type"], "user");
        assert_eq!(json["uuid"], "u-1");
        assert_eq!(json["message"]["content"][0]["text"], "hi");
        assert_eq!(json["message"]["content"][1]["source"]["data"], "AAAA");
    }
}
