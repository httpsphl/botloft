//! A bot that runs on Antigravity's `agy` (spec 30), on a FakeRuntime: its
//! process prints `agy`'s stream-json, and the chat, the read receipt, the
//! conversation to resume and the files `agy` reads come out as for Claude.

mod common;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    AgentKind, BotState, BotsCreateParams, ChatBody, CrewsCreateParams, ToolStatus,
};
use botloftd::runtime::fake::FakeProcess;
use botloftd::service::{bots, crews};
use botloftd::state::Daemon;
use botloftd::supervisor;
use common::{new_daemon, test_settings};
use serde_json::{Value, json};

struct Agy {
    daemon: Arc<Daemon>,
    runtime: botloftd::runtime::fake::FakeRuntime,
    bot: BotId,
    paths: botloftd::paths::Paths,
    _dir: tempfile::TempDir,
}

async fn agy_bot() -> Agy {
    let mut settings = test_settings();
    settings.experimental_agents = vec!["agy".to_owned()];
    settings.agy_path = r"C:\Agy\agy.exe".to_owned();
    let parts = new_daemon(settings);
    let crew = crews::create(
        &parts.daemon,
        CrewsCreateParams {
            name: "Ops".into(),
            work_folder: None,
            lead: None,
        },
    )
    .expect("crew");
    let bot = bots::create(
        &parts.daemon,
        BotsCreateParams {
            crew_id: crew.id,
            name: "Scout".into(),
            role: String::new(),
            instructions: "Keep notes.".into(),
            color: None,
            model: None,
            agent: Some(AgentKind::Agy),
        },
    )
    .expect("agy bot");
    assert_eq!(bot.agent, AgentKind::Agy);
    tokio::spawn(supervisor::run(Arc::clone(&parts.daemon)));
    Agy {
        daemon: parts.daemon,
        runtime: parts.runtime,
        bot: bot.id,
        paths: parts.paths,
        _dir: parts.dir,
    }
}

impl Agy {
    async fn until(&self, state: BotState) {
        for _ in 0..600 {
            if self.daemon.supervisor.status(&self.bot).map(|(s, _)| s) == Some(state) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("bot never reached {state:?}");
    }

    fn items(&self) -> Vec<ChatBody> {
        let mut items = self
            .daemon
            .store()
            .chat_history(&self.bot, None, 50)
            .expect("history");
        items.reverse();
        items.into_iter().map(|item| item.body).collect()
    }
}

fn step(conversation: &str, index: u32, kind: &str, state: &str, extra: Value) -> Value {
    let mut update = json!({
        "conversation_id": conversation,
        "step_index": index,
        "step_type": kind,
        "state": state,
    });
    for (key, value) in extra.as_object().expect("object") {
        update[key] = value.clone();
    }
    json!({ "event": "step_update", "step_update": update })
}

#[tokio::test(start_paused = true)]
async fn an_agy_bot_starts_without_a_session_and_with_its_own_home() {
    let agy = agy_bot().await;
    let process = agy.runtime.process(1).await;
    let args = process.args();
    assert!(args.contains(&"stream-json".to_owned()));
    assert!(!args.contains(&"--conversation".to_owned()), "{args:?}");
    assert_eq!(args.last().map(String::as_str), Some("-p="));
    // `agy` does not answer Botloft's own requests, and refuses their format.
    assert!(process.control_requests().is_empty());
    assert!(process.input().is_empty());
    assert_eq!(process.spec.program.to_string_lossy(), r"C:\Agy\agy.exe");

    // Its home, where `agy` reads its own settings and MCP servers.
    let workspace = process.spec.cwd.clone();
    let home = workspace.join(".botloft").join("agy-home");
    assert_eq!(process.env("USERPROFILE").as_deref(), home.to_str());
    let mcp: Value = serde_json::from_str(
        &std::fs::read_to_string(home.join(".gemini/config/mcp_config.json")).expect("mcp"),
    )
    .expect("json");
    assert_eq!(
        mcp["mcpServers"]["botloft"]["serverUrl"],
        "http://127.0.0.1:45710/mcp"
    );
    assert!(
        mcp["mcpServers"]["botloft"]["headers"]["Authorization"]
            .as_str()
            .is_some_and(|value| value.starts_with("Bearer "))
    );
    let settings: Value = serde_json::from_str(
        &std::fs::read_to_string(home.join(".gemini/antigravity-cli/settings.json"))
            .expect("settings"),
    )
    .expect("json");
    assert_eq!(settings["permissions"]["allow"][0], "mcp(botloft/*)");
    let secrets = agy.paths.secrets().to_string_lossy().replace('\\', "/");
    let deny = settings["permissions"]["deny"].as_array().expect("deny");
    assert!(deny.contains(&json!(format!("write_file({secrets})"))));

    // The rules the bot reads, in the file `agy` loads from its folder.
    let rules = std::fs::read_to_string(workspace.join("AGENTS.md")).expect("AGENTS.md");
    assert!(
        rules.contains("Scout") && rules.contains("Keep notes."),
        "{rules}"
    );
}

#[tokio::test(start_paused = true)]
async fn what_agy_prints_becomes_the_chat() {
    let agy = agy_bot().await;
    let process: FakeProcess = agy.runtime.process(1).await;
    agy.until(BotState::Idle).await;

    // The courier writes a message, tracked by its uuid until `agy` takes it.
    agy.daemon
        .supervisor
        .write_message(&agy.bot, "u-1", "{}\n".into())
        .expect("running");
    agy.until(BotState::Busy).await;

    let conv = "c-1";
    process
        .emit(json!({ "event": "init", "init": { "conversation_id": conv, "model": "gemini-3.8-flash-low" } }))
        .await;
    process
        .emit(step(conv, 0, "user_input", "DONE", json!({})))
        .await;
    process
        .emit(step(
            conv,
            1,
            "agent_response",
            "ACTIVE",
            json!({ "text_delta": "On it." }),
        ))
        .await;
    process
        .emit(step(
            conv,
            1,
            "agent_response",
            "DONE",
            json!({ "text_delta": "\n" }),
        ))
        .await;
    process
        .emit(step(
            conv,
            2,
            "tool",
            "ACTIVE",
            json!({ "tool_name": "run_command", "tool_info": { "parameters": { "CommandLine": "echo hi" } } }),
        ))
        .await;
    process
        .emit(step(
            conv,
            2,
            "tool",
            "ERROR",
            json!({ "tool_name": "run_command", "tool_info": { "error": { "message": "denied" } } }),
        ))
        .await;
    process
        .emit(step(
            conv,
            3,
            "agent_response",
            "ACTIVE",
            json!({ "text_delta": "Done." }),
        ))
        .await;
    process
        .emit(step(
            conv,
            3,
            "agent_response",
            "DONE",
            json!({ "text_delta": "\n" }),
        ))
        .await;
    process
        .emit(json!({ "event": "result", "result": {
            "conversation_id": conv, "status": "SUCCESS", "response": "Done.\n",
            "duration_seconds": 2.5, "num_turns": 1,
            "usage": { "input_tokens": 100, "output_tokens": 7, "thinking_tokens": 3, "cache_read_tokens": 40, "total_tokens": 150 }
        } }))
        .await;
    agy.until(BotState::Idle).await;

    let items = agy.items();
    let replies: Vec<&str> = items
        .iter()
        .filter_map(|body| match body {
            ChatBody::Reply(reply) => Some(reply.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(replies, ["On it.", "Done."]);
    let tool = items
        .iter()
        .find_map(|body| match body {
            ChatBody::Tool(tool) => Some(tool),
            _ => None,
        })
        .expect("a tool");
    assert_eq!(tool.name, "Bash");
    assert_eq!(tool.status, ToolStatus::Failed);
    assert_eq!(tool.output.as_deref(), Some("denied"));
    let turn = items
        .iter()
        .find_map(|body| match body {
            ChatBody::Turn(turn) => Some(turn),
            _ => None,
        })
        .expect("a turn");
    assert_eq!(turn.duration_ms, 2500);
    assert_eq!(turn.error, None);
    let tokens = turn.tokens.as_ref().expect("tokens");
    assert_eq!(
        (tokens.input, tokens.cache_read, tokens.output),
        (100, 40, 10)
    );

    // `agy` has the conversation from its first message on: the next start
    // resumes it.
    assert_eq!(
        agy.daemon
            .store()
            .session_id(&agy.bot)
            .expect("read")
            .as_deref(),
        Some(conv)
    );
    let bot = agy
        .daemon
        .store()
        .bot(&agy.bot)
        .expect("read")
        .expect("bot");
    assert_eq!(bot.model_in_use.as_deref(), Some("gemini-3.8-flash-low"));
}

#[tokio::test(start_paused = true)]
async fn a_failed_turn_says_why() {
    let agy = agy_bot().await;
    let process = agy.runtime.process(1).await;
    agy.until(BotState::Idle).await;
    agy.daemon
        .supervisor
        .write_message(&agy.bot, "u-1", "{}\n".into())
        .expect("running");
    process
        .emit(step("c-2", 0, "user_input", "DONE", json!({})))
        .await;
    process
        .emit(json!({ "event": "result", "result": {
            "conversation_id": "c-2", "status": "ERROR", "response": "", "error": "model overloaded",
            "duration_seconds": 1.0, "num_turns": 0, "usage": {}
        } }))
        .await;
    agy.until(BotState::Idle).await;
    let failed = agy.items().into_iter().find_map(|body| match body {
        ChatBody::Turn(turn) => turn.error,
        _ => None,
    });
    assert_eq!(failed.as_deref(), Some("model overloaded"));
}

#[tokio::test(start_paused = true)]
async fn the_next_start_resumes_the_conversation() {
    let agy = agy_bot().await;
    let first = agy.runtime.process(1).await;
    agy.until(BotState::Idle).await;
    agy.daemon
        .supervisor
        .write_message(&agy.bot, "u-1", "{}\n".into())
        .expect("running");
    first
        .emit(step("c-3", 0, "user_input", "DONE", json!({})))
        .await;
    first
        .emit(json!({ "event": "result", "result": { "status": "SUCCESS", "duration_seconds": 0.1, "usage": {} } }))
        .await;
    agy.until(BotState::Idle).await;

    first.exit(1).await;
    let second = agy.runtime.process(2).await;
    let args = second.args();
    let at = args
        .iter()
        .position(|a| a == "--conversation")
        .expect("resumed");
    assert_eq!(args[at + 1], "c-3");
}
