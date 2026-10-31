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
        json!({ "startWithWindows": true, "keepAwake": true, "approvalWaitMinutes": 0, "sleepAfterMinutes": 0, "defaultAgent": "claude", "showExperimentalAgents": false })
    );

    let changed = app
        .call("settings.update", json!({ "keepAwake": false }))
        .await
        .expect("update");
    assert_eq!(
        changed,
        json!({ "startWithWindows": true, "keepAwake": false, "approvalWaitMinutes": 0, "sleepAfterMinutes": 0, "defaultAgent": "claude", "showExperimentalAgents": false })
    );
    assert!(!*t.daemon.settings.keep_awake().borrow());

    let changed = app
        .call("settings.update", json!({ "startWithWindows": false }))
        .await
        .expect("update");
    assert_eq!(
        changed,
        json!({ "startWithWindows": false, "keepAwake": false, "approvalWaitMinutes": 0, "sleepAfterMinutes": 0, "defaultAgent": "claude", "showExperimentalAgents": false })
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

    // How long agents stay idle before they sleep (spec 7.7): 0 is never,
    // and more than a day is refused.
    let changed = app
        .call("settings.update", json!({ "sleepAfterMinutes": 60 }))
        .await
        .expect("update");
    assert_eq!(changed["sleepAfterMinutes"], 60);
    assert_eq!(
        t.daemon.settings.sleep_after(),
        Some(std::time::Duration::from_secs(3600))
    );
    let refused = app
        .call("settings.update", json!({ "sleepAfterMinutes": 1441 }))
        .await;
    assert!(refused.is_err());
    app.call("settings.update", json!({ "sleepAfterMinutes": 0 }))
        .await
        .expect("never");
    assert_eq!(t.daemon.settings.sleep_after(), None);
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

#[tokio::test]
async fn the_owner_tells_where_an_agents_program_is() {
    let t = TestDaemon::start().await;
    std::fs::create_dir_all(&t.paths.home).expect("home");
    let mut app = t.session().await;
    app.call("settings.update", json!({ "showExperimentalAgents": true }))
        .await
        .expect("on");

    // A file that is not there is refused, and Claude Code has its own key.
    let missing = app
        .call(
            "agents.setPath",
            json!({ "agent": "codex", "path": "C:/nowhere/codex.exe" }),
        )
        .await;
    assert!(missing.is_err(), "no file, no path");
    let claude = app
        .call("agents.setPath", json!({ "agent": "claude", "path": "x" }))
        .await;
    assert!(claude.is_err());

    let program = t.paths.home.join("my-codex.exe");
    std::fs::write(&program, b"").expect("file");
    let check = app
        .call(
            "agents.setPath",
            json!({ "agent": "codex", "path": program.display().to_string() }),
        )
        .await
        .expect("set");
    assert_eq!(check["agent"], json!("codex"));
    assert_eq!(check["path"], json!(program.display().to_string()));
    assert_eq!(
        t.daemon
            .supervisor
            .program_setting(botloft_core::protocol::AgentKind::Codex),
        program.display().to_string()
    );

    // The status shows it at once, not after the minute of its cache.
    let status = app
        .call("system.status", json!(null))
        .await
        .expect("status");
    let codex = status["agentChecks"]
        .as_array()
        .expect("checks")
        .iter()
        .find(|check| check["agent"] == json!("codex"))
        .expect("codex check")
        .clone();
    assert_eq!(codex["path"], json!(program.display().to_string()));

    // Empty goes back to looking for it.
    app.call("agents.setPath", json!({ "agent": "codex", "path": "" }))
        .await
        .expect("clear");
    assert_eq!(
        t.daemon
            .supervisor
            .program_setting(botloft_core::protocol::AgentKind::Codex),
        ""
    );
}
