//! A bot that runs on Codex (spec 30), on a FakeRuntime: the process speaks
//! `codex app-server`'s JSON-RPC, and the handshake, the turn, the chat, the
//! context and the answers to its requests come out as for the other agents.

mod common;

use std::sync::Arc;
use std::time::Duration;

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    AgentKind, BotState, BotsCreateParams, ChatBody, CrewsCreateParams, ToolStatus,
};
use botloftd::agent;
use botloftd::runtime::fake::{FakeProcess, FakeRuntime};
use botloftd::service::{bots, crews};
use botloftd::state::Daemon;
use botloftd::supervisor;
use common::{new_daemon, test_settings};
use serde_json::{Value, json};

struct Codex {
    daemon: Arc<Daemon>,
    runtime: FakeRuntime,
    bot: BotId,
    _dir: tempfile::TempDir,
}

async fn codex_bot() -> Codex {
    let mut settings = test_settings();
    settings.experimental_agents = vec!["codex".to_owned()];
    settings.codex_path = r"C:\Codex\codex.exe".to_owned();
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
            agent: Some(AgentKind::Codex),
        },
    )
    .expect("codex bot");
    assert_eq!(bot.agent, AgentKind::Codex);
    tokio::spawn(supervisor::run(Arc::clone(&parts.daemon)));
    Codex {
        daemon: parts.daemon,
        runtime: parts.runtime,
        bot: bot.id,
        _dir: parts.dir,
    }
}

impl Codex {
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

    /// The owner's message, as the courier writes it.
    fn message(&self, uuid: &str, text: &str) {
        let line = agent::of(AgentKind::Codex).expect("agent").encode_turn(
            uuid,
            &agent::Turn {
                text: text.into(),
                images: Vec::new(),
            },
        );
        self.daemon
            .supervisor
            .write_message(&self.bot, uuid, line)
            .expect("running");
    }
}

/// The methods the process was sent, in order.
fn methods(process: &FakeProcess) -> Vec<String> {
    process
        .input_lines()
        .iter()
        .map(|line| line["method"].as_str().unwrap_or("(answer)").to_owned())
        .collect()
}

/// Waits for the process to have been sent `method`, and returns that line.
async fn sent(process: &FakeProcess, method: &str) -> Value {
    for _ in 0..600 {
        if let Some(line) = process
            .input_lines()
            .into_iter()
            .find(|line| line["method"] == method)
        {
            return line;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("never sent {method}: {:?}", methods(process));
}

/// The handshake: `initialize`, then the thread.
async fn handshake(process: &FakeProcess, thread: &str) {
    let init = sent(process, "initialize").await;
    process
        .emit(json!({ "id": init["id"], "result": { "userAgent": "codex" } }))
        .await;
    let start = sent(process, "thread/start").await;
    process
        .emit(json!({ "id": start["id"], "result": { "thread": { "id": thread }, "model": "gpt-5.5" } }))
        .await;
}

fn notify(method: &str, params: Value) -> Value {
    json!({ "method": method, "params": params })
}

#[tokio::test(start_paused = true)]
async fn a_codex_bot_starts_its_server_with_the_owners_things_off_and_its_own_mcp() {
    let codex = codex_bot().await;
    let process = codex.runtime.process(1).await;
    assert_eq!(
        process.spec.program.to_string_lossy(),
        r"C:\Codex\codex.exe"
    );
    let args = process.args();
    assert_eq!(args[0], "app-server");
    for flag in [
        "features.apps=false",
        "features.hooks=false",
        "features.plugins=false",
        "notify=[]",
    ] {
        assert!(args.contains(&flag.to_owned()), "{flag} in {args:?}");
    }
    // It answers Botloft's lines in JSON-RPC, not the control requests.
    assert!(process.control_requests().is_empty());

    handshake(&process, "th-1").await;
    let start = sent(&process, "thread/start").await;
    assert_eq!(start["params"]["approvalPolicy"], "on-request");
    assert_eq!(start["params"]["sandbox"], "read-only");
    let server = &start["params"]["config"]["mcp_servers"]["botloft"];
    assert_eq!(server["url"], "http://127.0.0.1:45710/mcp");
    assert!(
        server["http_headers"]["Authorization"]
            .as_str()
            .is_some_and(|value| value.starts_with("Bearer "))
    );
    let workspace = process.spec.cwd.clone();
    let rules = std::fs::read_to_string(workspace.join("AGENTS.md")).expect("AGENTS.md");
    assert!(rules.contains("Scout") && rules.contains("Keep notes."));
}

#[tokio::test(start_paused = true)]
async fn what_codex_prints_becomes_the_chat() {
    let codex = codex_bot().await;
    let process = codex.runtime.process(1).await;
    handshake(&process, "th-1").await;
    codex.until(BotState::Idle).await;

    codex.message("u-1", "List the folder");
    codex.until(BotState::Busy).await;
    let turn = sent(&process, "turn/start").await;
    assert_eq!(turn["params"]["threadId"], "th-1");
    assert_eq!(turn["params"]["input"][0]["text"], "List the folder");
    process
        .emit(json!({ "id": turn["id"], "result": { "turn": { "id": "t-1", "status": "inProgress" } } }))
        .await;

    let t = "th-1";
    process
        .emit(notify(
            "turn/started",
            json!({ "threadId": t, "turn": { "id": "t-1" } }),
        ))
        .await;
    process
        .emit(notify(
            "item/started",
            json!({ "threadId": t, "item": { "type": "userMessage", "id": "i0" } }),
        ))
        .await;
    process
        .emit(notify(
            "item/started",
            json!({ "threadId": t, "item": {
            "type": "commandExecution", "id": "c1", "command": "powershell.exe -Command 'ls'",
            "commandActions": [{ "type": "unknown", "command": "ls" }], "status": "inProgress" } }),
        ))
        .await;
    process
        .emit(notify(
            "item/completed",
            json!({ "threadId": t, "item": {
            "type": "commandExecution", "id": "c1", "status": "completed", "exitCode": 0,
            "aggregatedOutput": "a.txt\n" } }),
        ))
        .await;
    process
        .emit(notify(
            "item/started",
            json!({ "threadId": t, "item": {
            "type": "mcpToolCall", "id": "m1", "server": "botloft", "tool": "send_message",
            "arguments": { "to": "writer" }, "status": "inProgress" } }),
        ))
        .await;
    process
        .emit(notify(
            "item/completed",
            json!({ "threadId": t, "item": {
            "type": "mcpToolCall", "id": "m1", "status": "completed",
            "result": { "content": [{ "type": "text", "text": "sent" }] } } }),
        ))
        .await;
    process
        .emit(notify(
            "item/agentMessage/delta",
            json!({ "threadId": t, "delta": "One " }),
        ))
        .await;
    process
        .emit(notify("item/completed", json!({ "threadId": t, "item": {
            "type": "agentMessage", "id": "a1", "text": "One file: a.txt", "phase": "final_answer" } })))
        .await;
    process
        .emit(notify("thread/tokenUsage/updated", json!({ "threadId": t, "tokenUsage": {
            "total": { "totalTokens": 1100, "inputTokens": 1000, "cachedInputTokens": 400, "outputTokens": 80, "reasoningOutputTokens": 20 },
            "last": { "totalTokens": 1100, "inputTokens": 1000, "cachedInputTokens": 400, "outputTokens": 80, "reasoningOutputTokens": 20 },
            "modelContextWindow": 258400 } })))
        .await;
    process
        .emit(notify(
            "turn/completed",
            json!({ "threadId": t, "turn": {
            "id": "t-1", "status": "completed", "error": null, "durationMs": 2500 } }),
        ))
        .await;
    codex.until(BotState::Idle).await;

    let items = codex.items();
    let replies: Vec<&str> = items
        .iter()
        .filter_map(|body| match body {
            ChatBody::Reply(reply) => Some(reply.text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(replies, ["One file: a.txt"]);
    let tools: Vec<(&str, ToolStatus)> = items
        .iter()
        .filter_map(|body| match body {
            ChatBody::Tool(tool) => Some((tool.name.as_str(), tool.status)),
            _ => None,
        })
        .collect();
    assert_eq!(
        tools,
        [
            ("Bash", ToolStatus::Done),
            ("mcp__botloft__send_message", ToolStatus::Done)
        ]
    );
    let turn_item = items
        .iter()
        .find_map(|body| match body {
            ChatBody::Turn(turn) => Some(turn),
            _ => None,
        })
        .expect("a turn");
    assert_eq!(turn_item.duration_ms, 2500);
    assert_eq!(turn_item.error, None);
    let tokens = turn_item.tokens.as_ref().expect("tokens");
    assert_eq!(
        (tokens.input, tokens.cache_read, tokens.output),
        (600, 400, 100)
    );

    // The context is the last request's, with the model's own window.
    let context = codex.daemon.contexts.get(&codex.bot).expect("context");
    assert_eq!(
        (context.used_tokens, context.window_tokens),
        (1080, 258_400)
    );
    // The conversation to resume, and the model it ran on.
    assert_eq!(
        codex
            .daemon
            .store()
            .session_id(&codex.bot)
            .expect("read")
            .as_deref(),
        Some("th-1")
    );
    let bot = codex
        .daemon
        .store()
        .bot(&codex.bot)
        .expect("read")
        .expect("bot");
    assert_eq!(bot.model_in_use.as_deref(), Some("gpt-5.5"));
}

/// The approval card waiting in the chat, once it is there.
async fn card(codex: &Codex) -> botloft_core::protocol::ApprovalItem {
    for _ in 0..600 {
        let found = codex.items().into_iter().find_map(|body| match body {
            ChatBody::Approval(item)
                if item.status == botloft_core::protocol::ApprovalStatus::Pending =>
            {
                Some(item)
            }
            _ => None,
        });
        if let Some(item) = found {
            return item;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("no approval card");
}

fn answer_of(process: &FakeProcess, id: u64) -> Option<Value> {
    process
        .input_lines()
        .into_iter()
        .find(|line| line["id"] == id)
}

async fn answer_for(process: &FakeProcess, id: u64) -> Value {
    for _ in 0..600 {
        if let Some(line) = answer_of(process, id) {
            return line;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("no answer to {id}");
}

fn file_started(
    process: &FakeProcess,
    id: &str,
    path: &str,
) -> impl std::future::Future<Output = ()> {
    process.emit(notify(
        "item/started",
        json!({ "threadId": "th-1", "item": { "type": "fileChange", "id": id,
            "changes": [{ "path": path, "kind": { "type": "add" }, "diff": "x\n" }], "status": "inProgress" } }),
    ))
}

fn owner(codex: &Codex, approval: &botloft_core::protocol::ApprovalItem, allow: bool) {
    botloftd::approvals::answer(
        &codex.daemon,
        botloft_core::protocol::ApprovalsAnswerParams {
            approval_id: approval.approval_id.clone(),
            allow,
            note: None,
            input: None,
            always: None,
        },
    )
    .expect("answered");
}

#[tokio::test(start_paused = true)]
async fn a_change_waits_for_the_owner_who_allows_or_denies_it() {
    let codex = codex_bot().await;
    let process = codex.runtime.process(1).await;
    handshake(&process, "th-1").await;
    codex.until(BotState::Idle).await;

    file_started(&process, "f1", r"C:\work\a.txt").await;
    process
        .emit(json!({ "id": 90, "method": "item/fileChange/requestApproval", "params": { "itemId": "f1" } }))
        .await;
    let shown = card(&codex).await;
    assert_eq!(shown.tool_name, "Write");
    assert!(shown.input.contains("a.txt"));
    // Nothing is answered while the owner has not.
    assert!(answer_of(&process, 90).is_none());
    owner(&codex, &shown, true);
    assert_eq!(
        answer_for(&process, 90).await["result"]["decision"],
        "accept"
    );

    file_started(&process, "f2", r"C:\work\b.txt").await;
    process
        .emit(json!({ "id": 91, "method": "item/fileChange/requestApproval", "params": { "itemId": "f2" } }))
        .await;
    let second = loop {
        let shown = card(&codex).await;
        if shown.input.contains("b.txt") {
            break shown;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    owner(&codex, &second, false);
    assert_eq!(
        answer_for(&process, 91).await["result"]["decision"],
        "decline"
    );
}

#[tokio::test(start_paused = true)]
async fn a_command_asks_and_the_crew_tools_do_not() {
    let codex = codex_bot().await;
    let process = codex.runtime.process(1).await;
    handshake(&process, "th-1").await;
    codex.until(BotState::Idle).await;

    process
        .emit(notify("item/started", json!({ "threadId": "th-1", "item": {
            "type": "commandExecution", "id": "c1", "command": "powershell -Command 'rm x'",
            "commandActions": [{ "type": "unknown", "command": "rm x" }], "status": "inProgress" } })))
        .await;
    process
        .emit(json!({ "id": 80, "method": "item/commandExecution/requestApproval", "params": { "itemId": "c1", "command": "rm x" } }))
        .await;
    let shown = card(&codex).await;
    assert_eq!(shown.tool_name, "Bash");
    assert!(shown.input.contains("rm x"));
    owner(&codex, &shown, true);
    assert_eq!(
        answer_for(&process, 80).await["result"]["decision"],
        "accept"
    );

    // Botloft's own tools go through; another server's tool asks.
    process
        .emit(json!({ "id": 81, "method": "mcpServer/elicitation/request", "params": { "serverName": "botloft", "threadId": "th-1" } }))
        .await;
    assert_eq!(answer_for(&process, 81).await["result"]["action"], "accept");
    // What the server asks that Botloft does not take gets an error.
    process
        .emit(json!({ "id": 82, "method": "item/tool/call", "params": {} }))
        .await;
    assert!(answer_for(&process, 82).await["error"].is_object());
}

#[tokio::test(start_paused = true)]
async fn a_folder_botloft_fences_is_refused_without_asking() {
    let codex = codex_bot().await;
    // Another crew's folder.
    let other = crews::create(
        &codex.daemon,
        CrewsCreateParams {
            name: "Other".into(),
            work_folder: None,
            lead: None,
        },
    )
    .expect("crew");
    let theirs = codex.daemon.paths.crew_dir(&other.slug).join("notes.txt");
    let process = codex.runtime.process(1).await;
    handshake(&process, "th-1").await;
    codex.until(BotState::Idle).await;

    file_started(&process, "f9", &theirs.to_string_lossy()).await;
    process
        .emit(json!({ "id": 70, "method": "item/fileChange/requestApproval", "params": { "itemId": "f9" } }))
        .await;
    assert_eq!(
        answer_for(&process, 70).await["result"]["decision"],
        "decline"
    );

    // A command that names it too.
    process
        .emit(
            json!({ "id": 71, "method": "item/commandExecution/requestApproval", "params": {
            "itemId": "c9", "command": format!("type {}", theirs.to_string_lossy()) } }),
        )
        .await;
    assert_eq!(
        answer_for(&process, 71).await["result"]["decision"],
        "decline"
    );
    // No card was made for either.
    assert!(
        !codex
            .items()
            .iter()
            .any(|body| matches!(body, ChatBody::Approval(_)))
    );
}

#[tokio::test(start_paused = true)]
async fn the_bots_mode_decides_what_is_not_asked() {
    use botloft_core::protocol::{BotsSetPermissionModeParams, PermissionMode};
    let codex = codex_bot().await;
    botloftd::service::modes::set_permission_mode(
        &codex.daemon,
        BotsSetPermissionModeParams {
            bot_id: codex.bot.clone(),
            mode: PermissionMode::AcceptEdits,
        },
    )
    .expect("mode");
    let process = codex.runtime.process(1).await;
    handshake(&process, "th-1").await;
    codex.until(BotState::Idle).await;

    file_started(&process, "f1", r"C:\work\a.txt").await;
    process
        .emit(json!({ "id": 60, "method": "item/fileChange/requestApproval", "params": { "itemId": "f1" } }))
        .await;
    assert_eq!(
        answer_for(&process, 60).await["result"]["decision"],
        "accept"
    );
    // A command still asks.
    process
        .emit(json!({ "id": 61, "method": "item/commandExecution/requestApproval", "params": { "itemId": "c1", "command": "rm x" } }))
        .await;
    let _ = card(&codex).await;
    assert!(answer_of(&process, 61).is_none());
}

#[tokio::test(start_paused = true)]
async fn the_next_start_resumes_the_thread_and_a_lost_one_starts_again() {
    let codex = codex_bot().await;
    let first = codex.runtime.process(1).await;
    handshake(&first, "th-7").await;
    codex.until(BotState::Idle).await;
    codex.message("u-1", "hello");
    let turn = sent(&first, "turn/start").await;
    first.emit(json!({ "id": turn["id"], "result": {} })).await;
    first
        .emit(notify(
            "item/started",
            json!({ "threadId": "th-7", "item": { "type": "userMessage", "id": "i" } }),
        ))
        .await;
    first
        .emit(notify("turn/completed", json!({ "threadId": "th-7", "turn": { "id": "t", "status": "completed", "durationMs": 10 } })))
        .await;
    codex.until(BotState::Idle).await;

    first.exit(1).await;
    let second = codex.runtime.process(2).await;
    let init = sent(&second, "initialize").await;
    second.emit(json!({ "id": init["id"], "result": {} })).await;
    let resume = sent(&second, "thread/resume").await;
    assert_eq!(resume["params"]["threadId"], "th-7");
    // Codex does not have it: a new thread takes its place.
    second
        .emit(json!({ "id": resume["id"], "error": { "code": -1, "message": "no rollout found" } }))
        .await;
    sent(&second, "thread/start").await;
}
