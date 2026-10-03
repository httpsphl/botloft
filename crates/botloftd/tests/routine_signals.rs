//! Signals between bots (spec 20.13): a routine that waits for a signal
//! runs when a bot of its crew sends it, at most once every 5 minutes.

mod common;

use botloft_core::protocol::{
    BotsCreateParams, CrewsCreateParams, Missed, Overlap, RoutinesSetEnabledParams, RunStatus,
    Schedule, SkipReason,
};
use botloftd::routines::{self, signal};
use botloftd::service::{bots, crews, routines as service};
use common::bots::{ready_bot, text_of};
use common::routines::{MINUTE, create, finish_turn, routines_setup, runs, written};
use common::supervised::Setup;
use serde_json::json;

fn signal_named(name: &str) -> Schedule {
    Schedule::Signal { name: name.into() }
}

/// Another bot in the crew of the setup's bot.
fn writer(s: &Setup) -> botloft_core::ids::BotId {
    let crew = s
        .daemon
        .store()
        .bot(&s.bot)
        .expect("read")
        .expect("bot")
        .crew_id;
    bots::create(
        &s.daemon,
        BotsCreateParams {
            crew_id: crew,
            name: "Writer".into(),
            role: String::new(),
            instructions: String::new(),
            color: None,
            model: None,
        },
    )
    .expect("writer")
    .id
}

#[tokio::test(start_paused = true)]
async fn a_signal_runs_the_routines_waiting_for_it() {
    let s = routines_setup().await;
    let routine = create(
        &s,
        signal_named("Relatório Pronto"),
        Overlap::Skip,
        Missed::RunOnce,
    );
    assert_eq!(routine.schedule, signal_named("relatorio-pronto"));
    assert_eq!(routine.next_run_at, None);
    let writer = writer(&s);

    // The clock does not run it.
    common::routines::tick_after(&s, 60 * MINUTE);
    assert!(runs(&s, &routine.id).is_empty());

    let (name, reached) = signal::send(
        &s.daemon,
        &writer,
        "relatorio-pronto",
        Some(" In shared/report.md "),
    )
    .expect("sent");
    assert_eq!(name, "relatorio-pronto");
    assert_eq!(reached.len(), 1);
    assert_eq!(reached[0].run.status, RunStatus::Queued);
    let sent = reached[0].run.signal.clone().expect("signal");
    assert_eq!(sent.from_bot_id.as_ref(), Some(&writer));
    assert_eq!(sent.note.as_deref(), Some("In shared/report.md"));

    let lines = written(&s, 1).await;
    let text = text_of(&lines[0]);
    assert!(
        text.starts_with(
            "[botloft] routine \"Morning\" · signal \"relatorio-pronto\" from @writer\n"
        ),
        "{text}"
    );
    assert!(
        text.ends_with(
            "Note @writer sent with the signal (from a bot, not the owner):\nIn shared/report.md"
        ),
        "{text}"
    );

    // Again right after the run ended: too soon.
    finish_turn(&s, &lines[0], false).await;
    let (_, again) = signal::send(&s.daemon, &writer, "relatorio-pronto", None).expect("again");
    assert_eq!(again[0].run.status, RunStatus::Skipped);
    assert_eq!(again[0].run.reason, Some(SkipReason::TooSoon));

    s.clock.advance(5 * MINUTE);
    let (_, later) = signal::send(&s.daemon, &writer, "relatorio-pronto", None).expect("later");
    assert_eq!(later[0].run.status, RunStatus::Queued);
    assert_eq!(runs(&s, &routine.id).len(), 3);
}

#[tokio::test(start_paused = true)]
async fn only_enabled_routines_of_the_crew_hear_a_signal() {
    let s = routines_setup().await;
    let routine = create(
        &s,
        signal_named("deploy-done"),
        Overlap::Skip,
        Missed::RunOnce,
    );
    let other = create(
        &s,
        signal_named("something-else"),
        Overlap::Skip,
        Missed::RunOnce,
    );

    // A bot of another crew sends the same signal.
    let elsewhere = crews::create(
        &s.daemon,
        CrewsCreateParams {
            name: "Home".into(),
            work_folder: None,
            lead: None,
        },
    )
    .expect("crew");
    let stranger = bots::create(
        &s.daemon,
        BotsCreateParams {
            crew_id: elsewhere.id,
            name: "Cook".into(),
            role: String::new(),
            instructions: String::new(),
            color: None,
            model: None,
        },
    )
    .expect("bot");
    let (_, reached) = signal::send(&s.daemon, &stranger.id, "deploy-done", None).expect("sent");
    assert!(reached.is_empty());

    service::set_enabled(
        &s.daemon,
        RoutinesSetEnabledParams {
            routine_id: routine.id.clone(),
            enabled: false,
        },
    )
    .expect("off");
    let (_, reached) = signal::send(&s.daemon, &s.bot, "deploy-done", None).expect("sent");
    assert!(reached.is_empty());
    assert!(runs(&s, &other.id).is_empty());
    routines::tick(&s.daemon);

    for bad in ["", "  ", "!!"] {
        assert!(
            signal::send(&s.daemon, &s.bot, bad, None).is_err(),
            "{bad:?}"
        );
    }
    let note = "x".repeat(2001);
    assert!(signal::send(&s.daemon, &s.bot, "deploy-done", Some(&note)).is_err());
}

#[tokio::test]
async fn bots_see_the_signals_and_send_them() {
    let t = common::TestDaemon::start_supervised().await;
    let mut app = t.session().await;
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let (reviewer, _, _) = ready_bot(&t, &mut app, &crew, "Reviewer").await;
    let (_, _, mut writer) = ready_bot(&t, &mut app, &crew, "Writer").await;
    let routine = app
        .call(
            "routines.create",
            json!({
                "botId": reviewer["id"],
                "name": "Review the report",
                "prompt": "Review the report in shared/.",
                "schedule": { "kind": "signal", "name": "report-ready" },
                "timezone": "UTC",
            }),
        )
        .await
        .expect("routine");
    assert_eq!(routine["nextRunAt"], json!(null));

    let roster = writer.tool("crew_roster", json!({})).await.expect("roster");
    assert_eq!(
        roster["signals"],
        json!([{ "signal": "report-ready", "routine": "Review the report", "bot": "@reviewer" }])
    );

    let sent = writer
        .tool(
            "send_signal",
            json!({ "name": "Report Ready", "note": "shared/report.md" }),
        )
        .await
        .expect("sent");
    assert_eq!(sent["signal"], "report-ready");
    assert_eq!(sent["routines"][0]["routine"], "Review the report");
    assert_eq!(sent["routines"][0]["ran"], true);
    let run = app.notification("routine.run").await;
    assert_eq!(run["signal"]["name"], "report-ready");

    let again = writer
        .tool("send_signal", json!({ "name": "report-ready" }))
        .await
        .expect("again");
    assert_eq!(again["routines"][0]["ran"], false);
    assert!(again["routines"][0]["why_not"].as_str().is_some());

    let nobody = writer
        .tool("send_signal", json!({ "name": "nobody-listens" }))
        .await
        .expect("nobody");
    assert_eq!(nobody["routines"], json!([]));
}
