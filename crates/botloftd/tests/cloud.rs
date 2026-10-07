//! The account and the copies in the cloud (spec 27.5), against the real
//! server (`botloft-cloud`) running in this process on 127.0.0.1.

mod common;

use std::fs;
use std::sync::Arc;
use std::time::Duration;

use botloft_cloud::{AppState, Clock, Config, CopyStore, Db, Mailer, Outbox, router};
use botloft_store::Store;
use botloftd::backup::restore;
use botloftd::paths::Paths;
use common::{Client, TestDaemon};
use serde_json::{Value, json};
use tokio::net::TcpListener;

/// The server of the account, and what a test needs to reach into.
struct CloudServer {
    url: String,
    outbox: Outbox,
    clock: Clock,
}

async fn cloud_server() -> CloudServer {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let url = format!("http://{}", listener.local_addr().expect("addr"));
    let (outbox, clock) = (Outbox::default(), Clock::default());
    let mut config = Config::for_tests();
    config.public_url = url.clone();
    let state = AppState {
        db: Db::memory().expect("db"),
        mailer: Arc::new(Mailer::Outbox(outbox.clone())),
        clock: clock.clone(),
        config: Arc::new(config),
        store: CopyStore::memory(),
    };
    tokio::spawn(async move {
        axum::serve(listener, router(state)).await.expect("serve");
    });
    CloudServer { url, outbox, clock }
}

impl CloudServer {
    /// The owner opens the link in the newest e-mail and presses the button.
    async fn press_the_link(&self) {
        let mail = self.outbox.sent().pop().expect("an e-mail");
        let link = mail
            .body
            .split_whitespace()
            .find(|word| word.starts_with("http"))
            .expect("a link");
        let code = link.split("code=").nth(1).expect("a code");
        let pressed = reqwest::Client::new()
            .post(format!("{}/v1/login/confirm", self.url))
            .header("content-type", "application/x-www-form-urlencoded")
            .body(format!("code={code}"))
            .send()
            .await
            .expect("press");
        assert!(pressed.status().is_success(), "{}", pressed.status());
    }
}

async fn signed_in(cloud: &CloudServer) -> (TestDaemon, Client) {
    let t = TestDaemon::start_with_cloud(&cloud.url).await;
    let mut app = t.session().await;
    app.call(
        "cloud.signin",
        json!({ "email": "ana@exemplo.com", "locale": "pt-BR" }),
    )
    .await
    .expect("signin");
    cloud.press_the_link().await;
    let signed = app.notification("cloud.signed_in").await;
    assert_eq!(signed["email"], "ana@exemplo.com");
    (t, app)
}

async fn fails(app: &mut Client, method: &str, params: Value) -> common::RpcFailure {
    app.call(method, params).await.expect_err("it should fail")
}

#[tokio::test]
async fn an_account_signs_in_sends_a_light_copy_and_gets_it_back_elsewhere() {
    let cloud = cloud_server().await;
    let t = TestDaemon::start_with_cloud(&cloud.url).await;
    let mut app = t.session().await;

    let status = app.call("cloud.status", Value::Null).await.expect("status");
    assert_eq!(status["url"], cloud.url.as_str());
    assert_eq!(status["signedIn"], false);
    assert_eq!(status["pending"], false);

    // Ask for the link: the e-mail goes in the app's language.
    let started = app
        .call(
            "cloud.signin",
            json!({ "email": "Ana@Exemplo.com", "locale": "pt-BR" }),
        )
        .await
        .expect("signin");
    assert_eq!(started["wait"], 600);
    assert!(cloud.outbox.sent()[0].body.contains("Abra este link"));
    let status = app.call("cloud.status", Value::Null).await.expect("status");
    assert_eq!(status["pending"], true);
    assert_eq!(status["signedIn"], false);

    // The owner opens it, anywhere; this computer notices.
    cloud.press_the_link().await;
    let signed = app.notification("cloud.signed_in").await;
    assert_eq!(signed["email"], "ana@exemplo.com");
    let status = app.call("cloud.status", Value::Null).await.expect("status");
    assert_eq!(status["signedIn"], true);
    assert_eq!(status["pending"], false);
    assert_eq!(status["email"], "ana@exemplo.com");
    assert_eq!(status["used"], 0);
    assert!(status["quota"].as_u64().expect("quota") > 0);
    let kept = fs::read_to_string(t.paths.secrets().join("cloud.json")).expect("credentials");
    assert!(kept.contains("ana@exemplo.com"));

    // A crew with a bot and its memory goes up, light and sealed.
    let crew = app
        .call("crews.create", json!({ "name": "Ops" }))
        .await
        .expect("crew");
    let bot = json!({ "crewId": crew["id"], "name": "Scout", "role": "Finds", "instructions": "" });
    app.call("bots.create", bot).await.expect("bot");
    let folder = t.paths.bot_workspace("ops", "scout");
    fs::write(folder.join("CLAUDE.md"), "The client likes short reports.").expect("memory");
    fs::write(t.paths.shared_dir("ops").join("report.md"), "# Report").expect("report");

    let short = fails(&mut app, "cloud.upload", json!({ "passphrase": "short" })).await;
    assert_eq!(short.reason, "short_passphrase");
    let copy = app
        .call("cloud.upload", json!({ "passphrase": "correct horse" }))
        .await
        .expect("upload");
    assert!(copy["id"].as_str().expect("id").starts_with("cpy_"));
    let size = copy["size"].as_u64().expect("size");
    assert!(size > 0);
    assert!(
        !t.paths.home.join("cloud-upload").exists(),
        "the sealed file is not kept"
    );
    let status = app.call("cloud.status", Value::Null).await.expect("status");
    assert_eq!(status["used"], size);

    let listed = app.call("cloud.copies", Value::Null).await.expect("copies");
    assert_eq!(listed["copies"].as_array().expect("copies").len(), 1);
    assert_eq!(listed["copies"][0]["id"], copy["id"]);

    // Another computer brings it down and restores it, the usual way.
    let downloaded = app
        .call("cloud.download", json!({ "id": copy["id"] }))
        .await
        .expect("download");
    let file = std::path::PathBuf::from(downloaded["path"].as_str().expect("path"));
    assert!(
        file.ends_with("cloud-download/copy.botloft")
            || file.ends_with("cloud-download\\copy.botloft")
    );
    assert_eq!(fs::metadata(&file).expect("file").len(), size);

    let other = tempfile::tempdir().expect("tempdir");
    let paths = Paths::new(other.path().join("Data"), other.path().join("Bots"));
    fs::create_dir_all(&paths.home).expect("home");
    drop(Store::open(&paths.db()).expect("its own database"));
    assert!(restore::stage(&paths, &file, "wrong horse").is_err());
    let manifest = restore::stage(&paths, &file, "correct horse").expect("stage");
    assert_eq!(manifest.scope, botloft_core::protocol::BackupScope::Light);
    restore::confirm(&paths).expect("confirm");
    restore::apply_pending(&paths, "t1")
        .expect("apply")
        .expect("applied");
    assert_eq!(
        fs::read_to_string(paths.bot_workspace("ops", "scout").join("CLAUDE.md")).expect("memory"),
        "The client likes short reports."
    );
    assert!(!paths.shared_dir("ops").join("report.md").exists());

    // Progress was told while the copy moved.
    assert!(!app.queued("cloud.progress").is_empty());

    app.call("cloud.delete", json!({ "id": copy["id"] }))
        .await
        .expect("delete");
    let listed = app.call("cloud.copies", Value::Null).await.expect("copies");
    assert!(listed["copies"].as_array().expect("copies").is_empty());

    // Signing out forgets this computer, and the account no longer opens.
    app.call("cloud.signout", Value::Null)
        .await
        .expect("signout");
    assert!(!t.paths.secrets().join("cloud.json").exists());
    let status = app.call("cloud.status", Value::Null).await.expect("status");
    assert_eq!(status["signedIn"], false);
    let out = fails(&mut app, "cloud.copies", Value::Null).await;
    assert_eq!(out.reason, "not_signed_in");
}

#[tokio::test]
async fn a_link_that_expires_or_is_cancelled_signs_nobody_in() {
    let cloud = cloud_server().await;
    let t = TestDaemon::start_with_cloud(&cloud.url).await;
    let mut app = t.session().await;

    app.call("cloud.signin", json!({ "email": "ana@exemplo.com" }))
        .await
        .expect("signin");
    cloud.clock.advance(601_000);
    app.notification("cloud.signin_expired").await;
    let status = app.call("cloud.status", Value::Null).await.expect("status");
    assert_eq!(status["pending"], false);
    assert_eq!(status["signedIn"], false);

    // Cancelled: the link may still be pressed, and nothing happens here.
    cloud.clock.advance(61_000);
    app.call("cloud.signin", json!({ "email": "ana@exemplo.com" }))
        .await
        .expect("signin");
    app.call("cloud.signin_cancel", Value::Null)
        .await
        .expect("cancel");
    cloud.press_the_link().await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    let status = app.call("cloud.status", Value::Null).await.expect("status");
    assert_eq!(status["signedIn"], false);
    assert_eq!(status["pending"], false);
    assert!(!t.paths.secrets().join("cloud.json").exists());
}

#[tokio::test]
async fn a_revoked_sign_in_is_forgotten_the_next_time_it_is_used() {
    let cloud = cloud_server().await;
    let (t, mut app) = signed_in(&cloud).await;
    // Another computer removed this one: its token no longer works.
    let path = t.paths.secrets().join("cloud.json");
    let mut kept: Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("json");
    kept["token"] = json!("a-token-the-server-does-not-know");
    fs::write(&path, kept.to_string()).expect("write");

    let out = fails(&mut app, "cloud.copies", Value::Null).await;
    assert_eq!(out.reason, "signed_out");
    assert!(!path.exists());
    let status = app.call("cloud.status", Value::Null).await.expect("status");
    assert_eq!(status["signedIn"], false);
}

#[tokio::test]
async fn the_cloud_asks_for_a_server_and_for_https_and_says_when_it_is_offline() {
    // No server set up.
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let none = fails(
        &mut app,
        "cloud.signin",
        json!({ "email": "ana@exemplo.com" }),
    )
    .await;
    assert_eq!(none.reason, "no_server");
    assert_eq!(
        app.call("cloud.status", Value::Null).await.expect("status")["url"],
        ""
    );

    // An address that is not https, and not on this computer.
    let t = TestDaemon::start_with_cloud("http://cloud.example.org").await;
    let mut app = t.session().await;
    let plain = fails(
        &mut app,
        "cloud.signin",
        json!({ "email": "ana@exemplo.com" }),
    )
    .await;
    assert_eq!(plain.reason, "no_server");

    // Nobody listens there.
    let t = TestDaemon::start_with_cloud("http://127.0.0.1:1").await;
    let mut app = t.session().await;
    let down = fails(
        &mut app,
        "cloud.signin",
        json!({ "email": "ana@exemplo.com" }),
    )
    .await;
    assert_eq!(down.reason, "offline");
}

#[tokio::test]
async fn the_servers_own_refusals_reach_the_app_with_their_reason() {
    let cloud = cloud_server().await;
    let t = TestDaemon::start_with_cloud(&cloud.url).await;
    let mut app = t.session().await;
    let bad = fails(
        &mut app,
        "cloud.signin",
        json!({ "email": "not-an-address" }),
    )
    .await;
    assert_eq!(bad.reason, "bad_email");
    app.call("cloud.signin", json!({ "email": "ana@exemplo.com" }))
        .await
        .expect("signin");
    let again = fails(
        &mut app,
        "cloud.signin",
        json!({ "email": "ana@exemplo.com" }),
    )
    .await;
    assert_eq!(again.reason, "rate_limited");
}
