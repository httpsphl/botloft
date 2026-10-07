//! Signing in by e-mail link (spec 27.3).

mod common;

use axum::http::StatusCode;
use common::{server, start};
use serde_json::json;

#[tokio::test]
async fn the_link_signs_a_device_in_and_the_token_works_once_collected() {
    let s = server();
    let asked = s.ask("Ana@Exemplo.com", "1.1.1.1").await;
    assert_eq!(asked.status, StatusCode::ACCEPTED);
    assert_eq!(asked.json()["wait"], 600);
    let request = asked.json()["request"]
        .as_str()
        .expect("request")
        .to_owned();

    // The e-mail went to the cleaned address, in the language the app asked.
    let mails = s.outbox.sent();
    assert_eq!(mails.len(), 1);
    assert_eq!(mails[0].to, "ana@exemplo.com");
    assert!(mails[0].subject.contains("Botloft") && mails[0].body.contains("PC da Ana"));
    assert!(mails[0].body.contains("expira em 10 minutos"));

    // Nothing is approved by the e-mail alone, and opening the link only
    // shows the button: a mail scanner that opens it signs nobody in.
    let poll = format!("/v1/login/{request}");
    assert_eq!(s.get(&poll).await.json()["status"], "pending");
    let (path, code) = s.last_link();
    let page = s.get(&path).await;
    assert_eq!(page.status, StatusCode::OK);
    assert!(page.body.contains("<form") && page.body.contains("Entrar no Botloft"));
    assert_eq!(s.get(&poll).await.json()["status"], "pending");

    // The button approves, and the app collects the token once.
    let pressed = s.press(&code).await;
    assert_eq!(pressed.status, StatusCode::OK);
    assert!(pressed.body.contains("Pode voltar ao Botloft"));
    let got = s.get(&poll).await.json();
    assert_eq!(got["status"], "approved");
    assert_eq!(got["email"], "ana@exemplo.com");
    let token = got["token"].as_str().expect("token").to_owned();
    assert_eq!(s.get(&poll).await.json()["status"], "expired");

    let me = s.authed("GET", "/v1/me", &token).await.json();
    assert_eq!(me["email"], "ana@exemplo.com");
    assert_eq!(me["used"], 0);
    assert_eq!(me["devices"].as_array().expect("devices").len(), 1);
    assert_eq!(me["devices"][0]["name"], "PC da Ana");
    assert_eq!(me["devices"][0]["current"], true);

    // Signing out ends this token.
    assert_eq!(
        s.authed("POST", "/v1/logout", &token).await.status,
        StatusCode::NO_CONTENT
    );
    let again = s.authed("GET", "/v1/me", &token).await;
    assert_eq!(again.status, StatusCode::UNAUTHORIZED);
    assert_eq!(again.json()["reason"], "unauthorized");
}

#[tokio::test]
async fn a_known_address_and_a_new_one_get_the_same_answer() {
    let s = server();
    s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.clock.advance(61_000);
    let known = s.ask("ana@exemplo.com", "1.1.1.1").await;
    let new = s.ask("bia@exemplo.com", "1.1.1.1").await;
    assert_eq!(known.status, new.status);
    let keys = |reply: &common::Reply| {
        let mut keys: Vec<String> = reply
            .json()
            .as_object()
            .expect("object")
            .keys()
            .cloned()
            .collect();
        keys.sort();
        keys
    };
    assert_eq!(keys(&known), keys(&new));
    assert_eq!(s.outbox.sent().len(), 3);
}

#[tokio::test]
async fn a_link_works_once_and_not_after_ten_minutes() {
    let s = server();
    s.ask("ana@exemplo.com", "1.1.1.1").await;
    let (path, code) = s.last_link();
    assert_eq!(s.press(&code).await.status, StatusCode::OK);
    // Used already: the page and the button both say it is gone.
    assert_eq!(s.get(&path).await.status, StatusCode::GONE);
    assert_eq!(s.press(&code).await.status, StatusCode::GONE);

    s.clock.advance(61_000);
    let asked = s.ask("bia@exemplo.com", "1.1.1.1").await;
    let request = asked.json()["request"]
        .as_str()
        .expect("request")
        .to_owned();
    let (path, code) = s.last_link();
    s.clock.advance(601_000);
    let page = s.get(&path).await;
    assert_eq!(page.status, StatusCode::GONE);
    assert!(page.body.contains("Ask for a new one"));
    assert_eq!(s.press(&code).await.status, StatusCode::GONE);
    let polled = s.get(&format!("/v1/login/{request}")).await.json();
    assert_eq!(polled["status"], "expired");
}

#[tokio::test]
async fn a_request_that_never_existed_is_just_expired() {
    let s = server();
    let polled = s.get("/v1/login/not-a-request").await;
    assert_eq!(polled.status, StatusCode::OK);
    assert_eq!(polled.json()["status"], "expired");
}

#[tokio::test]
async fn another_e_mail_is_held_for_a_minute_and_an_hour_allows_five() {
    let s = server();
    assert_eq!(
        s.ask("ana@exemplo.com", "1.1.1.1").await.status,
        StatusCode::ACCEPTED
    );
    // A second e-mail to the same person within a minute is refused, not sent.
    let held = s.ask("ana@exemplo.com", "1.1.1.1").await;
    assert_eq!(held.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(held.json()["reason"], "rate_limited");
    assert!(held.json()["retry_after"].as_u64().expect("seconds") <= 60);
    assert_eq!(s.outbox.sent().len(), 1);

    for _ in 0..4 {
        s.clock.advance(61_000);
        assert_eq!(
            s.ask("ana@exemplo.com", "1.1.1.1").await.status,
            StatusCode::ACCEPTED
        );
    }
    s.clock.advance(61_000);
    let sixth = s.ask("ana@exemplo.com", "1.1.1.1").await;
    assert_eq!(sixth.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(s.outbox.sent().len(), 5);
}

#[tokio::test]
async fn one_address_cannot_flood_the_server_with_different_e_mails() {
    let s = server();
    for n in 0..20 {
        let sent = s.ask(&format!("p{n}@exemplo.com"), "9.9.9.9").await;
        assert_eq!(sent.status, StatusCode::ACCEPTED, "{n}");
    }
    assert_eq!(
        s.ask("p20@exemplo.com", "9.9.9.9").await.status,
        StatusCode::TOO_MANY_REQUESTS
    );
    // Someone else is not held back by it.
    assert_eq!(
        s.ask("p20@exemplo.com", "8.8.8.8").await.status,
        StatusCode::ACCEPTED
    );
}

#[tokio::test]
async fn bad_addresses_send_nothing() {
    let s = server();
    for bad in [
        "",
        "ana",
        "a b@x.com",
        "ana@x.com\r\nBcc: z@z.com",
        "<a>@x.com",
    ] {
        let reply = s.ask(bad, "1.1.1.1").await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{bad:?}");
        assert_eq!(reply.json()["reason"], "bad_email");
    }
    assert!(s.outbox.sent().is_empty());
}

#[tokio::test]
async fn a_mail_that_does_not_go_leaves_no_request_behind() {
    let s = start(botloft_cloud::Outbox::broken());
    let reply = s.ask("ana@exemplo.com", "1.1.1.1").await;
    assert_eq!(reply.status, StatusCode::BAD_GATEWAY);
    assert_eq!(reply.json()["reason"], "mail_failed");
}

#[tokio::test]
async fn the_device_name_cannot_become_markup_on_the_page() {
    let s = server();
    let body = json!({ "email": "ana@exemplo.com", "device_name": "<script>x()</script>", "locale": "en" });
    s.ask_with(body, "1.1.1.1").await;
    let (path, _) = s.last_link();
    let page = s.get(&path).await;
    assert!(page.body.contains("&lt;script&gt;x()&lt;/script&gt;"));
    assert!(!page.body.contains("<script>"));
}

#[tokio::test]
async fn a_device_can_remove_another_of_its_own_account_only() {
    let s = server();
    let first = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.clock.advance(61_000);
    let second = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let other = s.sign_in("bia@exemplo.com", "2.2.2.2").await;

    let me = s.authed("GET", "/v1/me", &first).await.json();
    let devices = me["devices"].as_array().expect("devices");
    assert_eq!(devices.len(), 2);
    let second_id = devices
        .iter()
        .find(|d| d["current"] == false)
        .map(|d| d["id"].as_str().expect("id").to_owned())
        .expect("the other device");

    // Bia cannot touch Ana's device, and Ana can.
    let denied = s
        .authed("DELETE", &format!("/v1/devices/{second_id}"), &other)
        .await;
    assert_eq!(denied.status, StatusCode::NOT_FOUND);
    let removed = s
        .authed("DELETE", &format!("/v1/devices/{second_id}"), &first)
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert_eq!(
        s.authed("GET", "/v1/me", &second).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        s.authed("GET", "/v1/me", &first).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn without_a_token_nothing_opens() {
    let s = server();
    assert_eq!(s.get("/v1/me").await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        s.authed("GET", "/v1/me", "made-up").await.status,
        StatusCode::UNAUTHORIZED
    );
}
