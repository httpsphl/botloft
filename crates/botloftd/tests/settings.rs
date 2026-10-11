//! The app's Settings (spec 11.2): read and changed over RPC, applied at
//! once. The test daemon has no scheduled task and no `config.toml`, so
//! only the values and the sign-in record change.

mod common;

use common::TestDaemon;
use serde_json::json;

#[tokio::test]
async fn settings_are_read_and_changed_one_at_a_time() {
    let t = TestDaemon::start().await;
    std::fs::create_dir_all(&t.paths.home).expect("home");
    let mut app = t.session().await;
    let settings = app.call("settings.get", json!(null)).await.expect("get");
    assert_eq!(
        settings,
        json!({ "startWithWindows": true, "keepAwake": true, "approvalWaitMinutes": 0, "defaultAgent": "claude", "showExperimentalAgents": false })
    );

    let changed = app
        .call("settings.update", json!({ "keepAwake": false }))
        .await
        .expect("update");
    assert_eq!(
        changed,
        json!({ "startWithWindows": true, "keepAwake": false, "approvalWaitMinutes": 0, "defaultAgent": "claude", "showExperimentalAgents": false })
    );
    assert!(!*t.daemon.settings.keep_awake().borrow());

    let changed = app
        .call("settings.update", json!({ "startWithWindows": false }))
        .await
        .expect("update");
    assert_eq!(
        changed,
        json!({ "startWithWindows": false, "keepAwake": false, "approvalWaitMinutes": 0, "defaultAgent": "claude", "showExperimentalAgents": false })
    );
    assert_eq!(
        app.call("settings.get", json!(null)).await.expect("get"),
        changed
    );
    if botloftd::platform::sign_in_id().is_some() {
        assert!(
            t.paths.home.join("signin").is_file(),
            "the daemon runs in this sign-in"
        );
    }
}

#[tokio::test]
async fn the_experimental_agents_are_switched_on_and_off_in_settings() {
    let t = TestDaemon::start().await;
    std::fs::create_dir_all(&t.paths.home).expect("home");
    let mut app = t.session().await;
    let agents = |status: serde_json::Value| status["enabledAgents"].clone();
    let status = app
        .call("system.status", json!(null))
        .await
        .expect("status");
    assert_eq!(agents(status), json!(["claude"]));

    let on = app
        .call("settings.update", json!({ "showExperimentalAgents": true }))
        .await
        .expect("on");
    assert_eq!(on["showExperimentalAgents"], json!(true));
    let status = app
        .call("system.status", json!(null))
        .await
        .expect("status");
    assert_eq!(agents(status), json!(["claude", "agy", "codex"]));

    app.call("settings.update", json!({ "defaultAgent": "codex" }))
        .await
        .expect("default");
    let off = app
        .call(
            "settings.update",
            json!({ "showExperimentalAgents": false }),
        )
        .await
        .expect("off");
    assert_eq!(off["showExperimentalAgents"], json!(false));
    assert_eq!(
        off["defaultAgent"],
        json!("claude"),
        "no default that cannot be chosen"
    );
    let status = app
        .call("system.status", json!(null))
        .await
        .expect("status");
    assert_eq!(agents(status), json!(["claude"]));
}
