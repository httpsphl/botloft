//! Several conversations of one bot (spec 40): each its own process, with
//! its own session, running at the same time, and at most three awake.

mod common;

use std::time::Duration;

use botloft_core::ids::{BotId, ConversationId};
use botloft_core::protocol::BotState;
use botloft_store::ConversationRecord;
use botloftd::runtime::fake::FakeProcess;
use common::{Client, TestDaemon, stream};
use serde_json::{Value, json};

const NAP: Duration = Duration::from_millis(300);

/// The `n`th process the runtime started, waiting up to 5 s for it.
async fn nth_process(t: &TestDaemon, n: usize) -> FakeProcess {
    for _ in 0..500 {
        if let Some(process) = t.runtime.processes().into_iter().nth(n) {
            return process;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!("process {n} never started");
}

/// A sleeping bot of a new crew, and its id.
async fn sleeping_bot(t: &TestDaemon, app: &mut Client) -> (Value, BotId) {
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

/// Another conversation of `bot`, as the owner will open it (C2).
fn open(t: &TestDaemon, bot: &BotId) -> ConversationId {
    let conversation = ConversationRecord::new(bot, t.daemon.clock.now_ms());
    t.daemon
        .store()
        .insert_conversation(&conversation)
        .expect("conversation");
    t.daemon.supervisor.wake();
    conversation.id
}

async fn send(app: &mut Client, bot: &Value, body: &str, conversation: Option<&ConversationId>) {
    let mut params = json!({ "botId": bot["id"], "body": body });
    if let Some(conversation) = conversation {
        params["conversationId"] = json!(conversation.as_str());
    }
    app.call("messages.send", params).await.expect("send");
}

/// The process whose first message is `body`, among the first `count`.
async fn holding(t: &TestDaemon, count: usize, body: &str) -> (FakeProcess, Value) {
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
async fn turn(process: &FakeProcess, line: &Value, session: &str, reply: &str) {
    let mut opened = stream::replay(line);
    opened["session_id"] = json!(session);
    process.emit(stream::init(session)).await;
    process.emit(opened).await;
    process.emit(stream::text(reply)).await;
    process.emit(stream::result(false)).await;
}

fn resumes(process: &FakeProcess) -> Option<String> {
    let args = process.args();
    let at = args.iter().position(|arg| arg == "--resume")?;
    args.get(at + 1).cloned()
}

#[tokio::test(flavor = "multi_thread")]
async fn two_conversations_run_at_once_each_with_its_own_session() {
    let t = TestDaemon::start_supervised().await;
    t.daemon.settings.set_sleep_after(Some(NAP));
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let other = open(&t, &id);

    send(&mut app, &bot, "Hello main", None).await;
    send(&mut app, &bot, "Hello other", Some(&other)).await;
    let (main_process, main_line) = holding(&t, 2, "Hello main").await;
    let (other_process, other_line) = holding(&t, 2, "Hello other").await;
    // Both are up together, and neither resumes anything yet.
    assert!(!main_process.killed() && !other_process.killed());
    assert_eq!(resumes(&main_process), None);
    assert_eq!(resumes(&other_process), None);

    turn(&other_process, &other_line, "session-other", "Other here").await;
    turn(&main_process, &main_line, "session-main", "Main here").await;

    // Each reply is in its own conversation.
    let reply_in = async |app: &mut Client, text: &str| {
        for _ in 0..200 {
            let history = app
                .call("chat.history", json!({ "botId": bot["id"] }))
                .await
                .expect("history");
            let found = history
                .as_array()
                .expect("items")
                .iter()
                .find(|item| item["body"]["text"] == text)
                .map(|item| item["conversationId"].clone());
            if let Some(found) = found {
                return found;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("no reply {text}");
    };
    let main = ConversationId::main_of(&id);
    assert_eq!(reply_in(&mut app, "Main here").await, json!(main.as_str()));
    assert_eq!(
        reply_in(&mut app, "Other here").await,
        json!(other.as_str())
    );

    // Each one keeps its session, and resumes it once asleep.
    let store_sessions = || {
        let store = t.daemon.store();
        (
            store.conversation_session(&main).expect("main"),
            store.conversation_session(&other).expect("other"),
        )
    };
    assert_eq!(
        store_sessions(),
        (Some("session-main".into()), Some("session-other".into()))
    );
    t.until_state(&bot, BotState::Sleeping).await;
    for _ in 0..200 {
        if other_process.killed() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(other_process.killed(), "the other conversation sleeps too");
    send(&mut app, &bot, "Again", Some(&other)).await;
    let again = nth_process(&t, 2).await;
    assert_eq!(resumes(&again).as_deref(), Some("session-other"));
    let line = again.wait_lines(1).await.remove(0);
    assert!(line.to_string().contains("Again"), "{line}");
}

#[tokio::test(flavor = "multi_thread")]
async fn at_most_three_conversations_of_a_bot_are_awake() {
    let t = TestDaemon::start_supervised().await;
    t.daemon
        .settings
        .set_sleep_after(Some(Duration::from_secs(600)));
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let others: Vec<ConversationId> = (0..3).map(|_| open(&t, &id)).collect();

    send(&mut app, &bot, "To main", None).await;
    for (n, other) in others.iter().enumerate() {
        send(&mut app, &bot, &format!("To other {n}"), Some(other)).await;
    }
    // Three start; the fourth waits while all three work.
    let _ = nth_process(&t, 2).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(t.runtime.processes().len(), 3);

    // One finishes: it goes to sleep to make room, and the last one starts.
    let first = nth_process(&t, 0).await;
    let line = first.wait_lines(1).await.remove(0);
    turn(&first, &line, "session-first", "Done").await;
    let fourth = nth_process(&t, 3).await;
    assert!(first.killed(), "the idle one made room");
    // It holds the message none of the first three got.
    let mut first_three = Vec::new();
    for n in 0..3 {
        let line = nth_process(&t, n).await.wait_lines(1).await.remove(0);
        first_three.push(line["message"]["content"][0]["text"].clone());
    }
    let waited = fourth.wait_lines(1).await.remove(0);
    let waited = &waited["message"]["content"][0]["text"];
    assert!(
        waited.as_str().is_some_and(|text| text.contains("To ")),
        "{waited}"
    );
    assert!(!first_three.contains(waited), "{waited} was already sent");
}
