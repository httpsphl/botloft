//! The phone's Home (spec 28.15): the owner's shortcuts, run with a tap, and
//! the day so far.

mod common;

use botloft_core::protocol::{FromPhone, RoutinesCreateParams, Schedule, ToPhone};
use common::bots::ready_bot;
use common::mobile::{Phone, connect_phone, setup};
use serde_json::{Value, json};

fn bot_id(bot: &Value) -> botloft_core::ids::BotId {
    bot["id"].as_str().expect("id").parse().expect("bot id")
}

fn routine(bot: &Value, name: &str, schedule: Schedule) -> RoutinesCreateParams {
    RoutinesCreateParams {
        bot_id: bot_id(bot),
        name: name.into(),
        prompt: "Summarize.".into(),
        schedule,
        timezone: "UTC".into(),
        overlap: None,
        missed: None,
    }
}

async fn ran(phone: &mut Phone, client_id: &str, routine_id: &str) -> (bool, Option<String>) {
    phone
        .send(&FromPhone::RoutineRun {
            client_id: client_id.into(),
            routine_id: routine_id.parse().expect("routine id"),
        })
        .await;
    loop {
        if let ToPhone::Ran {
            client_id: got,
            ok,
            reason,
        } = phone.next().await
            && got == client_id
        {
            return (ok, reason);
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_home_lists_the_shortcuts_and_runs_only_those() {
    let (cloud, t, mut app) = setup().await;
    let mut phone = connect_phone(&cloud, &mut app).await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (bot, process, _) = ready_bot(&t, &mut app, &crew, "Lead").await;
    drop(process);
    let daemon = &t.daemon;
    let create = |params| botloftd::service::routines::create(daemon, params).expect("routine");
    let summary = create(routine(&bot, "Morning summary", Schedule::Manual));
    let backup = create(routine(&bot, "backup files", Schedule::Manual));
    let hourly = create(routine(
        &bot,
        "Hourly check",
        Schedule::Interval { minutes: 60 },
    ));

    phone.send(&FromPhone::Home { since: 0 }).await;
    let (shortcuts, today) = loop {
        if let ToPhone::Home { shortcuts, today } = phone.next().await {
            break (shortcuts, today);
        }
    };
    // By name, whatever the case; a scheduled routine is not a shortcut.
    let names: Vec<&str> = shortcuts.iter().map(|one| one.name.as_str()).collect();
    assert_eq!(names, ["backup files", "Morning summary"]);
    assert_eq!(shortcuts[1].routine_id, summary.id);
    assert_eq!(shortcuts[1].bot_id, bot_id(&bot));
    assert!(today.is_some());

    let (ok, reason) = ran(&mut phone, "r1", summary.id.as_str()).await;
    assert!(ok, "{reason:?}");
    let runs = app
        .call("routines.runs", json!({ "routineId": summary.id }))
        .await
        .expect("runs");
    assert_eq!(runs.as_array().expect("runs").len(), 1);

    let (ok, reason) = ran(&mut phone, "r2", hourly.id.as_str()).await;
    assert!(!ok);
    assert_eq!(reason.as_deref(), Some("not_found"));

    // An archived shortcut leaves the Home and cannot be run from there.
    app.call("routines.archive", json!({ "routineId": backup.id }))
        .await
        .expect("archive");
    let (ok, _) = ran(&mut phone, "r3", backup.id.as_str()).await;
    assert!(!ok);
}
