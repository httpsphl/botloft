//! The automatic backup (spec 27.10), against the real account server running
//! in this process. The daemon's clock is moved by hand; `tick` is one pass of
//! the loop that runs in the real daemon.

mod common;

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use botloft_core::protocol::BackupScope;
use botloftd::autobackup::{MemorySecrets, tick};
use botloftd::backup::restore;
use botloftd::clock::Clock;
use botloftd::paths::Paths;
use common::cloud_server::{cloud_server, sign_in, signed_in};
use common::{Client, TestDaemon};
use serde_json::{Value, json};

const PASSPHRASE: &str = "correct horse battery";
const DAY: Duration = Duration::from_secs(24 * 3600);
const HOUR: Duration = Duration::from_secs(3600);

/// One pass of the loop, on the blocking pool as in the daemon.
async fn look(t: &TestDaemon) {
    let daemon = Arc::clone(&t.daemon);
    tokio::task::spawn_blocking(move || tick(&daemon))
        .await
        .expect("a pass");
}

async fn copies(app: &mut Client) -> Vec<Value> {
    app.call("cloud.copies", Value::Null).await.expect("copies")["copies"]
        .as_array()
        .expect("a list")
        .clone()
}

async fn status(app: &mut Client) -> Value {
    app.call("autobackup.status", Value::Null)
        .await
        .expect("status")
}

async fn turn_on(app: &mut Client, every: &str) -> Value {
    app.call(
        "autobackup.enable",
        json!({ "passphrase": PASSPHRASE, "every": every }),
    )
    .await
    .expect("enable")
}

/// A crew with a bot whose memory the copy carries; returns the memory file.
async fn a_bot_with_memory(t: &TestDaemon, app: &mut Client) -> PathBuf {
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "Finds", "instructions": "" });
    app.call("bots.create", bot).await.expect("bot");
    let memory = t.paths.bot_workspace("ops", "scout").join("CLAUDE.md");
    fs::write(&memory, "The client likes short reports.").expect("memory");
    memory
}

fn ms(by: Duration) -> i64 {
    i64::try_from(by.as_millis()).expect("fits")
}

#[tokio::test]
async fn a_daily_copy_goes_up_when_something_changed_and_not_before() {
    let cloud = cloud_server().await;
    let (t, mut app) = signed_in(&cloud).await;
    let memory = a_bot_with_memory(&t, &mut app).await;

    let off = status(&mut app).await;
    assert_eq!(off["available"], true);
    assert_eq!(off["enabled"], false);

    // Turning it on looks at once: the first copy goes up.
    let on = turn_on(&mut app, "daily").await;
    assert_eq!(
        (on["enabled"].clone(), on["every"].clone()),
        (true.into(), "daily".into())
    );
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 1);
    let sent_at = t.clock.now_ms();
    let after = status(&mut app).await;
    assert_eq!(after["lastOkAt"], sent_at);
    assert_eq!(after["nextAt"], sent_at + ms(DAY));
    assert!(after["lastError"].is_null());

    // Not its time yet: nothing happens, even with a change.
    fs::write(&memory, "The client likes long reports.").expect("changed");
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 1);

    // Its time, and what changed goes up.
    t.clock.advance(DAY + HOUR);
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 2);

    // Its time again and nothing changed: it looks and sends nothing.
    t.clock.advance(DAY + HOUR);
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 2);
    let quiet = status(&mut app).await;
    assert_eq!(quiet["nextAt"], t.clock.now_ms() + ms(DAY));

    // The copy is sealed with the passphrase it keeps, and it is the light one.
    let newest = copies(&mut app).await[0]["id"].clone();
    let downloaded = app
        .call("cloud.download", json!({ "id": newest }))
        .await
        .expect("download");
    let file = PathBuf::from(downloaded["path"].as_str().expect("path"));
    let elsewhere = tempfile::tempdir().expect("dir");
    let paths = Paths::new(elsewhere.path().join("home"), elsewhere.path().join("work"));
    assert!(restore::stage(&paths, &file, "not the passphrase").is_err());
    let manifest = restore::stage(&paths, &file, PASSPHRASE).expect("stage");
    assert_eq!(manifest.scope, BackupScope::Light);
}

#[tokio::test]
async fn a_weekly_copy_waits_a_week() {
    let cloud = cloud_server().await;
    let (t, mut app) = signed_in(&cloud).await;
    let memory = a_bot_with_memory(&t, &mut app).await;
    turn_on(&mut app, "weekly").await;
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 1);

    fs::write(&memory, "Changed.").expect("changed");
    t.clock.advance(DAY * 3);
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 1);
    t.clock.advance(DAY * 5);
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 2);
}

#[tokio::test]
async fn what_changes_by_itself_does_not_make_a_copy() {
    let cloud = cloud_server().await;
    let (t, mut app) = signed_in(&cloud).await;
    a_bot_with_memory(&t, &mut app).await;
    turn_on(&mut app, "daily").await;
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 1);

    // A conversation is not worth a copy.
    let bots = app.call("bots.list", Value::Null).await.expect("bots");
    let bot = bots[0]["id"].clone();
    app.call("messages.send", json!({ "botId": bot, "body": "hello" }))
        .await
        .expect("send");
    t.clock.advance(DAY + HOUR);
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 1);
}

#[tokio::test]
async fn a_copy_sent_by_hand_counts_for_the_automatic_one() {
    let cloud = cloud_server().await;
    let (t, mut app) = signed_in(&cloud).await;
    a_bot_with_memory(&t, &mut app).await;
    turn_on(&mut app, "daily").await;
    // The owner is quicker than the loop.
    app.call("cloud.upload", json!({ "passphrase": PASSPHRASE }))
        .await
        .expect("upload");
    look(&t).await;
    assert_eq!(copies(&mut app).await.len(), 1);
    // And the next look is a day away.
    let state = status(&mut app).await;
    assert_eq!(state["nextAt"], t.clock.now_ms() + ms(DAY));
}

#[tokio::test]
async fn turning_it_on_asks_for_what_it_needs() {
    let cloud = cloud_server().await;
    let t = TestDaemon::start_with_cloud(&cloud.url).await;
    let mut app = t.session().await;
    let enable = |passphrase: &str| json!({ "passphrase": passphrase, "every": "daily" });

    let no_account = app
        .call("autobackup.enable", enable(PASSPHRASE))
        .await
        .expect_err("no account");
    assert_eq!(no_account.reason, "not_signed_in");

    sign_in(&cloud, &mut app).await;
    let short = app
        .call("autobackup.enable", enable("short"))
        .await
        .expect_err("short");
    assert_eq!(short.reason, "short_passphrase");
    assert_eq!(status(&mut app).await["enabled"], false);
    assert_eq!(t.daemon.autobackup.secrets().load().expect("load"), None);
}

#[tokio::test]
async fn without_a_credential_store_it_cannot_be_turned_on() {
    let cloud = cloud_server().await;
    let t =
        TestDaemon::start_with_secrets(&cloud.url, Arc::new(MemorySecrets::unavailable())).await;
    let mut app = t.session().await;
    sign_in(&cloud, &mut app).await;
    assert_eq!(status(&mut app).await["available"], false);
    let refused = app
        .call(
            "autobackup.enable",
            json!({ "passphrase": PASSPHRASE, "every": "daily" }),
        )
        .await
        .expect_err("no store");
    assert_eq!(refused.reason, "no_keystore");
}

#[tokio::test]
async fn it_stops_when_it_is_turned_off_the_passphrase_is_gone_or_the_account_is_left() {
    let cloud = cloud_server().await;
    let (t, mut app) = signed_in(&cloud).await;
    a_bot_with_memory(&t, &mut app).await;
    let kept = || t.daemon.autobackup.secrets().load().expect("load");

    // Turning it off by hand forgets the passphrase.
    turn_on(&mut app, "daily").await;
    assert_eq!(kept().as_deref(), Some(PASSPHRASE));
    let off = app
        .call("autobackup.disable", Value::Null)
        .await
        .expect("disable");
    assert_eq!(off["enabled"], false);
    assert_eq!(kept(), None);

    // The passphrase disappears from the credential store.
    turn_on(&mut app, "daily").await;
    t.daemon.autobackup.secrets().delete().expect("delete");
    look(&t).await;
    let stopped = status(&mut app).await;
    assert_eq!(stopped["enabled"], false);
    assert_eq!(stopped["lastError"], "no_passphrase");
    assert!(copies(&mut app).await.is_empty());

    // Signing out forgets the passphrase and turns it off.
    turn_on(&mut app, "daily").await;
    app.call("cloud.signout", Value::Null)
        .await
        .expect("signout");
    assert_eq!(status(&mut app).await["enabled"], false);
    assert_eq!(kept(), None);
}

#[tokio::test]
async fn the_apps_are_told_when_it_changes_and_what_it_saves_has_no_passphrase() {
    let cloud = cloud_server().await;
    let (t, mut app) = signed_in(&cloud).await;
    turn_on(&mut app, "weekly").await;
    let told = app.notification("autobackup.changed").await;
    assert_eq!(told["enabled"], true);
    assert_eq!(told["every"], "weekly");

    app.call("autobackup.set_every", json!({ "every": "daily" }))
        .await
        .expect("set");
    assert_eq!(status(&mut app).await["every"], "daily");

    // What it remembers is in the data folder, and the passphrase is not.
    let saved = fs::read_to_string(t.paths.home.join("autobackup.json")).expect("saved");
    assert!(saved.contains("\"enabled\":true") && saved.contains("daily"));
    assert!(!saved.contains(PASSPHRASE));
}

#[tokio::test]
async fn a_try_without_a_connection_is_made_again_in_a_quarter_of_an_hour() {
    // Signed in to a server that is not there: nothing listens on port 1.
    let url = "http://127.0.0.1:1";
    let t = TestDaemon::start_with_cloud(url).await;
    let mut app = t.session().await;
    fs::create_dir_all(t.paths.secrets()).expect("secrets");
    let account = json!({ "url": url, "token": "t", "email": "ana@exemplo.com", "device": "d" });
    fs::write(t.paths.secrets().join("cloud.json"), account.to_string()).expect("account");
    a_bot_with_memory(&t, &mut app).await;

    turn_on(&mut app, "daily").await;
    look(&t).await;
    let failed = status(&mut app).await;
    assert_eq!(failed["enabled"], true);
    assert_eq!(failed["lastError"], "offline");
    assert!(failed["lastOkAt"].is_null());
    assert_eq!(
        failed["nextAt"],
        t.clock.now_ms() + ms(Duration::from_secs(15 * 60))
    );

    // It does not try before then.
    t.clock.advance(Duration::from_secs(10 * 60));
    look(&t).await;
    assert_eq!(status(&mut app).await["nextAt"], failed["nextAt"]);
}
