//! Deleting an account (spec 27.3).

mod common;

use axum::http::StatusCode;
use common::server;

fn bytes(n: usize) -> Vec<u8> {
    (0..n).map(|i| i as u8).collect()
}

#[tokio::test]
async fn the_link_asks_and_only_the_button_deletes_everything() {
    let s = server();
    let ana = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.clock.advance(61_000);
    let other_device = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let bia = s.sign_in("bia@exemplo.com", "2.2.2.2").await;
    let id = s.put_copy(&ana, &bytes(70)).await.json()["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let bia_copy = s.put_copy(&bia, &bytes(40)).await.json()["id"]
        .as_str()
        .expect("id")
        .to_owned();

    // Asking only sends the e-mail.
    let asked = s.authed("POST", "/v1/account/delete", &ana).await;
    assert_eq!(asked.status, StatusCode::ACCEPTED);
    let mail = s.outbox.sent().pop().expect("an e-mail");
    assert_eq!(mail.to, "ana@exemplo.com");
    let (path, code) = s.last_link();
    assert_eq!(s.authed("GET", "/v1/me", &ana).await.status, StatusCode::OK);

    // Opening the link shows the button, and still deletes nothing.
    let page = s.get(&path).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.body.contains("<form") && page.body.contains("ana@exemplo.com"));
    assert_eq!(s.authed("GET", "/v1/me", &ana).await.status, StatusCode::OK);

    // The button deletes the account, its devices and the bytes of its copies.
    let done = s.post_delete(&code).await;
    assert_eq!(done.status, StatusCode::OK);
    assert_eq!(
        s.authed("GET", "/v1/me", &ana).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        s.authed("GET", "/v1/me", &other_device).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert!(s.store.read(&format!("1/{id}"), None).await.is_err());

    // Bia is untouched, and the link works once.
    assert_eq!(s.authed("GET", "/v1/me", &bia).await.status, StatusCode::OK);
    assert!(s.store.read(&format!("2/{bia_copy}"), None).await.is_ok());
    assert_eq!(s.post_delete(&code).await.status, StatusCode::GONE);
    assert_eq!(s.get(&path).await.status, StatusCode::GONE);
}

#[tokio::test]
async fn the_same_address_starts_over_with_nothing_after_it_was_deleted() {
    let s = server();
    let ana = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.put_copy(&ana, &bytes(70)).await;
    s.authed("POST", "/v1/account/delete", &ana).await;
    let (_, code) = s.last_link();
    s.post_delete(&code).await;

    s.clock.advance(61_000);
    let again = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let me = s.authed("GET", "/v1/me", &again).await.json();
    assert_eq!(me["used"], 0);
    let list = s.authed("GET", "/v1/copies", &again).await.json();
    assert!(list["copies"].as_array().expect("copies").is_empty());
}

#[tokio::test]
async fn the_link_expires_and_asking_again_too_soon_is_held() {
    let s = server();
    let ana = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    assert_eq!(
        s.authed("POST", "/v1/account/delete", &ana).await.status,
        StatusCode::ACCEPTED
    );
    let held = s.authed("POST", "/v1/account/delete", &ana).await;
    assert_eq!(held.status, StatusCode::TOO_MANY_REQUESTS);

    let (path, code) = s.last_link();
    s.clock.advance(601_000);
    assert_eq!(s.get(&path).await.status, StatusCode::GONE);
    assert_eq!(s.post_delete(&code).await.status, StatusCode::GONE);
    assert_eq!(s.authed("GET", "/v1/me", &ana).await.status, StatusCode::OK);
}

#[tokio::test]
async fn without_a_token_nobody_can_ask() {
    let s = server();
    assert_eq!(
        s.authed("POST", "/v1/account/delete", "made-up")
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}
