//! Web Push (spec 28.8): the phone says where to tell it, the computer says
//! something waits, and the push service gets a signed call with nothing in it.

mod common;

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use botloft_cloud::{Pusher, new_key};
use common::mobile::Sock;
use common::{Server, server, server_pushing};
use p256::ecdsa::signature::Verifier;
use p256::ecdsa::{Signature, VerifyingKey};
use serde_json::{Value, json};

struct Hit {
    path: String,
    authorization: String,
    ttl: String,
    body: usize,
}

type Hits = Arc<Mutex<Vec<Hit>>>;

/// A push service that answers `status` and writes down what it was asked.
async fn service(status: Arc<AtomicU16>) -> (SocketAddr, Hits) {
    let hits = Hits::default();
    async fn take(
        State((hits, status)): State<(Hits, Arc<AtomicU16>)>,
        uri: axum::http::Uri,
        headers: HeaderMap,
        body: Bytes,
    ) -> StatusCode {
        let text = |name: &str| {
            headers
                .get(name)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default()
                .to_owned()
        };
        hits.lock().expect("hits").push(Hit {
            path: uri.path().to_owned(),
            authorization: text("authorization"),
            ttl: text("ttl"),
            body: body.len(),
        });
        StatusCode::from_u16(status.load(Ordering::SeqCst)).expect("status")
    }
    let app = Router::new()
        .route("/{*any}", post(take))
        .with_state((hits.clone(), status));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (address, hits)
}

fn pusher(gap_ms: u64) -> (Pusher, String) {
    let (private, public) = new_key().expect("key");
    let pusher = Pusher::new(
        &private,
        "mailto:ops@example.org",
        &["127.0.0.1".to_owned()],
        Duration::from_millis(gap_ms),
    )
    .expect("pusher");
    (pusher, public)
}

async fn subscribe(s: &Server, token: &str, endpoint: &str) -> StatusCode {
    s.call(
        "POST",
        "/v1/push/subscribe",
        Some(token),
        Some(json!({ "endpoint": endpoint })),
    )
    .await
    .status
}

async fn until(what: &str, check: impl Fn() -> bool) {
    for _ in 0..100 {
        if check() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("{what}");
}

#[tokio::test]
async fn a_new_key_makes_a_pusher_and_the_public_half_is_a_p256_point() {
    let (private, public) = new_key().expect("key");
    assert_eq!(B64.decode(&public).expect("b64").len(), 65);
    assert!(
        public.starts_with('B'),
        "an uncompressed point starts with 4"
    );
    assert!(Pusher::new(&private, "mailto:a@b.c", &[], Duration::from_secs(1)).is_ok());
    assert!(Pusher::new("not a key", "mailto:a@b.c", &[], Duration::from_secs(1)).is_err());
    assert!(
        Pusher::new(
            &B64.encode([0u8; 32]),
            "mailto:a@b.c",
            &[],
            Duration::from_secs(1)
        )
        .is_err()
    );
}

#[tokio::test]
async fn only_a_phone_subscribes_and_only_to_a_push_service_the_server_knows() {
    let (address, _) = service(Arc::new(AtomicU16::new(201))).await;
    let (pusher, public) = pusher(50);
    let s = server_pushing(pusher);
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    let endpoint = format!("http://{address}/push/abc");

    // The key a phone subscribes with is the server's public half.
    let key = s.authed("GET", "/v1/push/key", &phone.token).await;
    assert_eq!(key.json()["key"], public);
    // A computer has nothing to do here.
    assert_eq!(
        s.authed("GET", "/v1/push/key", &computer).await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        subscribe(&s, &computer, &endpoint).await,
        StatusCode::FORBIDDEN
    );

    // The server calls only the services it was told to: not another address,
    // not another scheme, not one with a password in it.
    for bad in [
        "https://evil.example.com/push/abc",
        "ftp://127.0.0.1/push",
        "http://evil.example.com/push",
        "https://user:secret@127.0.0.1/push",
        "https://127.0.0.1.evil.example.com/push",
        "not an address",
        "",
    ] {
        let reply = s
            .call(
                "POST",
                "/v1/push/subscribe",
                Some(&phone.token),
                Some(json!({ "endpoint": bad })),
            )
            .await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(reply.json()["reason"], "bad_endpoint");
    }
    assert_eq!(
        subscribe(&s, &phone.token, &endpoint).await,
        StatusCode::NO_CONTENT
    );
    // The same phone again replaces it, and it can be taken back.
    assert_eq!(
        subscribe(&s, &phone.token, &endpoint).await,
        StatusCode::NO_CONTENT
    );
    assert_eq!(
        s.authed("DELETE", "/v1/push/subscribe", &phone.token)
            .await
            .status,
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn a_server_without_push_says_so_and_the_default_services_are_the_browsers() {
    let s = server();
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    let key = s.authed("GET", "/v1/push/key", &phone.token).await;
    assert_eq!(key.status, StatusCode::NOT_FOUND);
    assert_eq!(key.json()["reason"], "no_push");
    assert_eq!(
        subscribe(&s, &phone.token, "https://fcm.googleapis.com/fcm/send/x").await,
        StatusCode::NOT_FOUND
    );

    // With no list of its own, a pusher takes the browsers' services and not a local address.
    let (private, _) = new_key().expect("key");
    let browsers =
        Pusher::new(&private, "mailto:a@b.c", &[], Duration::from_secs(1)).expect("pusher");
    for good in [
        "https://fcm.googleapis.com/fcm/send/abc",
        "https://updates.push.services.mozilla.com/wpush/v2/abc",
        "https://web.push.apple.com/abc",
        "https://wns2-par02p.notify.windows.com/w/?token=abc",
    ] {
        assert!(browsers.allows(good), "{good}");
    }
    for bad in [
        "http://fcm.googleapis.com/x",
        "https://googleapis.com.evil.org/x",
        "http://127.0.0.1:1/x",
    ] {
        assert!(!browsers.allows(bad), "{bad}");
    }
}

#[tokio::test]
async fn something_waiting_sends_the_phones_of_that_computer_a_signed_push_with_nothing_in_it() {
    let (address, hits) = service(Arc::new(AtomicU16::new(201))).await;
    let (pusher, public) = pusher(50);
    let s = server_pushing(pusher);
    let relay = s.listen().await;
    let a = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.clock.advance(61_000);
    let b = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let (phone_a, phone_a2, phone_b) = (
        s.pair(&a, 1).await,
        s.pair(&a, 2).await,
        s.pair(&b, 3).await,
    );
    subscribe(&s, &phone_a.token, &format!("http://{address}/a")).await;
    subscribe(&s, &phone_b.token, &format!("http://{address}/b")).await;
    // phone_a2 never subscribed: it is simply not told.
    let _ = &phone_a2;

    let (mut pc_a, _) = Sock::ready(relay, &a).await;
    pc_a.send(json!({ "t": "wake" })).await;
    until("a push", || !hits.lock().expect("hits").is_empty()).await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let seen = hits.lock().expect("hits");
    assert_eq!(seen.len(), 1, "only computer A's subscribed phone");
    let hit = &seen[0];
    assert_eq!(hit.path, "/a");
    assert_eq!((hit.ttl.as_str(), hit.body), ("3600", 0));

    // The call is signed with the key the phones subscribed with.
    let rest = hit
        .authorization
        .strip_prefix("vapid t=")
        .expect("a vapid header");
    let (jwt, k) = rest.split_once(", k=").expect("the key");
    assert_eq!(k, public);
    let parts: Vec<&str> = jwt.split('.').collect();
    assert_eq!(parts.len(), 3);
    let head: Value = serde_json::from_slice(&B64.decode(parts[0]).expect("head")).expect("json");
    assert_eq!(head, json!({ "typ": "JWT", "alg": "ES256" }));
    let claims: Value =
        serde_json::from_slice(&B64.decode(parts[1]).expect("claims")).expect("json");
    assert_eq!(claims["aud"], format!("http://{address}"));
    assert_eq!(claims["sub"], "mailto:ops@example.org");
    let now = i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_secs(),
    )
    .expect("secs");
    let exp = claims["exp"].as_i64().expect("exp");
    assert!(
        exp > now + 11 * 3600 && exp <= now + 13 * 3600,
        "twelve hours"
    );
    let verifying = VerifyingKey::from_sec1_bytes(&B64.decode(k).expect("key")).expect("point");
    let signature =
        Signature::from_slice(&B64.decode(parts[2]).expect("signature")).expect("signature");
    verifying
        .verify(format!("{}.{}", parts[0], parts[1]).as_bytes(), &signature)
        .expect("the signature is the server's");
}

#[tokio::test]
async fn notices_that_come_close_together_are_one_now_and_one_at_the_end_of_the_wait() {
    let (address, hits) = service(Arc::new(AtomicU16::new(201))).await;
    let (pusher, _) = pusher(400);
    let s = server_pushing(pusher);
    let relay = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    subscribe(&s, &phone.token, &format!("http://{address}/a")).await;
    let (mut pc, _) = Sock::ready(relay, &computer).await;

    for _ in 0..5 {
        pc.send(json!({ "t": "wake" })).await;
    }
    until("the first", || hits.lock().expect("hits").len() == 1).await;
    // The four that followed wait and go as one.
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(hits.lock().expect("hits").len(), 1);
    until("the second", || hits.lock().expect("hits").len() == 2).await;
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(hits.lock().expect("hits").len(), 2, "and no more");
}

#[tokio::test]
async fn a_push_service_that_says_gone_loses_the_phone_s_address() {
    let status = Arc::new(AtomicU16::new(410));
    let (address, hits) = service(Arc::clone(&status)).await;
    let (pusher, _) = pusher(50);
    let s = server_pushing(pusher);
    let relay = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    subscribe(&s, &phone.token, &format!("http://{address}/a")).await;
    let (mut pc, _) = Sock::ready(relay, &computer).await;

    pc.send(json!({ "t": "wake" })).await;
    until("the push", || hits.lock().expect("hits").len() == 1).await;
    tokio::time::sleep(Duration::from_millis(150)).await;
    // It is not asked again, even when the service would now accept.
    status.store(201, Ordering::SeqCst);
    pc.send(json!({ "t": "wake" })).await;
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(hits.lock().expect("hits").len(), 1);
}
