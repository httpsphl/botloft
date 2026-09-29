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
        json!({ "startWithWindows": true, "keepAwake": true })
    );

    let changed = app
        .call("settings.update", json!({ "keepAwake": false }))
        .await
        .expect("update");
    assert_eq!(
        changed,
        json!({ "startWithWindows": true, "keepAwake": false })
    );
    assert!(!*t.daemon.settings.keep_awake().borrow());

    let changed = app
        .call("settings.update", json!({ "startWithWindows": false }))
        .await
        .expect("update");
    assert_eq!(
        changed,
        json!({ "startWithWindows": false, "keepAwake": false })
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
