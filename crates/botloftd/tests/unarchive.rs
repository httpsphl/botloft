//! `crews.unarchive` and `bots.unarchive` (spec 7.6): what was archived
//! comes back with what was archived with it, and starts again.

mod common;

use botloft_core::protocol::BotState;
use common::bots::ready_bot;
use common::{Client, TestDaemon};
use serde_json::{Value, json};

const CONFLICT: i64 = -32003;

async fn names(app: &mut Client, method: &str) -> Vec<String> {
    let list = app.call(method, json!({})).await.expect("list");
    list.as_array()
        .expect("list")
        .iter()
        .map(|item| item["name"].as_str().expect("name").to_owned())
        .collect()
}

async fn routine(app: &mut Client, bot: &Value) -> Value {
    let routine = json!({
        "botId": bot["id"], "name": "Morning inbox", "prompt": "Read the new mail.",
        "schedule": { "kind": "weekly", "days": [1, 2, 3, 4, 5], "time": "08:00" },
        "timezone": "America/Sao_Paulo",
    });
    app.call("routines.create", routine).await.expect("routine")
}

async fn routines_of(app: &mut Client, bot: &Value) -> Value {
    app.call("routines.list", json!({ "botId": bot["id"] }))
        .await
        .expect("routines")
}

#[tokio::test]
async fn a_crew_comes_back_with_what_was_archived_with_it() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (scout, _, _) = ready_bot(&t, &mut app, &crew, "Scout").await;
    let (writer, first, _) = ready_bot(&t, &mut app, &crew, "Writer").await;
    routine(&mut app, &writer).await;
    app.call("bots.archive", json!({ "botId": scout["id"] }))
        .await
        .expect("archive a bot");
    app.call("crews.archive", json!({ "crewId": crew["id"] }))
        .await
        .expect("archive the crew");
    t.until_state(&writer, BotState::Archived).await;
    assert_eq!(routines_of(&mut app, &writer).await, json!([]));

    let back = app
        .call("crews.unarchive", json!({ "crewId": crew["id"] }))
        .await
        .expect("unarchive");
    assert_eq!(back["archivedAt"], Value::Null);
    assert_eq!(names(&mut app, "crews.list").await, ["Ops"]);
    // Scout was archived on its own before, so it stays in the archive.
    assert_eq!(names(&mut app, "bots.list").await, ["Writer"]);
    let archive = app
        .call("archive.list", json!(null))
        .await
        .expect("archive");
    assert_eq!(archive["crews"], json!([]));
    assert_eq!(archive["bots"][0]["name"], "Scout");

    // Writer starts again, with a token of its own.
    t.until_state(&writer, BotState::Idle).await;
    let second = t.process_of(&writer).await;
    assert_ne!(
        first.env("BOTLOFT_BOT_TOKEN"),
        second.env("BOTLOFT_BOT_TOKEN")
    );
    let routines = routines_of(&mut app, &writer).await;
    assert_eq!(routines[0]["name"], "Morning inbox");
    assert!(routines[0]["nextRunAt"].as_i64().is_some());

    let again = app
        .call("crews.unarchive", json!({ "crewId": crew["id"] }))
        .await
        .expect("idempotent");
    assert_eq!(again["archivedAt"], Value::Null);
}

#[tokio::test]
async fn a_bot_comes_back_while_its_crew_and_its_name_are_free() {
    let t = TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (scout, _, _) = ready_bot(&t, &mut app, &crew, "Scout").await;
    routine(&mut app, &scout).await;
    let paused = routine(&mut app, &scout).await;
    app.call(
        "routines.setEnabled",
        json!({ "routineId": paused["id"], "enabled": false }),
    )
    .await
    .expect("turn off");
    app.call("bots.archive", json!({ "botId": scout["id"] }))
        .await
        .expect("archive");

    // Another Scout took the name meanwhile.
    let (other, _, _) = ready_bot(&t, &mut app, &crew, "Scout").await;
    let err = app
        .call("bots.unarchive", json!({ "botId": scout["id"] }))
        .await
        .expect_err("name taken");
    assert_eq!(err.code, CONFLICT);
    app.call("bots.archive", json!({ "botId": other["id"] }))
        .await
        .expect("archive the other");

    // A bot of an archived crew comes back with its crew only.
    app.call("crews.archive", json!({ "crewId": crew["id"] }))
        .await
        .expect("archive the crew");
    let err = app
        .call("bots.unarchive", json!({ "botId": scout["id"] }))
        .await
        .expect_err("crew archived");
    assert_eq!(err.code, CONFLICT);
    app.call("crews.unarchive", json!({ "crewId": crew["id"] }))
        .await
        .expect("unarchive the crew");

    let back = app
        .call("bots.unarchive", json!({ "botId": scout["id"] }))
        .await
        .expect("unarchive");
    assert_eq!(back["archivedAt"], Value::Null);
    assert_eq!(names(&mut app, "bots.list").await, ["Scout"]);
    t.until_state(&scout, BotState::Idle).await;
    // Both routines come back; the one that was off stays off.
    let routines = routines_of(&mut app, &scout).await;
    let routines = routines.as_array().expect("routines");
    assert_eq!(routines.len(), 2);
    let off = routines
        .iter()
        .find(|routine| routine["id"] == paused["id"])
        .expect("the paused one");
    assert_eq!(off["enabled"], false);
    assert_eq!(off["nextRunAt"], Value::Null);
}
