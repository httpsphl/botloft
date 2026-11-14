//! The plan read from the owner's Claude account (spec 8.13), end to end,
//! against a server of the test's own in Anthropic's place: the login from
//! Claude Code's file, the windows that come back, and what is kept when
//! the account refuses the login.

mod common;

use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use botloft_core::protocol::AccountUsage;
use common::TestDaemon;
use serde_json::{Value, json};
use tokio::net::TcpListener;

/// The account's answer when the request carries the test's login, as the
/// account answered on 2026-11-14; a 401 otherwise.
async fn usage(headers: HeaderMap) -> Response {
    let login = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok());
    let beta = headers
        .get("anthropic-beta")
        .and_then(|value| value.to_str().ok());
    if login != Some("Bearer test-login") || beta != Some("oauth-2025-04-20") {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    axum::Json(json!({
        "five_hour": null,
        "seven_day": { "utilization": 80.0, "resets_at": "2026-11-15T13:00:00+00:00" },
        "limits": [
            { "kind": "weekly_all", "percent": 80, "resets_at": "2026-11-15T13:00:00+00:00",
              "scope": null },
            { "kind": "weekly_scoped", "percent": 30, "resets_at": "2026-11-15T13:00:00+00:00",
              "scope": { "model": { "display_name": "Fable" } } },
        ],
    }))
    .into_response()
}

/// The account's place, and Claude Code's folder with `login` in it.
async fn account(t: &TestDaemon, login: &str) -> tempfile::TempDir {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
    let addr = listener.local_addr().expect("addr");
    tokio::spawn(async move {
        axum::serve(listener, Router::new().route("/usage", get(usage)))
            .await
            .expect("serve");
    });
    let folder = tempfile::tempdir().expect("folder");
    let expires = t.daemon.clock.now_ms() + 3_600_000;
    let file = json!({ "claudeAiOauth": { "accessToken": login, "expiresAt": expires } });
    std::fs::write(folder.path().join(".credentials.json"), file.to_string()).expect("login");
    t.daemon
        .plan_reader
        .point_at(&format!("http://{addr}/usage"), folder.path());
    tokio::spawn(botloftd::plan_usage::run(Arc::clone(&t.daemon)));
    folder
}

/// The usage `system.status` gives once `done` says it is in.
async fn status_until(
    app: &mut common::Client,
    done: impl Fn(&Value) -> bool,
) -> Option<AccountUsage> {
    for _ in 0..200 {
        let status = app.call("system.status", json!({})).await.expect("status");
        if done(&status["usage"]) {
            return serde_json::from_value(status["usage"].clone()).expect("usage");
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    None
}

#[tokio::test]
async fn the_plan_comes_from_the_account_with_each_models_week() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let _folder = account(&t, "test-login").await;
    let usage = status_until(&mut app, |usage| usage["fromAccount"] == true)
        .await
        .expect("read from the account");
    assert_eq!(usage.windows.len(), 2);
    assert_eq!(usage.windows[0].name, "seven_day");
    assert!((usage.windows[0].utilization - 0.8).abs() < 1e-9);
    assert_eq!(usage.windows[1].model.as_deref(), Some("Fable"));
    assert_eq!(usage.status, "allowed");
}

#[tokio::test]
async fn a_refused_login_leaves_the_plan_as_the_agents_saw_it() {
    let t = TestDaemon::start().await;
    let mut app = t.session().await;
    let _folder = account(&t, "old-login").await;
    // Long enough for the reading to have been tried and refused.
    let read = status_until(&mut app, |usage| usage["fromAccount"] == true).await;
    assert!(read.is_none(), "nothing read with a refused login");
    let status = app.call("system.status", json!({})).await.expect("status");
    assert_eq!(status["usage"], Value::Null);
}
