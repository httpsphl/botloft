//! Connecting a phone (spec 28.3) and what a phone's token may reach (28.2).

mod common;

use axum::http::StatusCode;
use common::mobile::{key, pair_id, proof};
use common::server;
use serde_json::json;

#[tokio::test]
async fn a_phone_joins_the_computer_accepts_and_the_phone_collects_its_token_once() {
    let s = server();
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let id = pair_id(1);
    let path = format!("/v1/pairings/{id}");

    let opened = s
        .call(
            "POST",
            "/v1/pairings",
            Some(&computer),
            Some(json!({ "id": id })),
        )
        .await;
    assert_eq!(opened.status, StatusCode::CREATED);
    assert_eq!(opened.json()["expires_in"], 300);
    assert_eq!(
        s.call("GET", &path, Some(&computer), None).await.json()["status"],
        "waiting"
    );

    // The phone has no token: it joins with its key and proof, and keeps the
    // secret that lets it collect the answer.
    let joined = s
        .call(
            "POST",
            &format!("{path}/join"),
            None,
            Some(json!({
                "phone_pub": key('P'), "device_name": "Celular da Ana", "proof": proof('p'),
            })),
        )
        .await;
    assert_eq!(joined.status, StatusCode::ACCEPTED);
    let poll = joined.json()["poll"].as_str().expect("poll").to_owned();

    // The computer sees who joined, exactly as sent.
    let seen = s.call("GET", &path, Some(&computer), None).await.json();
    assert_eq!(seen["status"], "joined");
    assert_eq!(seen["device_name"], "Celular da Ana");
    assert_eq!(seen["phone_pub"], key('P'));
    assert_eq!(seen["proof"], proof('p'));

    // Nothing for the phone until the computer accepts.
    let result = format!("{path}/result");
    assert_eq!(
        s.call("GET", &result, Some(&poll), None).await.json()["status"],
        "pending"
    );
    let accepted = s
        .call(
            "POST",
            &format!("{path}/accept"),
            Some(&computer),
            Some(json!({ "daemon_pub": key('D'), "proof2": proof('d') })),
        )
        .await;
    assert_eq!(accepted.status, StatusCode::NO_CONTENT);

    let got = s.call("GET", &result, Some(&poll), None).await.json();
    assert_eq!(got["status"], "approved");
    assert_eq!(got["daemon_pub"], key('D'));
    assert_eq!(got["proof2"], proof('d'));
    let token = got["token"].as_str().expect("token").to_owned();
    let device = got["device"].as_str().expect("device").to_owned();
    // Once: the pairing is gone and the token is not handed out again.
    assert_eq!(
        s.call("GET", &result, Some(&poll), None).await.json()["status"],
        "expired"
    );

    // The phone is a device of the account, tied to the computer that opened it.
    let me = s.authed("GET", "/v1/me", &token).await.json();
    assert_eq!(me["kind"], "phone");
    assert_eq!(me["device"], device);
    assert_eq!(me["email"], "ana@exemplo.com");
    assert!(me.get("devices").is_none() && me.get("used").is_none());
    let mine = s.authed("GET", "/v1/me", &computer).await.json();
    let devices = mine["devices"].as_array().expect("devices");
    assert_eq!(devices.len(), 2);
    let phone = devices.iter().find(|d| d["id"] == device).expect("phone");
    assert_eq!(phone["kind"], "phone");
    assert_eq!(phone["name"], "Celular da Ana");
    assert_eq!(me["peer"], phone["peer"]);
    assert!(
        devices
            .iter()
            .any(|d| d["kind"] == "computer" && d["peer"].is_null())
    );
}

#[tokio::test]
async fn a_phone_token_reaches_the_relay_and_nothing_that_belongs_to_a_computer() {
    let s = server();
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;

    for (method, path) in [
        ("GET", "/v1/copies"),
        ("GET", "/v1/copies/abc"),
        ("DELETE", "/v1/copies/abc"),
        ("POST", "/v1/account/delete"),
        ("DELETE", "/v1/devices/anything"),
        ("GET", "/v1/pairings/anything"),
    ] {
        let reply = s.authed(method, path, &phone.token).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN, "{method} {path}");
        assert_eq!(reply.json()["reason"], "wrong_device");
    }
    let opened = s
        .call(
            "POST",
            "/v1/pairings",
            Some(&phone.token),
            Some(json!({ "id": pair_id(9) })),
        )
        .await;
    assert_eq!(opened.status, StatusCode::FORBIDDEN);
    let put = s.put_copy(&phone.token, b"sealed").await;
    assert_eq!(put.status, StatusCode::FORBIDDEN);
    assert_eq!(
        s.authed("GET", "/v1/me", &phone.token).await.status,
        StatusCode::OK
    );

    // It can disconnect itself, and then its token is dead.
    assert_eq!(
        s.authed("POST", "/v1/logout", &phone.token).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        s.authed("GET", "/v1/me", &phone.token).await.status,
        StatusCode::UNAUTHORIZED
    );
    // The computer is still signed in.
    assert_eq!(
        s.authed("GET", "/v1/me", &computer).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn a_pairing_is_used_once_runs_out_and_checks_what_it_is_given() {
    let s = server();
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let id = pair_id(1);
    let join = format!("/v1/pairings/{id}/join");
    let body = json!({ "phone_pub": key('P'), "device_name": "X", "proof": proof('p') });
    s.call(
        "POST",
        "/v1/pairings",
        Some(&computer),
        Some(json!({ "id": id })),
    )
    .await;

    // Bad shapes are refused before anything is stored.
    for bad in [
        json!({ "phone_pub": "has space", "device_name": "X", "proof": proof('p') }),
        json!({ "phone_pub": key('P'), "device_name": "X", "proof": "short" }),
        json!({ "phone_pub": "", "device_name": "X", "proof": proof('p') }),
    ] {
        let reply = s.call("POST", &join, None, Some(bad)).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST);
        assert_eq!(reply.json()["reason"], "bad_pairing");
    }
    let short = s
        .call(
            "POST",
            "/v1/pairings",
            Some(&computer),
            Some(json!({ "id": "short" })),
        )
        .await;
    assert_eq!(short.status, StatusCode::BAD_REQUEST);

    // The first phone to join gets it; a second one finds it used.
    assert_eq!(
        s.call("POST", &join, None, Some(body.clone())).await.status,
        StatusCode::ACCEPTED
    );
    let again = s.call("POST", &join, None, Some(body.clone())).await;
    assert_eq!(again.status, StatusCode::GONE);
    assert_eq!(again.json()["reason"], "pair_expired");

    // Without the poll secret of the phone that joined, there is no token,
    // even after the computer accepted.
    s.call(
        "POST",
        &format!("/v1/pairings/{id}/accept"),
        Some(&computer),
        Some(json!({ "daemon_pub": key('D'), "proof2": proof('d') })),
    )
    .await;
    let stolen = s
        .call(
            "GET",
            &format!("/v1/pairings/{id}/result"),
            Some("not-the-secret"),
            None,
        )
        .await;
    assert_eq!(stolen.json()["status"], "expired");
    assert!(stolen.json().get("token").is_none());
    let nobody = s
        .call("GET", &format!("/v1/pairings/{id}/result"), None, None)
        .await;
    assert_eq!(nobody.status, StatusCode::UNAUTHORIZED);

    // Five minutes later the code is dead, and the computer sees it so.
    let late = pair_id(2);
    s.call(
        "POST",
        "/v1/pairings",
        Some(&computer),
        Some(json!({ "id": late })),
    )
    .await;
    s.clock.advance(301_000);
    let expired = s
        .call(
            "POST",
            &format!("/v1/pairings/{late}/join"),
            None,
            Some(body),
        )
        .await;
    assert_eq!(expired.status, StatusCode::GONE);
    let seen = s
        .call(
            "GET",
            &format!("/v1/pairings/{late}"),
            Some(&computer),
            None,
        )
        .await;
    assert_eq!(seen.json()["status"], "expired");
}

#[tokio::test]
async fn a_computer_holds_three_open_codes_cancels_and_cannot_accept_what_nobody_joined() {
    let s = server();
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    for n in 1..=3 {
        let opened = s
            .call(
                "POST",
                "/v1/pairings",
                Some(&computer),
                Some(json!({ "id": pair_id(n) })),
            )
            .await;
        assert_eq!(opened.status, StatusCode::CREATED);
    }
    let fourth = s
        .call(
            "POST",
            "/v1/pairings",
            Some(&computer),
            Some(json!({ "id": pair_id(4) })),
        )
        .await;
    assert_eq!(fourth.status, StatusCode::TOO_MANY_REQUESTS);

    // Accepting before anyone joined has nothing to accept.
    let early = s
        .call(
            "POST",
            &format!("/v1/pairings/{}/accept", pair_id(1)),
            Some(&computer),
            Some(json!({ "daemon_pub": key('D'), "proof2": proof('d') })),
        )
        .await;
    assert_eq!(early.status, StatusCode::NOT_FOUND);

    // Cancelling frees the slot and kills the code.
    let path = format!("/v1/pairings/{}", pair_id(1));
    assert_eq!(
        s.call("DELETE", &path, Some(&computer), None).await.status,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        s.call("DELETE", &path, Some(&computer), None).await.status,
        StatusCode::NOT_FOUND
    );
    let join = s
        .call(
            "POST",
            &format!("{path}/join"),
            None,
            Some(json!({ "phone_pub": key('P'), "device_name": "X", "proof": proof('p') })),
        )
        .await;
    assert_eq!(join.status, StatusCode::GONE);
    let fourth = s
        .call(
            "POST",
            "/v1/pairings",
            Some(&computer),
            Some(json!({ "id": pair_id(4) })),
        )
        .await;
    assert_eq!(fourth.status, StatusCode::CREATED);
}

#[tokio::test]
async fn another_computer_cannot_see_or_accept_someone_elses_pairing() {
    let s = server();
    let a = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.clock.advance(61_000);
    let b = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let id = pair_id(1);
    s.call("POST", "/v1/pairings", Some(&a), Some(json!({ "id": id })))
        .await;
    s.call(
        "POST",
        &format!("/v1/pairings/{id}/join"),
        None,
        Some(json!({ "phone_pub": key('P'), "device_name": "X", "proof": proof('p') })),
    )
    .await;
    let path = format!("/v1/pairings/{id}");
    assert_eq!(
        s.call("GET", &path, Some(&b), None).await.json()["status"],
        "expired"
    );
    let accept = s
        .call(
            "POST",
            &format!("{path}/accept"),
            Some(&b),
            Some(json!({ "daemon_pub": key('D'), "proof2": proof('d') })),
        )
        .await;
    assert_eq!(accept.status, StatusCode::NOT_FOUND);
    assert_eq!(
        s.call("DELETE", &path, Some(&b), None).await.status,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn removing_a_phone_or_its_computer_ends_the_tokens() {
    let s = server();
    let a = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.clock.advance(61_000);
    let b = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let first = s.pair(&a, 1).await;
    let second = s.pair(&a, 2).await;

    // Disconnecting one phone from a computer leaves the other.
    let gone = s
        .authed("DELETE", &format!("/v1/devices/{}", first.device), &a)
        .await;
    assert_eq!(gone.status, StatusCode::NO_CONTENT);
    assert_eq!(
        s.authed("GET", "/v1/me", &first.token).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        s.authed("GET", "/v1/me", &second.token).await.status,
        StatusCode::OK
    );

    // Removing the computer takes the phones that belong to it, not another computer.
    let me = s.authed("GET", "/v1/me", &a).await.json();
    let a_id = me["devices"]
        .as_array()
        .expect("devices")
        .iter()
        .find(|d| d["current"] == true)
        .expect("this device")["id"]
        .as_str()
        .expect("id")
        .to_owned();
    let removed = s.authed("DELETE", &format!("/v1/devices/{a_id}"), &b).await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(
        s.authed("GET", "/v1/me", &a).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        s.authed("GET", "/v1/me", &second.token).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(s.authed("GET", "/v1/me", &b).await.status, StatusCode::OK);
}

#[tokio::test]
async fn deleting_the_account_takes_the_phones_too() {
    let s = server();
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    assert_eq!(
        s.authed("POST", "/v1/account/delete", &computer)
            .await
            .status,
        StatusCode::ACCEPTED
    );
    let (_, code) = s.last_link();
    assert_eq!(s.post_delete(&code).await.status, StatusCode::OK);
    assert_eq!(
        s.authed("GET", "/v1/me", &phone.token).await.status,
        StatusCode::UNAUTHORIZED
    );
}
