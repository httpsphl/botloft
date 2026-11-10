//! A 👍 or 👎 on a reply that asks something is the owner's answer (spec
//! 8.9): it goes at once, as a message of the owner with just the emoji,
//! and the bot reads which question it answers. Other reactions still
//! wait for the owner's next message.

mod common;

use botloft_core::protocol::BotState;
use common::TestDaemon;
use common::bots::text_of;
use common::stream;
use serde_json::{Value, json};

const QUESTION: &str = "The report is ready.\n\nShould I send it to Acme?";

#[tokio::test]
async fn a_thumb_on_a_question_answers_it_at_once() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let scout = app
        .call(
            "bots.create",
            json!({ "crewId": crew["id"], "name": "Scout", "role": "", "instructions": "" }),
        )
        .await
        .expect("bot");
    t.until_state(&scout, BotState::Idle).await;
    let process = t.process_of(&scout).await;

    // Two replies: one that says something, one that asks.
    for (at, (ask, says)) in [
        ("Who signs?", "Acme signs annual."),
        ("And the report?", QUESTION),
    ]
    .into_iter()
    .enumerate()
    {
        app.call(
            "messages.send",
            json!({ "botId": scout["id"], "body": ask }),
        )
        .await
        .expect("send");
        let line = process.wait_lines(at + 1).await.remove(at);
        stream::answer(&process, &line, says).await;
        t.until_state(&scout, BotState::Idle).await;
    }
    let history = app
        .call("chat.history", json!({ "botId": scout["id"] }))
        .await
        .expect("history");
    let reply = |text: &str| {
        history
            .as_array()
            .expect("items")
            .iter()
            .find(|item| item["body"]["kind"] == "reply" && item["body"]["text"] == text)
            .expect("reply")
            .clone()
    };
    let (said, asked) = (&reply("Acme signs annual."), &reply(QUESTION));
    let react = |item: &Value, emoji: &str| json!({ "botId": scout["id"], "itemId": item["id"], "emoji": emoji });

    // A thumb on what was only said waits, and so does a heart on the
    // question.
    for (item, emoji) in [(said, "👍"), (asked, "❤️")] {
        let put = app
            .call("reactions.set", react(item, emoji))
            .await
            .expect("react");
        assert_eq!(put["sentIn"], Value::Null);
    }
    assert_eq!(process.input_lines().len(), 2);

    // 👎 on the question goes now, and takes the waiting 👍 along.
    let answer = app
        .call("reactions.set", react(asked, "👎"))
        .await
        .expect("answer");
    assert_ne!(answer["sentIn"], Value::Null);
    let line = process.wait_lines(3).await.remove(2);
    let text = text_of(&line);
    assert!(
        text.contains("Answered 👎 (no) to your question: \"Should I send it to Acme?\""),
        "{text}"
    );
    assert!(
        text.contains("Reacted 👍 to: \"Acme signs annual.\""),
        "{text}"
    );
    assert!(text.ends_with("\n\n👎"), "{text}");
    let message = app
        .call("messages.list", json!({ "botId": scout["id"], "limit": 1 }))
        .await
        .expect("messages");
    assert_eq!(message[0]["id"], answer["sentIn"]);
    assert_eq!(message[0]["body"], "👎");
    assert_eq!(message[0]["fromKind"], "owner");
    stream::answer(&process, &line, "Then I hold it.").await;
    t.until_state(&scout, BotState::Idle).await;

    // The same answer again sends nothing; a changed one goes again.
    let again = app
        .call("reactions.set", react(asked, "👎"))
        .await
        .expect("again");
    assert_eq!(again, answer);
    let changed = app
        .call("reactions.set", react(asked, "👍"))
        .await
        .expect("changed");
    assert_ne!(changed["sentIn"], answer["sentIn"]);
    let line = process.wait_lines(4).await.remove(3);
    assert_eq!(
        text_of(&line),
        "Answered 👍 (yes) to your question: \"Should I send it to Acme?\"\n\n👍"
    );
}
