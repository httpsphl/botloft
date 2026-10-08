//! The relay (spec 28.4): a real server on `127.0.0.1` and real sockets.

mod common;

use common::mobile::{Sock, key, pair_id, proof};
use common::server;
use serde_json::json;

/// What a sealed message looks like to the server: text it must not touch.
const SEALED: &str = "AAECAwQF+/8=?not-base64 {\"even\":\"json\"} \u{1f525}";

#[tokio::test]
async fn the_first_frame_must_be_a_valid_hello() {
    let s = server();
    let address = s.listen().await;

    let mut wrong = Sock::open(address, "not-a-token").await;
    assert_eq!(wrong.next().await["reason"], "unauthorized");
    wrong.closed().await;

    let mut silent = Sock::connect(address).await;
    silent
        .send(json!({ "t": "msg", "seq": 1, "body": "x" }))
        .await;
    silent.closed().await;

    // A phone that disconnected has no way back in with the old token.
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    s.authed("POST", "/v1/logout", &phone.token).await;
    let mut gone = Sock::open(address, &phone.token).await;
    assert_eq!(gone.next().await["reason"], "unauthorized");
}

#[tokio::test]
async fn a_phone_and_its_computer_talk_and_see_each_other_come_and_go() {
    let s = server();
    let address = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;

    let (mut pc, ready) = Sock::ready(address, &computer).await;
    assert_eq!(ready["kind"], "computer");
    assert_eq!(
        ready["phones"],
        json!([{ "id": phone.device, "online": false }])
    );

    let (mut mobile, ready) = Sock::ready(address, &phone.token).await;
    assert_eq!(ready["kind"], "phone");
    assert_eq!(ready["online"], true);
    assert_eq!(
        pc.next().await,
        json!({ "t": "presence", "device": phone.device, "online": true })
    );

    // Each body arrives exactly as sent.
    mobile
        .send(json!({ "t": "msg", "seq": 1, "body": SEALED }))
        .await;
    assert_eq!(
        pc.next().await,
        json!({ "t": "msg", "from": phone.device, "seq": 1, "body": SEALED })
    );
    pc.send(json!({ "t": "msg", "to": phone.device, "seq": 1, "body": SEALED }))
        .await;
    assert_eq!(
        mobile.next().await,
        json!({ "t": "msg", "seq": 1, "body": SEALED })
    );

    // A computer cannot write to a device that is not one of its phones.
    pc.send(json!({ "t": "msg", "to": "dev_nobody", "seq": 2, "body": "x" }))
        .await;
    let refused = pc.next().await;
    assert_eq!(refused["reason"], "unknown_device");
    assert_eq!(refused["seq"], 2);

    drop(mobile);
    assert_eq!(
        pc.next().await,
        json!({ "t": "presence", "device": phone.device, "online": false })
    );
    // And the phone sees the computer leave.
    let (mut mobile, _) = Sock::ready(address, &phone.token).await;
    pc.next().await;
    drop(pc);
    assert_eq!(
        mobile.next().await,
        json!({ "t": "presence", "device": ready_peer(&s, &computer).await, "online": false })
    );
}

/// The id of the computer behind `token`.
async fn ready_peer(s: &common::Server, token: &str) -> String {
    let me = s.authed("GET", "/v1/me", token).await.json();
    me["devices"]
        .as_array()
        .expect("devices")
        .iter()
        .find(|d| d["current"] == true)
        .expect("this device")["id"]
        .as_str()
        .expect("id")
        .to_owned()
}

#[tokio::test]
async fn answers_wait_for_a_computer_that_is_away_until_it_acknowledges_them() {
    let s = server();
    let address = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;

    let (mut mobile, ready) = Sock::ready(address, &phone.token).await;
    assert_eq!(ready["online"], false);
    for seq in [1, 2, 3] {
        mobile
            .send(json!({ "t": "msg", "seq": seq, "body": format!("answer {seq}") }))
            .await;
    }
    // The same seq again is not queued twice.
    mobile
        .send(json!({ "t": "msg", "seq": 2, "body": "answer 2" }))
        .await;
    mobile.sync().await;

    // The computer comes back: ready, then everything, in order.
    let (mut pc, _) = Sock::ready(address, &computer).await;
    for seq in [1, 2, 3] {
        let frame = pc.next().await;
        assert_eq!(
            (frame["seq"].as_i64(), frame["from"].as_str()),
            (Some(seq), Some(phone.device.as_str()))
        );
        assert_eq!(frame["body"], format!("answer {seq}"));
    }
    pc.quiet().await;

    // It takes up to 2; the next time only 3 is left.
    pc.send(json!({ "t": "ack", "from": phone.device, "upto": 2 }))
        .await;
    pc.sync().await;
    drop(pc);
    let (mut pc, _) = Sock::ready(address, &computer).await;
    let frame = pc.next().await;
    assert_eq!(
        (frame["seq"].as_i64(), frame["body"].as_str()),
        (Some(3), Some("answer 3"))
    );
    pc.quiet().await;
    pc.send(json!({ "t": "ack", "from": phone.device, "upto": 3 }))
        .await;
    pc.sync().await;
    drop(pc);
    let (mut pc, _) = Sock::ready(address, &computer).await;
    pc.quiet().await;
}

#[tokio::test]
async fn a_message_nobody_took_in_an_hour_is_dropped_and_the_queue_has_a_limit() {
    let s = server();
    let address = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    let (mut mobile, _) = Sock::ready(address, &phone.token).await;

    mobile
        .send(json!({ "t": "msg", "seq": 1, "body": "old" }))
        .await;
    mobile.sync().await;
    s.clock.advance(3_600_001);
    let (mut pc, _) = Sock::ready(address, &computer).await;
    pc.quiet().await;
    drop(pc);

    // Fifty wait, the fifty-first is told so, and an acknowledgement makes room.
    for seq in 10..60 {
        mobile
            .send(json!({ "t": "msg", "seq": seq, "body": "x" }))
            .await;
    }
    mobile
        .send(json!({ "t": "msg", "seq": 60, "body": "x" }))
        .await;
    // (The computer coming and going above is told to the phone as presence.)
    let mut full = mobile.next().await;
    while full["t"] == "presence" {
        full = mobile.next().await;
    }
    assert_eq!(
        (full["reason"].as_str(), full["seq"].as_i64()),
        (Some("queue_full"), Some(60))
    );
    s.clock.advance(61_000);
    let (mut pc, _) = Sock::ready(address, &computer).await;
    for _ in 10..60 {
        pc.next().await;
    }
    pc.send(json!({ "t": "ack", "from": phone.device, "upto": 59 }))
        .await;
    pc.sync().await;
    mobile
        .send(json!({ "t": "msg", "seq": 61, "body": "room" }))
        .await;
    assert_eq!(pc.next().await["body"], "room");
}

#[tokio::test]
async fn disconnecting_a_phone_closes_its_socket_and_tells_the_computer() {
    let s = server();
    let address = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    let (mut pc, _) = Sock::ready(address, &computer).await;
    let (mut mobile, _) = Sock::ready(address, &phone.token).await;
    pc.next().await;

    s.authed(
        "DELETE",
        &format!("/v1/devices/{}", phone.device),
        &computer,
    )
    .await;
    mobile.closed().await;
    // The computer hears that the phone is revoked (and may see it go offline).
    let mut frames = [pc.next().await, pc.next().await];
    frames.sort_by_key(|frame| frame["t"].as_str().unwrap_or_default().to_owned());
    assert_eq!(
        frames[0],
        json!({ "t": "presence", "device": phone.device, "online": false })
    );
    assert_eq!(frames[1], json!({ "t": "revoked", "device": phone.device }));
    // The phone's own logout does the same.
    let again = s.pair(&computer, 2).await;
    let (mut second, _) = Sock::ready(address, &again.token).await;
    pc.next().await;
    s.authed("POST", "/v1/logout", &again.token).await;
    second.closed().await;
}

#[tokio::test]
async fn removing_a_computer_closes_it_and_its_phones() {
    let s = server();
    let address = s.listen().await;
    let a = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.clock.advance(61_000);
    let b = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&a, 1).await;
    let (mut pc, _) = Sock::ready(address, &a).await;
    let (mut mobile, _) = Sock::ready(address, &phone.token).await;
    pc.next().await;

    let a_id = ready_peer(&s, &a).await;
    s.authed("DELETE", &format!("/v1/devices/{a_id}"), &b).await;
    pc.closed().await;
    mobile.closed().await;
}

#[tokio::test]
async fn one_computers_phones_never_reach_another_computer() {
    let s = server();
    let address = s.listen().await;
    let a = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    s.clock.advance(61_000);
    let b = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone_a = s.pair(&a, 1).await;
    let phone_b = s.pair(&b, 2).await;
    let (mut pc_a, _) = Sock::ready(address, &a).await;
    let (mut pc_b, _) = Sock::ready(address, &b).await;
    let (mut mobile_a, _) = Sock::ready(address, &phone_a.token).await;
    let (mut mobile_b, _) = Sock::ready(address, &phone_b.token).await;
    pc_a.next().await;
    pc_b.next().await;

    mobile_b
        .send(json!({ "t": "msg", "seq": 1, "body": "for b" }))
        .await;
    assert_eq!(pc_b.next().await["body"], "for b");
    pc_a.quiet().await;

    // Computer A cannot write to computer B's phone, even knowing its id.
    pc_a.send(json!({ "t": "msg", "to": phone_b.device, "seq": 1, "body": "intruder" }))
        .await;
    assert_eq!(pc_a.next().await["reason"], "unknown_device");
    mobile_b.quiet().await;
    mobile_a.quiet().await;
}

#[tokio::test]
async fn a_device_sends_sixty_frames_a_minute_and_a_second_socket_replaces_the_first() {
    let s = server();
    let address = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let phone = s.pair(&computer, 1).await;
    let (mut mobile, _) = Sock::ready(address, &phone.token).await;
    for seq in 1..=60 {
        mobile
            .send(json!({ "t": "msg", "seq": seq, "body": "x" }))
            .await;
    }
    mobile
        .send(json!({ "t": "msg", "seq": 61, "body": "x" }))
        .await;
    // (Fifty queue; frames 51-60 are told the queue is full; the 61st the rate.)
    let mut reasons = Vec::new();
    for _ in 0..11 {
        reasons.push(mobile.next().await["reason"].as_str().map(str::to_owned));
    }
    assert_eq!(reasons.last(), Some(&Some("rate_limited".to_owned())));
    s.clock.advance(61_000);
    mobile.sync().await;

    let (mut again, _) = Sock::ready(address, &phone.token).await;
    mobile.closed().await;
    again.sync().await;
}

#[tokio::test]
async fn the_computer_hears_the_phones_id_when_it_collects_its_token() {
    let s = server();
    let address = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let (mut pc, _) = Sock::ready(address, &computer).await;
    let phone = s.pair(&computer, 3).await;
    // The join came first, then the phone collected its token.
    assert_eq!(
        pc.next().await,
        json!({ "t": "pairing", "id": phone.pairing })
    );
    assert_eq!(
        pc.next().await,
        json!({ "t": "paired", "pairing": phone.pairing, "device": phone.device })
    );
}

#[tokio::test]
async fn the_computer_hears_when_a_phone_joins() {
    let s = server();
    let address = s.listen().await;
    let computer = s.sign_in("ana@exemplo.com", "1.1.1.1").await;
    let (mut pc, _) = Sock::ready(address, &computer).await;
    let id = pair_id(7);
    s.call(
        "POST",
        "/v1/pairings",
        Some(&computer),
        Some(json!({ "id": id })),
    )
    .await;
    s.call(
        "POST",
        &format!("/v1/pairings/{id}/join"),
        None,
        Some(json!({ "phone_pub": key('P'), "device_name": "X", "proof": proof('p') })),
    )
    .await;
    assert_eq!(pc.next().await, json!({ "t": "pairing", "id": id }));
}
