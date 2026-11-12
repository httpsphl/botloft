//! Titles made by AI (spec 40.6): the start of the first message names a
//! new conversation, then the title the AI makes after the first reply. A
//! title the owner gave stays.

mod common;

use std::io;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use botloft_core::ids::ConversationId;
use botloftd::state::Daemon;
use botloftd::titles::{TitleAsk, TitleMaker};
use common::conversations::{holding, open, send, sleeping_bot, turn};
use common::{Client, TestDaemon};
use serde_json::{Value, json};

/// Answers `title`, or fails with none, and keeps what it was asked.
struct FakeTitles {
    title: Option<&'static str>,
    asked: Mutex<Vec<TitleAsk>>,
}

impl TitleMaker for FakeTitles {
    fn make(&self, _: &Daemon, ask: &TitleAsk) -> io::Result<String> {
        self.asked.lock().expect("titles").push(ask.clone());
        self.title
            .map(|title| format!("\"{title}.\""))
            .ok_or_else(|| io::Error::other("no title"))
    }
}

async fn start(title: Option<&'static str>) -> (TestDaemon, Arc<FakeTitles>) {
    let t = TestDaemon::start_supervised().await;
    t.daemon
        .settings
        .set_sleep_after(Some(Duration::from_secs(600)));
    let titles = Arc::new(FakeTitles {
        title,
        asked: Mutex::default(),
    });
    t.daemon.titles.set_maker(titles.clone());
    tokio::spawn(botloftd::service::conversations::follow(Arc::clone(
        &t.daemon,
    )));
    tokio::spawn(botloftd::titles::run(Arc::clone(&t.daemon)));
    (t, titles)
}

/// The next `conversation.changed` about `id` that `matches`.
async fn changed(app: &mut Client, id: &ConversationId, matches: impl Fn(&Value) -> bool) -> Value {
    let wait = async {
        loop {
            let shown = app.notification("conversation.changed").await;
            if shown["id"] == id.as_str() && matches(&shown) {
                return shown;
            }
        }
    };
    tokio::time::timeout(Duration::from_secs(5), wait)
        .await
        .expect("conversation.changed")
}

const FIRST: &str = "I want to plan a five day trip to Lisbon in March, cheaply";

#[tokio::test(flavor = "multi_thread")]
async fn a_new_conversation_is_named_by_its_first_message_then_by_the_ai() {
    let (t, titles) = start(Some("Lisbon trip in March")).await;
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let other = open(&mut app, &id).await;

    send(&mut app, &bot, FIRST, Some(&other)).await;
    let meanwhile = changed(&mut app, &other, |shown| !shown["title"].is_null()).await;
    assert_eq!(
        meanwhile["title"],
        "I want to plan a five day trip to Lisbon…"
    );
    assert_eq!(meanwhile["titleBy"], Value::Null);

    let (process, line) = holding(&t, 1, FIRST).await;
    turn(
        &process,
        &line,
        "session-other",
        "Sure, here is a plan for March.",
    )
    .await;
    let named = changed(&mut app, &other, |shown| shown["titleBy"] == "ai").await;
    assert_eq!(named["title"], "Lisbon trip in March");
    let asked = titles.asked.lock().expect("titles").clone();
    assert_eq!(
        asked,
        [TitleAsk {
            first: FIRST.to_owned(),
            reply: "Sure, here is a plan for March.".to_owned(),
        }]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_s_title_stays_and_a_failed_one_leaves_the_start() {
    let (t, titles) = start(None).await;
    let mut app = t.session().await;
    let (bot, id) = sleeping_bot(&t, &mut app).await;
    let named = open(&mut app, &id).await;
    let other = open(&mut app, &id).await;
    let rename = json!({ "conversationId": named, "title": "Mine" });
    app.call("conversations.rename", rename)
        .await
        .expect("rename");

    send(&mut app, &bot, "Plan the trip", Some(&named)).await;
    send(&mut app, &bot, FIRST, Some(&other)).await;
    for body in ["Plan the trip", FIRST] {
        let (process, line) = holding(&t, 2, body).await;
        turn(&process, &line, "session", "On it.").await;
    }
    changed(&mut app, &other, |shown| !shown["title"].is_null()).await;
    // Only the one without the owner's title was asked for; it failed.
    for _ in 0..200 {
        if !titles.asked.lock().expect("titles").is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(titles.asked.lock().expect("titles").len(), 1);
    let store = t.daemon.store();
    let kept = store.conversation(&named).unwrap().expect("named");
    assert_eq!(
        (kept.title.as_deref(), kept.title_by.as_deref()),
        (Some("Mine"), Some("owner"))
    );
    let left = store.conversation(&other).unwrap().expect("other");
    assert_eq!(
        (left.title.as_deref(), left.title_by.as_deref()),
        (Some("I want to plan a five day trip to Lisbon…"), None)
    );
}
