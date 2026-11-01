//! The files a phone sends to its computer (spec 28.14): sealed on the
//! phone, kept here until the computer takes them, two hours at most.

mod common;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use botloft_cloud::MAX_FILE;
use common::{Reply, Server, server, sha};

fn bytes(n: usize, seed: u8) -> Vec<u8> {
    (0..n)
        .map(|i| (i as u8).wrapping_mul(7).wrapping_add(seed))
        .collect()
}

async fn put_file_with(
    s: &Server,
    token: &str,
    data: &[u8],
    hash: Option<&str>,
    length: Option<u64>,
) -> Reply {
    let mut request =
        Request::put("/v1/files").header(header::AUTHORIZATION, format!("Bearer {token}"));
    if let Some(hash) = hash {
        request = request.header("x-botloft-sha256", hash);
    }
    if let Some(length) = length {
        request = request.header(header::CONTENT_LENGTH, length);
    }
    s.send(request.body(Body::from(data.to_vec())).expect("request"))
        .await
}

async fn put_file(s: &Server, token: &str, data: &[u8]) -> Reply {
    put_file_with(s, token, data, Some(&sha(data)), Some(data.len() as u64)).await
}

fn id_of(reply: &Reply) -> String {
    reply.json()["id"].as_str().expect("id").to_owned()
}

/// A computer signed in, and a phone connected to it.
async fn computer_and_phone(s: &Server) -> (String, common::mobile::Phone) {
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    (computer, phone)
}

/// The first account of a new server is number 1.
fn kept(id: &str) -> String {
    format!("1/files/{id}")
}

#[tokio::test]
async fn a_file_from_the_phone_reaches_its_computer_once() {
    let s = server();
    let (computer, phone) = computer_and_phone(&s).await;
    let data = bytes(5000, 3);
    let before = s.clock.now();
    let put = put_file(&s, &phone.token, &data).await;
    assert_eq!(put.status, StatusCode::CREATED, "{}", put.body);
    assert_eq!(put.json()["size"], 5000);
    let expires = put.json()["expires"].as_i64().expect("expires");
    assert!((before..=s.clock.now()).contains(&(expires - 2 * 3_600_000)));
    let id = id_of(&put);

    let (status, headers, body) = s.fetch(&format!("/v1/files/{id}"), &computer, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, data);
    assert_eq!(headers["x-botloft-sha256"], sha(&data).as_str());

    let taken = s
        .authed("DELETE", &format!("/v1/files/{id}"), &computer)
        .await;
    assert_eq!(taken.status, StatusCode::NO_CONTENT);
    let (again, _, _) = s.fetch(&format!("/v1/files/{id}"), &computer, None).await;
    assert_eq!(again, StatusCode::NOT_FOUND);
    let twice = s
        .authed("DELETE", &format!("/v1/files/{id}"), &computer)
        .await;
    assert_eq!(twice.status, StatusCode::NOT_FOUND);
    assert!(s.store.read(&kept(&id), None).await.is_err());
}

#[tokio::test]
async fn only_the_phone_sends_and_only_its_computer_takes() {
    let s = server();
    let (computer, phone) = computer_and_phone(&s).await;
    let data = bytes(300, 1);

    let from_computer = put_file(&s, &computer, &data).await;
    assert_eq!(from_computer.status, StatusCode::FORBIDDEN);
    assert_eq!(from_computer.json()["reason"], "wrong_device");

    let id = id_of(&put_file(&s, &phone.token, &data).await);
    let path = format!("/v1/files/{id}");
    let (by_phone, _, _) = s.fetch(&path, &phone.token, None).await;
    assert_eq!(by_phone, StatusCode::FORBIDDEN);

    // Another computer of the same account, and a computer of another account.
    s.clock.advance(61_000);
    let other = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let stranger = s.sign_in("bia@exemplo.com", "2.2.2.2").await;
    for token in [&other, &stranger] {
        let (status, _, _) = s.fetch(&path, token, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        let removed = s.authed("DELETE", &path, token).await;
        assert_eq!(removed.status, StatusCode::NOT_FOUND);
    }
    let (status, _, _) = s.fetch(&path, &computer, None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn a_file_too_big_cut_or_wrong_is_refused() {
    let s = server();
    let (_, phone) = computer_and_phone(&s).await;
    let data = bytes(400, 2);

    let big = put_file_with(
        &s,
        &phone.token,
        &data,
        Some(&sha(&data)),
        Some(MAX_FILE + 1),
    )
    .await;
    assert_eq!(big.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(big.json()["reason"], "too_big");
    let wrong = put_file_with(&s, &phone.token, &data, Some(&sha(b"other")), Some(400)).await;
    assert_eq!(wrong.json()["reason"], "bad_hash");
    let cut = put_file_with(&s, &phone.token, &data[..100], Some(&sha(&data)), Some(400)).await;
    assert_eq!(cut.json()["reason"], "bad_length");
    let unsaid = put_file_with(&s, &phone.token, &data, Some(&sha(&data)), None).await;
    assert_eq!(unsaid.status, StatusCode::LENGTH_REQUIRED);
}

#[tokio::test]
async fn a_file_nobody_takes_runs_out_in_two_hours() {
    let s = server();
    let (computer, phone) = computer_and_phone(&s).await;
    let id = id_of(&put_file(&s, &phone.token, &bytes(200, 4)).await);
    assert!(s.store.read(&kept(&id), None).await.is_ok());

    s.clock.advance(2 * 3_600_000);
    let (status, _, _) = s.fetch(&format!("/v1/files/{id}"), &computer, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    // The next file from any phone sweeps the bytes away.
    let next = put_file(&s, &phone.token, &bytes(10, 5)).await;
    assert_eq!(next.status, StatusCode::CREATED, "{}", next.body);
    assert!(s.store.read(&kept(&id), None).await.is_err());
}

#[tokio::test]
async fn the_files_go_with_the_phone() {
    let s = server();
    let (computer, phone) = computer_and_phone(&s).await;
    let id = id_of(&put_file(&s, &phone.token, &bytes(200, 6)).await);
    let removed = s
        .authed(
            "DELETE",
            &format!("/v1/devices/{}", phone.device),
            &computer,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert!(s.store.read(&kept(&id), None).await.is_err());
    let (status, _, _) = s.fetch(&format!("/v1/files/{id}"), &computer, None).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_phone_leaves_twenty_files_waiting_at_most() {
    let s = server();
    let (computer, phone) = computer_and_phone(&s).await;
    let mut ids = Vec::new();
    for n in 0..20u8 {
        let put = put_file(&s, &phone.token, &bytes(50, n)).await;
        assert_eq!(put.status, StatusCode::CREATED, "{}", put.body);
        ids.push(id_of(&put));
    }
    let full = put_file(&s, &phone.token, &bytes(50, 99)).await;
    assert_eq!(full.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(full.json()["reason"], "queue_full");

    // Once the computer takes one, there is room again.
    let taken = s
        .authed("DELETE", &format!("/v1/files/{}", ids[0]), &computer)
        .await;
    assert_eq!(taken.status, StatusCode::NO_CONTENT);
    let room = put_file(&s, &phone.token, &bytes(50, 99)).await;
    assert_eq!(room.status, StatusCode::CREATED, "{}", room.body);
}
