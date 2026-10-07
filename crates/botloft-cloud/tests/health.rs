//! The way a proxy or Docker sees that the server is up (spec 27.2).

mod common;

use axum::http::StatusCode;
use common::server;

#[tokio::test]
async fn health_answers_without_a_token_and_says_nothing_about_anyone() {
    let s = server();
    let reply = s.get("/health").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, "ok");
    // And a call that needs a sign-in still says so.
    assert_eq!(s.get("/v1/me").await.status, StatusCode::UNAUTHORIZED);
}
