//! The copies an account keeps (spec 27.4).

mod common;

use axum::http::{StatusCode, header};
use common::{server, server_with, sha};

fn bytes(n: usize, seed: u8) -> Vec<u8> {
    (0..n).map(|i| (i as u8).wrapping_add(seed)).collect()
}

fn id_of(reply: &common::Reply) -> String {
    reply.json()["id"].as_str().expect("id").to_owned()
}

#[tokio::test]
async fn a_copy_goes_up_is_listed_and_comes_back_as_it_was() {
    let s = server();
    let token = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let data = bytes(600, 1);
    let put = s.put_copy(&token, &data).await;
    assert_eq!(put.status, StatusCode::CREATED, "{}", put.body);
    assert_eq!(put.json()["size"], 600);
    let id = id_of(&put);

    let list = s.authed("GET", "/v1/copies", &token).await.json();
    assert_eq!(list["copies"].as_array().expect("copies").len(), 1);
    assert_eq!(list["copies"][0]["id"], id.as_str());
    assert_eq!(list["copies"][0]["size"], 600);

    let (status, headers, body) = s.fetch(&format!("/v1/copies/{id}"), &token, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, data);
    assert_eq!(headers["x-botloft-sha256"], sha(&data).as_str());
    assert_eq!(headers[header::ACCEPT_RANGES], "bytes");

    let me = s.authed("GET", "/v1/me", &token).await.json();
    assert_eq!(me["used"], 600);
}

#[tokio::test]
async fn a_wrong_or_cut_upload_leaves_nothing() {
    let s = server();
    let token = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let data = bytes(300, 2);

    let wrong = s
        .put_copy_with(&token, &data, Some(&sha(b"other")), Some(300))
        .await;
    assert_eq!(wrong.status, StatusCode::BAD_REQUEST);
    assert_eq!(wrong.json()["reason"], "bad_hash");
    let none = s.put_copy_with(&token, &data, None, Some(300)).await;
    assert_eq!(none.json()["reason"], "bad_hash");
    let cut = s
        .put_copy_with(&token, &data[..100], Some(&sha(&data)), Some(300))
        .await;
    assert_eq!(cut.json()["reason"], "bad_length");
    let long = s
        .put_copy_with(&token, &data, Some(&sha(&data)), Some(100))
        .await;
    assert_eq!(long.json()["reason"], "bad_length");
    let unsaid = s
        .put_copy_with(&token, &data, Some(&sha(&data)), None)
        .await;
    assert_eq!(unsaid.status, StatusCode::LENGTH_REQUIRED);

    let list = s.authed("GET", "/v1/copies", &token).await.json();
    assert!(list["copies"].as_array().expect("copies").is_empty());
}

#[tokio::test]
async fn a_copy_cannot_be_bigger_than_allowed_nor_overfill_the_account() {
    let s = server_with(|c| {
        c.max_copy_bytes = 1000;
        c.quota_bytes = 2500;
        c.keep = 3;
    });
    let token = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let big = s.put_copy(&token, &bytes(1001, 0)).await;
    assert_eq!(big.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(big.json()["reason"], "too_big");

    let first = id_of(&s.put_copy(&token, &bytes(900, 1)).await);
    assert_eq!(
        s.put_copy(&token, &bytes(900, 2)).await.status,
        StatusCode::CREATED
    );
    let full = s.put_copy(&token, &bytes(900, 3)).await;
    assert_eq!(full.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(full.json()["reason"], "quota");

    // Deleting one makes room.
    let gone = s
        .authed("DELETE", &format!("/v1/copies/{first}"), &token)
        .await;
    assert_eq!(gone.status, StatusCode::NO_CONTENT);
    assert_eq!(
        s.put_copy(&token, &bytes(900, 3)).await.status,
        StatusCode::CREATED
    );
}

#[tokio::test]
async fn only_the_newest_few_are_kept_and_the_oldest_go_after_the_new_one() {
    let s = server_with(|c| c.keep = 2);
    let token = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let a = id_of(&s.put_copy(&token, &bytes(100, 1)).await);
    let b = id_of(&s.put_copy(&token, &bytes(100, 2)).await);

    // A failed upload does not push anything out.
    let bad = s
        .put_copy_with(&token, &bytes(100, 3), Some(&sha(b"x")), Some(100))
        .await;
    assert_eq!(bad.status, StatusCode::BAD_REQUEST);
    assert!(s.store.read(&format!("1/{a}"), None).await.is_ok());

    let c = id_of(&s.put_copy(&token, &bytes(100, 3)).await);
    let list = s.authed("GET", "/v1/copies", &token).await.json();
    let ids: Vec<&str> = list["copies"]
        .as_array()
        .expect("copies")
        .iter()
        .map(|copy| copy["id"].as_str().expect("id"))
        .collect();
    assert_eq!(ids, [c.as_str(), b.as_str()]);
    assert!(s.store.read(&format!("1/{a}"), None).await.is_err());
    assert!(s.store.read(&format!("1/{b}"), None).await.is_ok());
    assert!(s.store.read(&format!("1/{c}"), None).await.is_ok());
}

#[tokio::test]
async fn the_copy_that_will_be_pushed_out_does_not_count_against_the_room() {
    let s = server_with(|c| {
        c.max_copy_bytes = 1000;
        c.quota_bytes = 2000;
        c.keep = 2;
    });
    let token = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    for seed in 1..=3 {
        let put = s.put_copy(&token, &bytes(900, seed)).await;
        assert_eq!(put.status, StatusCode::CREATED, "{seed}: {}", put.body);
    }
    let me = s.authed("GET", "/v1/me", &token).await.json();
    assert_eq!(me["used"], 1800);
}

#[tokio::test]
async fn part_of_a_copy_can_be_asked_for() {
    let s = server();
    let token = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let data = bytes(100, 7);
    let path = format!("/v1/copies/{}", id_of(&s.put_copy(&token, &data).await));

    let (status, headers, body) = s.fetch(&path, &token, Some("bytes=10-19")).await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    assert_eq!(headers[header::CONTENT_RANGE], "bytes 10-19/100");
    assert_eq!(headers[header::CONTENT_LENGTH], "10");
    assert_eq!(body, data[10..20]);

    let (_, headers, body) = s.fetch(&path, &token, Some("bytes=-5")).await;
    assert_eq!(headers[header::CONTENT_RANGE], "bytes 95-99/100");
    assert_eq!(body, data[95..]);

    let (status, _, _) = s.fetch(&path, &token, Some("bytes=100-")).await;
    assert_eq!(status, StatusCode::RANGE_NOT_SATISFIABLE);
}

#[tokio::test]
async fn one_account_never_sees_or_removes_the_copies_of_another() {
    let s = server();
    let ana = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let bia = s.sign_in("bia@exemplo.com", "2.2.2.2").await;
    let id = id_of(&s.put_copy(&ana, &bytes(50, 1)).await);

    let list = s.authed("GET", "/v1/copies", &bia).await.json();
    assert!(list["copies"].as_array().expect("copies").is_empty());
    let (status, _, _) = s.fetch(&format!("/v1/copies/{id}"), &bia, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let removed = s.authed("DELETE", &format!("/v1/copies/{id}"), &bia).await;
    assert_eq!(removed.status, StatusCode::NOT_FOUND);
    let (status, _, _) = s.fetch(&format!("/v1/copies/{id}"), &ana, None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn deleting_a_copy_removes_its_bytes_and_without_a_token_nothing_opens() {
    let s = server();
    let token = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let id = id_of(&s.put_copy(&token, &bytes(80, 1)).await);
    let removed = s
        .authed("DELETE", &format!("/v1/copies/{id}"), &token)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert!(s.store.read(&format!("1/{id}"), None).await.is_err());
    let (status, _, _) = s.fetch(&format!("/v1/copies/{id}"), &token, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(s.authed("GET", "/v1/me", &token).await.json()["used"], 0);

    assert_eq!(s.get("/v1/copies").await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        s.put_copy("made-up", &bytes(10, 1)).await.status,
        StatusCode::UNAUTHORIZED
    );
}
