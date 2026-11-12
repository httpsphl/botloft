//! Helpers for the tests of several conversations per bot (spec 40).

use std::time::Duration;

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::BotState;
use botloftd::runtime::fake::FakeProcess;
use serde_json::{Value, json};

use super::{Client, TestDaemon, stream};

/// The `n`th process the runtime started, waiting up to 5 s for it.
pub async fn nth_process(t: &TestDaemon, n: usize) -> FakeProcess {
    for _ in 0..500 {
        if let Some(process) = t.runtime.processes().into_iter().nth(n) {
            return process;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("process {n} never started");
}

/// A sleeping bot of a new crew, and its id.
pub async fn sleeping_bot(t: &TestDaemon, app: &mut Client) -> (Value, BotId) {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" });
    let bot = app.call("bots.create", bot).await.expect("bot");
    t.until_state(&bot, BotState::Sleeping).await;
    let id = bot["id"].as_str().expect("id").parse().expect("bot id");
    (bot, id)
}

/// Another conversation of `bot`, opened as the owner opens it.
pub async fn open(app: &mut Client, bot: &BotId) -> ConversationId {
    let made = app
        .call("conversations.create", json!({ "botId": bot }))
        .await
        .expect("conversation");
    made["id"]
        .as_str()
        .expect("id")
        .parse()
        .expect("conversation id")
}

pub async fn send(
    app: &mut Client,
    bot: &Value,
    body: &str,
    conversation: Option<&ConversationId>,
) {
    let mut params = json!({ "botId": bot["id"], "body": body });
    if let Some(conversation) = conversation {
        params["conversationId"] = json!(conversation.as_str());
    }
    app.call("messages.send", params).await.expect("send");
}

/// The process whose first message is `body`, among the first `count`.
pub async fn holding(t: &TestDaemon, count: usize, body: &str) -> (FakeProcess, Value) {
    for n in 0..count {
        let process = nth_process(t, n).await;
        let line = process.wait_lines(1).await.remove(0);
        if line.to_string().contains(body) {
            return (process, line);
        }
    }
    panic!("no process got {body}");
}

/// The process takes up `line` in `session`, answers and ends the turn.
pub async fn turn(process: &FakeProcess, line: &Value, session: &str, reply: &str) {
    begin(process, line, session).await;
    process.emit(stream::text(reply)).await;
    process.emit(stream::result(false)).await;
}

/// The process takes up `line` in `session` and keeps working.
pub async fn begin(process: &FakeProcess, line: &Value, session: &str) {
    let mut opened = stream::replay(line);
    opened["session_id"] = json!(session);
    process.emit(stream::init(session)).await;
    process.emit(opened).await;
}

pub fn resumes(process: &FakeProcess) -> Option<String> {
    let args = process.args();
    let at = args.iter().position(|arg| arg == "--resume")?;
    args.get(at + 1).cloned()
}

/// The replies in one conversation's chat, newest first.
pub async fn replies(
    app: &mut Client,
    bot: &Value,
    conversation: Option<&ConversationId>,
) -> Vec<String> {
    let mut params = json!({ "botId": bot["id"] });
    if let Some(conversation) = conversation {
        params["conversationId"] = json!(conversation.as_str());
    }
    let history = app.call("chat.history", params).await.expect("history");
    history
        .as_array()
        .expect("items")
        .iter()
        .filter(|item| item["body"]["kind"] == "reply")
        .filter_map(|item| item["body"]["text"].as_str().map(str::to_owned))
        .collect()
}

/// Waits until `text` is a reply in that conversation's chat.
pub async fn until_reply(
    app: &mut Client,
    bot: &Value,
    conversation: Option<&ConversationId>,
    text: &str,
) {
    for _ in 0..200 {
        if replies(app, bot, conversation)
            .await
            .iter()
            .any(|each| each == text)
        {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("no reply {text}");
}
