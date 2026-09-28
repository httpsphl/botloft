//! Connection rules of `/rpc`: authentication, origins and error framing.

mod common;

use common::{Client, TOKEN, TestDaemon};
use serde_json::json;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::tungstenite;

const NOT_AUTHENTICATED: i64 = -32001;

#[tokio::test]
async fn health_answers_without_authentication() {
    let daemon = TestDaemon::start().await;
    let mut stream = tokio::net::TcpStream::connect(daemon.addr)
        .await
        .expect("connect");
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .await
        .expect("write");
    let mut response = String::new();
    stream.read_to_string(&mut response).await.expect("read");
    assert!(response.starts_with("HTTP/1.1 200"), "{response}");
    assert!(response.contains(r#""status":"ok""#), "{response}");
    assert!(response.contains(r#""protocol":2"#), "{response}");
}

#[tokio::test]
async fn hello_returns_the_daemon_version_and_protocol() {
    let daemon = TestDaemon::start().await;
    let mut client = daemon.client().await;
    let result = client.hello(TOKEN).await.expect("hello");
    assert_eq!(result["protocol"], 2);
    assert_eq!(result["daemonVersion"], env!("CARGO_PKG_VERSION"));
}

#[tokio::test]
async fn any_method_before_hello_is_refused_and_closes_the_connection() {
    let daemon = TestDaemon::start().await;
    let mut client = daemon.client().await;
    let err = client
        .call("crews.list", json!(null))
        .await
        .expect_err("refused");
    assert_eq!(err.code, NOT_AUTHENTICATED);
    assert!(client.closed().await);
}

#[tokio::test]
async fn a_wrong_token_is_refused_and_closes_the_connection() {
    let daemon = TestDaemon::start().await;
    let mut client = daemon.client().await;
    let err = client.hello("not-the-token").await.expect_err("refused");
    assert_eq!(err.code, NOT_AUTHENTICATED);
    assert!(client.closed().await);
}

#[tokio::test]
async fn another_protocol_version_is_refused() {
    let daemon = TestDaemon::start().await;
    let mut client = daemon.client().await;
    let err = client
        .call(
            "session.hello",
            json!({ "token": TOKEN, "client": { "name": "t", "version": "0" }, "protocol": 1 }),
        )
        .await
        .expect_err("refused");
    assert_eq!(err.code, -32004);
    assert!(client.closed().await);
}

#[tokio::test]
async fn browser_origins_other_than_the_app_are_refused() {
    let daemon = TestDaemon::start().await;
    let refused = Client::connect(&daemon.url(), Some("https://evil.example")).await;
    match refused {
        Err(tungstenite::Error::Http(response)) => assert_eq!(response.status(), 403),
        Err(other) => panic!("expected HTTP 403, got {other}"),
        Ok(_) => panic!("a foreign origin must be refused"),
    }
    for origin in [
        "http://tauri.localhost",
        "tauri://localhost",
        "http://localhost:1420",
    ] {
        let mut client = Client::connect(&daemon.url(), Some(origin))
            .await
            .unwrap_or_else(|err| panic!("{origin} refused: {err}"));
        client.hello(TOKEN).await.expect("hello");
    }
}

#[tokio::test]
async fn errors_after_hello_keep_the_connection_open() {
    let daemon = TestDaemon::start().await;
    let mut client = daemon.session().await;

    let err = client
        .call("crews.explode", json!(null))
        .await
        .expect_err("unknown");
    assert_eq!(err.code, -32601);
    let err = client.hello(TOKEN).await.expect_err("second hello");
    assert_eq!(err.code, -32003);
    let err = client
        .call("crews.create", json!({ "title": "x" }))
        .await
        .expect_err("bad params");
    assert_eq!(err.code, -32602);
    let err = client
        .call("bots.list", json!({ "crewId": "not-an-id" }))
        .await
        .expect_err("bad id");
    assert_eq!(err.code, -32602);

    client.send_raw("{ not json").await;
    let frame = client.recv().await.expect("parse error response");
    assert_eq!(frame["error"]["code"], -32700);
    assert_eq!(frame["id"], serde_json::Value::Null);

    let status = client
        .call("system.status", json!(null))
        .await
        .expect("still open");
    assert_eq!(status["protocol"], 2);
    assert!(status["uptimeMs"].as_i64().is_some());
    assert_eq!(status["claudeVersion"], serde_json::Value::Null);
}
