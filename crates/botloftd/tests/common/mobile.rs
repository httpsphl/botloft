//! A phone for the tests (spec 28): what the PWA does, in Rust, against the
//! real account server and a daemon in this process.

use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use botloft_core::protocol::{FromPhone, ToPhone};
use botloftd::mobile::seal::{self, Dir, Keypair, Keys};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use super::cloud_server::{CloudServer, cloud_server, sign_in};
use super::{Client, TestDaemon};

const WAIT: Duration = Duration::from_secs(10);

/// A phone that scanned the code and joined it.
pub struct Joined {
    base: String,
    pub pair_id: String,
    secret: Vec<u8>,
    daemon_pub: Vec<u8>,
    keys: Keypair,
    poll: String,
    /// The code the phone shows the owner to compare.
    pub code: String,
}

/// Reads what the QR holds and joins, with `proof` or the right one.
pub async fn join(base: &str, qr: &str, name: &str, wrong_proof: bool) -> Joined {
    let fragment = qr.split_once('#').expect("a fragment").1;
    let part = |key: &str| -> String {
        fragment
            .split('&')
            .find_map(|pair| pair.strip_prefix(&format!("{key}=")))
            .unwrap_or_else(|| panic!("{key} in the QR"))
            .to_owned()
    };
    let (pair_id, secret, daemon_pub) = (
        part("p"),
        B64.decode(part("s")).expect("secret"),
        B64.decode(part("d")).expect("key"),
    );
    let keys = Keypair::generate().expect("keys");
    let mut proof = seal::join_proof(&secret, &pair_id, keys.public());
    if wrong_proof {
        proof[0] ^= 1;
    }
    let answer = reqwest::Client::new()
        .post(format!("{base}/v1/pairings/{pair_id}/join"))
        .json(&json!({
            "phone_pub": B64.encode(keys.public()), "device_name": name, "proof": B64.encode(proof),
        }))
        .send()
        .await
        .expect("join");
    assert_eq!(
        answer.status(),
        202,
        "{}",
        answer.text().await.unwrap_or_default()
    );
    let poll = answer.json::<Value>().await.expect("json")["poll"]
        .as_str()
        .expect("poll")
        .to_owned();
    let code = seal::code(&secret, &daemon_pub, keys.public());
    Joined {
        base: base.to_owned(),
        pair_id,
        secret,
        daemon_pub,
        keys,
        poll,
        code,
    }
}

impl Joined {
    /// Asks for the answer until the computer accepts. `None` if the code is
    /// gone (the owner refused, or it ran out).
    pub async fn collect(self) -> Option<Phone> {
        let url = format!("{}/v1/pairings/{}/result", self.base, self.pair_id);
        for _ in 0..200 {
            let answer: Value = reqwest::Client::new()
                .get(&url)
                .bearer_auth(&self.poll)
                .send()
                .await
                .expect("result")
                .json()
                .await
                .expect("json");
            match answer["status"].as_str() {
                Some("approved") => {
                    // The computer proves it accepted, with the QR's secret.
                    let theirs = B64
                        .decode(answer["daemon_pub"].as_str().expect("key"))
                        .expect("b64");
                    assert_eq!(theirs, self.daemon_pub, "the key is the one in the QR");
                    let proof = B64
                        .decode(answer["proof2"].as_str().expect("proof"))
                        .expect("b64");
                    let expected = seal::accept_proof(
                        &self.secret,
                        &self.pair_id,
                        &theirs,
                        self.keys.public(),
                    );
                    assert_eq!(proof, expected);
                    return Some(Phone {
                        base: self.base.clone(),
                        token: answer["token"].as_str().expect("token").to_owned(),
                        id: answer["device"].as_str().expect("device").to_owned(),
                        keys: seal::derive(&self.keys, &theirs, &self.secret).expect("keys"),
                        sent: 0,
                        socket: None,
                    });
                }
                Some("expired") => return None,
                _ => tokio::time::sleep(Duration::from_millis(50)).await,
            }
        }
        panic!("the computer never accepted");
    }

    /// A word for the owner who compares: the same six digits.
    pub fn secret(&self) -> &[u8] {
        &self.secret
    }
}

/// A phone that is connected to a computer.
pub struct Phone {
    base: String,
    pub token: String,
    pub id: String,
    keys: Keys,
    sent: u64,
    socket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl Phone {
    /// Opens the relay as the phone does and waits for `ready`.
    pub async fn connect(&mut self) -> Value {
        let url = self.base.replacen("http://", "ws://", 1) + "/v1/relay";
        let (socket, _) = connect_async(url).await.expect("connect");
        self.socket = Some(socket);
        self.send_raw(json!({ "t": "hello", "token": self.token }))
            .await;
        loop {
            let frame = self.frame().await;
            if frame["t"] == "ready" {
                return frame;
            }
        }
    }

    pub async fn send_raw(&mut self, frame: Value) {
        let socket = self.socket.as_mut().expect("connected");
        socket
            .send(Message::Text(frame.to_string().into()))
            .await
            .expect("send");
    }

    /// The text relayed for `message`, with the next counter; the frame is
    /// not sent.
    pub fn seal(&mut self, message: &FromPhone) -> Value {
        self.sent += 1;
        let plain = serde_json::to_vec(message).expect("json");
        let body = seal::seal(
            &self.keys.p2c,
            Dir::PhoneToComputer,
            &self.id,
            self.sent,
            &plain,
        )
        .expect("seal");
        json!({ "t": "msg", "seq": self.sent, "body": body })
    }

    /// Seals and sends. Gives the frame back, to send again.
    pub async fn send(&mut self, message: &FromPhone) -> Value {
        let frame = self.seal(message);
        self.send_raw(frame.clone()).await;
        frame
    }

    /// The next frame, whatever it is.
    pub async fn frame(&mut self) -> Value {
        let socket = self.socket.as_mut().expect("connected");
        let next = tokio::time::timeout(WAIT, socket.next())
            .await
            .expect("a frame in time");
        match next {
            Some(Ok(Message::Text(text))) => serde_json::from_str(&text).expect("json"),
            other => panic!("not a frame: {other:?}"),
        }
    }

    /// The next thing the computer says, opened, except for the changes in
    /// the list of conversations (`next_any` gives those too).
    pub async fn next(&mut self) -> ToPhone {
        loop {
            let message = self.next_any().await;
            if !matches!(message, ToPhone::Line { .. }) {
                return message;
            }
        }
    }

    /// The next message, opened. Frames that are not messages (who is
    /// online) are skipped.
    pub async fn next_any(&mut self) -> ToPhone {
        loop {
            let frame = self.frame().await;
            if frame["t"] != "msg" {
                continue;
            }
            let (seq, plain) = seal::open(
                &self.keys.c2p,
                Dir::ComputerToPhone,
                &self.id,
                frame["body"].as_str().expect("body"),
            )
            .expect("a message that opens");
            assert_eq!(Some(seq), frame["seq"].as_u64());
            return serde_json::from_slice(&plain).expect("a known message");
        }
    }

    /// Nothing more from the computer for a while.
    pub async fn quiet(&mut self) {
        let socket = self.socket.as_mut().expect("connected");
        loop {
            match tokio::time::timeout(Duration::from_millis(400), socket.next()).await {
                Err(_) => return,
                Ok(Some(Ok(Message::Text(text)))) => {
                    let frame: Value = serde_json::from_str(&text).expect("json");
                    if frame["t"] == "msg" {
                        let (_, plain) = seal::open(
                            &self.keys.c2p,
                            Dir::ComputerToPhone,
                            &self.id,
                            frame["body"].as_str().expect("body"),
                        )
                        .expect("a message that opens");
                        let message: ToPhone = serde_json::from_slice(&plain).expect("known");
                        assert!(
                            matches!(message, ToPhone::Line { .. }),
                            "unexpected message: {message:?}"
                        );
                    }
                }
                Ok(other) => panic!("the socket ended: {other:?}"),
            }
        }
    }

    /// The server closes the socket (the phone was cut off).
    pub async fn closed(&mut self) {
        let socket = self.socket.as_mut().expect("connected");
        loop {
            match tokio::time::timeout(WAIT, socket.next()).await {
                Ok(Some(Ok(Message::Text(_)))) => {}
                Ok(None | Some(Err(_) | Ok(Message::Close(_)))) => return,
                other => panic!("not closed: {other:?}"),
            }
        }
    }
}

/// A daemon signed in to its own account server.
pub async fn setup() -> (CloudServer, TestDaemon, Client) {
    let cloud = cloud_server().await;
    let t = TestDaemon::start_supervised_with_cloud(&cloud.url).await;
    let mut app = t.session().await;
    sign_in(&cloud, &mut app).await;
    (cloud, t, app)
}

/// Waits until `mobile.status` satisfies `ok`.
pub async fn status_where(app: &mut Client, ok: impl Fn(&Value) -> bool) -> Value {
    for _ in 0..200 {
        let status = app
            .call("mobile.status", Value::Null)
            .await
            .expect("status");
        if ok(&status) {
            return status;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("mobile.status never became what the test waits for");
}

/// Connects a phone as the owner would, comparing the codes.
pub async fn connect_phone(cloud: &CloudServer, app: &mut Client) -> Phone {
    let started = app
        .call("mobile.pair_start", Value::Null)
        .await
        .expect("start");
    assert_eq!(started["expiresIn"], 300);
    let qr = started["url"].as_str().expect("url");
    assert!(qr.starts_with(&format!("{}/m#p=", cloud.url)), "{qr}");
    let joined = join(&cloud.url, qr, "Celular da Ana", false).await;
    let request = app.notification("mobile.pair_request").await;
    assert_eq!(request["name"], "Celular da Ana");
    assert_eq!(
        request["code"],
        joined.code.as_str(),
        "both screens show the same code"
    );
    assert_eq!(request["pairId"], started["pairId"]);
    app.call(
        "mobile.pair_confirm",
        json!({ "pairId": started["pairId"], "accept": true }),
    )
    .await
    .expect("confirm");
    let mut phone = joined.collect().await.expect("a token");
    status_where(app, |s| s["phones"][0]["id"] == phone.id.as_str()).await;
    let ready = phone.connect().await;
    assert_eq!(ready["kind"], "phone");
    status_where(app, |s| s["phones"][0]["online"] == true).await;
    phone
}
