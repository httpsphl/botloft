//! A crew's bot whose turn fails is reported to the chief, who can have it
//! try again with `retry_agent`, twice an hour at most (spec 10.6).

mod common;

use std::time::Duration;

use botloft_core::protocol::BotState;
use common::bots::{ready_bot, text_of};
use common::mcp::Mcp;
use common::{TestDaemon, stream};
use serde_json::json;

#[tokio::test]
async fn a_failed_turn_reaches_the_chief_who_has_it_try_again() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call(
            "crews.create",
            json!({
                "name": "Site",
                "lead": { "name": "Chefe", "role": "Leads the crew", "instructions": "Ship it" },
            }),
        )
        .await
        .expect("crew");
    let bots = app
        .call("bots.list", json!({ "crewId": crew["id"] }))
        .await
        .expect("bots");
    let chief_bot = bots[0].clone();
    let chief_process = t.process_of(&chief_bot).await;
    t.until_state(&chief_bot, BotState::Idle).await;
    let mut chief = Mcp::new(
        t.addr,
        chief_process.env("BOTLOFT_BOT_TOKEN").expect("token"),
    );
    let (writer, writer_process, mut writer_mcp) = ready_bot(&t, &mut app, &crew, "Writer").await;

    // The owner asks Writer, and its turn fails.
    app.call(
        "messages.send",
        json!({ "botId": writer["id"], "body": "Draft the launch post." }),
    )
    .await
    .expect("send");
    let asked = writer_process.wait_lines(1).await;
    writer_process.emit(stream::replay(&asked[0])).await;
    writer_process.emit(stream::result(true)).await;

    // The chief hears of it, with what Writer was working on.
    let told = chief_process.wait_lines(1).await;
    let report = text_of(&told[0]);
    assert!(
        report.contains("@writer's turn failed: it ended with an error (api_error)."),
        "{report}"
    );
    assert!(
        report.contains("from the owner:\n> Draft the launch post."),
        "{report}"
    );
    assert!(report.contains("retry_agent"), "{report}");

    // Only the chief may have it try again.
    let refused = writer_mcp
        .tool("retry_agent", json!({ "bot": "writer" }))
        .await
        .expect_err("not the chief");
    assert!(refused.contains("only the crew's chief"), "{refused}");

    chief
        .tool(
            "retry_agent",
            json!({ "bot": "@writer", "note": "Keep it under 200 words." }),
        )
        .await
        .expect("retry");
    let again = writer_process.wait_lines(2).await;
    let retried = text_of(&again[1]);
    assert!(retried.contains("Please try this again."), "{retried}");
    assert!(retried.contains("Keep it under 200 words."), "{retried}");
    assert!(retried.contains("Draft the launch post."), "{retried}");

    // It fails again at once: the chief already heard of Writer a moment ago.
    writer_process.emit(stream::replay(&again[1])).await;
    writer_process.emit(stream::result(true)).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert_eq!(
        chief_process.input_lines().len(),
        1,
        "no second report so soon"
    );

    // Twice an hour, then no more.
    chief
        .tool("retry_agent", json!({ "bot": "writer" }))
        .await
        .expect("second retry");
    let past = chief
        .tool("retry_agent", json!({ "bot": "writer" }))
        .await
        .expect_err("third retry");
    assert!(
        past.contains("already tried again 2 times this hour"),
        "{past}"
    );
}

#[tokio::test]
async fn a_crew_without_a_chief_tells_no_one() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Solo" }))
        .await
        .expect("crew");
    let (writer, writer_process, _) = ready_bot(&t, &mut app, &crew, "Writer").await;
    let (other, other_process, _) = ready_bot(&t, &mut app, &crew, "Other").await;
    app.call(
        "messages.send",
        json!({ "botId": writer["id"], "body": "Draft it." }),
    )
    .await
    .expect("send");
    let asked = writer_process.wait_lines(1).await;
    writer_process.emit(stream::replay(&asked[0])).await;
    writer_process.emit(stream::result(true)).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    assert!(other_process.input_lines().is_empty(), "{other}");
}
