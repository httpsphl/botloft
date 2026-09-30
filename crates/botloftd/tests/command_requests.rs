//! Requests to run a command (spec 10.1): what the bot says the command is
//! for goes to the owner next to the command, never in its place, and the
//! command is kept whole.

mod common;

use common::bots::{Crew, two_bots};
use common::mcp::Mcp;
use common::stream;
use serde_json::{Value, json};

/// The next `chat.item` of `kind`.
async fn kind(app: &mut common::Client, kind: &str) -> Value {
    loop {
        let changed = app.notification("chat.item").await;
        if changed["item"]["body"]["kind"] == kind {
            return changed;
        }
    }
}

/// A crew whose @lead is in a turn, ready to use tools.
async fn working() -> Crew {
    let mut c = two_bots().await;
    let lead = c.app.call("bots.list", json!({})).await.expect("bots")[0].clone();
    c.app
        .call(
            "messages.send",
            json!({ "botId": lead["id"], "body": "Go" }),
        )
        .await
        .expect("send");
    let lines = c.lead_process.wait_lines(1).await;
    let line = lines.last().expect("line");
    c.lead_process.emit(stream::replay(line)).await;
    c
}

/// @lead calls `tool` with `input` and Claude Code asks for permission; the
/// decision comes out of the handle.
async fn ask(c: &Crew, id: &str, tool: &str, input: &Value) -> tokio::task::JoinHandle<Value> {
    c.lead_process
        .emit(stream::tool_use(id, tool, input.clone()))
        .await;
    let mut mcp = Mcp::new(c.lead.addr, c.lead.token.clone());
    let args = json!({ "tool_name": tool, "input": input, "tool_use_id": id });
    tokio::spawn(async move { mcp.tool("permission_prompt", args).await.expect("decision") })
}

#[tokio::test]
async fn the_bots_explanation_goes_with_the_command_and_what_runs_is_the_command() {
    let mut c = working().await;
    let input = json!({
        "command": "pip install requests",
        "description": "Instala o pacote requests, para o script acessar páginas da internet",
    });
    let decision = ask(&c, "toolu_1", "Bash", &input).await;

    let tool = kind(&mut c.app, "tool").await;
    assert_eq!(tool["item"]["body"]["summary"], "pip install requests");
    assert_eq!(tool["item"]["body"]["explanation"], input["description"]);
    let pending = kind(&mut c.app, "approval").await;
    let body = &pending["item"]["body"];
    assert_eq!(body["summary"], "pip install requests");
    assert_eq!(body["explanation"], input["description"]);
    assert_eq!(pending["activity"]["text"], input["description"]);

    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": body["approvalId"], "allow": true }),
        )
        .await
        .expect("answer");
    // Claude Code gets back the input it asked about, untouched.
    let outcome = decision.await.expect("task");
    assert_eq!(
        outcome,
        json!({ "behavior": "allow", "updatedInput": input })
    );
    // The answered request still says what the bot said.
    let answered = kind(&mut c.app, "approval").await;
    assert_eq!(answered["item"]["body"]["status"], "allowed");
    assert_eq!(
        answered["item"]["body"]["explanation"],
        input["description"]
    );
}

#[tokio::test]
async fn a_command_without_an_explanation_has_none() {
    let mut c = working().await;
    let decision = ask(
        &c,
        "toolu_2",
        "PowerShell",
        &json!({ "command": "Get-Date" }),
    )
    .await;
    let pending = kind(&mut c.app, "approval").await;
    let body = &pending["item"]["body"];
    assert_eq!(body["summary"], "Get-Date");
    assert_eq!(body["explanation"], Value::Null);
    assert_eq!(pending["activity"]["text"], "Get-Date");
    c.app
        .call(
            "approvals.answer",
            json!({ "approvalId": body["approvalId"], "allow": false }),
        )
        .await
        .expect("answer");
    assert_eq!(decision.await.expect("task")["behavior"], "deny");
}

#[tokio::test]
async fn a_long_command_reaches_the_owner_whole() {
    let mut c = working().await;
    // Longer than other tools' input is kept (4 KB), as a script in a
    // heredoc is.
    let script = format!("python - <<'EOF'\n{}EOF", "print('hello')\n".repeat(600));
    let input = json!({ "command": script, "description": "Gera a planilha" });
    let decision = ask(&c, "toolu_3", "Bash", &input).await;
    for shown in ["tool", "approval"] {
        let item = kind(&mut c.app, shown).await["item"]["body"].clone();
        let kept: Value = serde_json::from_str(item["input"].as_str().expect("input"))
            .unwrap_or_else(|_| panic!("the {shown} item has the whole input"));
        assert_eq!(kept["command"], script);
    }
    c.lead_process.exit(1).await;
    decision.await.expect("task");
}

#[tokio::test]
async fn a_command_too_long_to_keep_is_cut_and_still_explained() {
    let mut c = working().await;
    let script = "echo hello\n".repeat(4000);
    let input = json!({ "command": script, "description": "Gera a planilha" });
    let decision = ask(&c, "toolu_4", "Bash", &input).await;
    let pending = kind(&mut c.app, "approval").await;
    let body = &pending["item"]["body"];
    let kept = body["input"].as_str().expect("input");
    assert!(kept.len() <= 32 * 1024 && kept.ends_with('…'));
    assert_eq!(body["explanation"], "Gera a planilha");
    c.lead_process.exit(1).await;
    decision.await.expect("task");
}
