//! Helpers for the phone's tests (spec 28): a real listener for the relay, a
//! WebSocket client, and the pairing flow with made-up (but well-shaped) keys.

#![allow(dead_code)]

use std::net::SocketAddr;
use std::time::Duration;

use axum::body::Body;
use axum::http::{Request, header};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use super::{Reply, Server};

/// 22 characters, as a pairing id is (16 bytes in base64url).
pub fn pair_id(n: u32) -> String {
    format!("{n:0>22}")
}

/// 43 characters, as a proof is (an HMAC-SHA-256 in base64url).
pub fn proof(letter: char) -> String {
    letter.to_string().repeat(43)
}

/// A key as the server sees it: only its shape matters to the server.
pub fn key(letter: char) -> String {
    letter.to_string().repeat(87)
}

/// A phone that finished connecting.
pub struct Phone {
    pub token: String,
    pub device: String,
    pub pairing: String,
}

impl Server {
    /// A request with a JSON body, and a bearer token if there is one.
    pub async fn call(
        &self,
        method: &str,
        path: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> Reply {
        let mut request = Request::builder().method(method).uri(path);
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        let body = match body {
            Some(body) => {
                request = request.header(header::CONTENT_TYPE, "application/json");
                Body::from(body.to_string())
            }
            None => Body::empty(),
        };
        self.send(request.body(body).expect("request")).await
    }

    /// Serves the router on a free port, for the relay's sockets.
    pub async fn listen(&self) -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let address = listener.local_addr().expect("address");
        let app = self.app.clone();
        tokio::spawn(async move {
            let _ = axum::serve(listener, app).await;
        });
        address
    }

    /// The whole flow of 28.3 with a computer's token: the pairing id is
    /// `pair_id(n)`, and the phone ends up with its own token.
    pub async fn pair(&self, computer: &str, n: u32) -> Phone {
        let id = pair_id(n);
        let opened = self
            .call(
                "POST",
                "/v1/pairings",
                Some(computer),
                Some(json!({ "id": id })),
            )
            .await;
        assert_eq!(opened.status, 201, "{}", opened.body);
        let joined = self
            .call(
                "POST",
                &format!("/v1/pairings/{id}/join"),
                None,
                Some(json!({
                    "phone_pub": key('P'), "device_name": "Celular da Ana", "proof": proof('p'),
                })),
            )
            .await;
        assert_eq!(joined.status, 202, "{}", joined.body);
        let poll = joined.json()["poll"].as_str().expect("poll").to_owned();
        let accepted = self
            .call(
                "POST",
                &format!("/v1/pairings/{id}/accept"),
                Some(computer),
                Some(json!({ "daemon_pub": key('D'), "proof2": proof('d') })),
            )
            .await;
        assert_eq!(accepted.status, 204, "{}", accepted.body);
        let got = self
            .call(
                "GET",
                &format!("/v1/pairings/{id}/result"),
                Some(&poll),
                None,
            )
            .await
            .json();
        assert_eq!(got["status"], "approved", "{got}");
        Phone {
            token: got["token"].as_str().expect("token").to_owned(),
            device: got["device"].as_str().expect("device").to_owned(),
            pairing: id,
        }
    }
}

/// A client of the relay.
pub struct Sock(WebSocketStream<MaybeTlsStream<TcpStream>>);

const WAIT: Duration = Duration::from_secs(3);

impl Sock {
    /// Connects and says hello with `token`; the next frame is the server's answer.
    pub async fn open(address: SocketAddr, token: &str) -> Self {
        let mut sock = Self::connect(address).await;
        sock.send(json!({ "t": "hello", "token": token })).await;
        sock
    }

    pub async fn connect(address: SocketAddr) -> Self {
        let (socket, _) = tokio_tungstenite::connect_async(format!("ws://{address}/v1/relay"))
            .await
            .expect("connect");
        Self(socket)
    }

    pub async fn send(&mut self, frame: Value) {
        self.send_text(&frame.to_string()).await;
    }

    pub async fn send_text(&mut self, text: &str) {
        self.0
            .send(Message::Text(text.to_owned().into()))
            .await
            .expect("send");
    }

    /// The next frame from the server.
    pub async fn next(&mut self) -> Value {
        let frame = tokio::time::timeout(WAIT, self.0.next())
            .await
            .expect("a frame in time");
        match frame {
            Some(Ok(Message::Text(text))) => serde_json::from_str(&text).expect("json"),
            other => panic!("not a frame: {other:?}"),
        }
    }

    /// Opens as `open` does and checks the `ready` frame.
    pub async fn ready(address: SocketAddr, token: &str) -> (Self, Value) {
        let mut sock = Self::open(address, token).await;
        let ready = sock.next().await;
        assert_eq!(ready["t"], "ready", "{ready}");
        (sock, ready)
    }

    /// No frame arrives for a while.
    pub async fn quiet(&mut self) {
        let frame = tokio::time::timeout(Duration::from_millis(300), self.0.next()).await;
        assert!(frame.is_err(), "unexpected: {frame:?}");
    }

    /// The server closes this socket.
    pub async fn closed(&mut self) {
        loop {
            let frame = tokio::time::timeout(WAIT, self.0.next())
                .await
                .expect("closed in time");
            match frame {
                None | Some(Err(_) | Ok(Message::Close(_))) => return,
                Some(Ok(_)) => {}
            }
        }
    }

    /// Once this returns, the server has handled every frame sent before it:
    /// a frame that is not JSON is answered, and frames are handled in order.
    pub async fn sync(&mut self) {
        self.send_text("not json").await;
        let reply = self.next().await;
        assert_eq!(reply["reason"], "bad_frame", "{reply}");
    }
}
