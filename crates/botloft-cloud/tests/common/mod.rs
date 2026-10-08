//! A server in memory: the router is called directly, mail goes to an outbox
//! and the clock moves when a test says so.

#![allow(dead_code)]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use botloft_cloud::{AppState, Clock, Config, CopyStore, Db, Hub, Mailer, Outbox, Pusher, router};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tower::ServiceExt;

pub mod mobile;

pub struct Server {
    app: Router,
    pub store: CopyStore,
    pub outbox: Outbox,
    pub clock: Clock,
}

pub struct Reply {
    pub status: StatusCode,
    pub headers: axum::http::HeaderMap,
    pub bytes: Vec<u8>,
    pub body: String,
}

impl Reply {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or_else(|_| panic!("not json: {}", self.body))
    }
}

pub fn server() -> Server {
    start(Outbox::default())
}

pub fn start(outbox: Outbox) -> Server {
    build(outbox, Config::for_tests(), Pusher::off())
}

/// A server that sends Web Push through `pusher`.
pub fn server_pushing(pusher: Pusher) -> Server {
    build(Outbox::default(), Config::for_tests(), pusher)
}

/// A server with other limits than the defaults.
pub fn server_with(change: impl FnOnce(&mut Config)) -> Server {
    let mut config = Config::for_tests();
    change(&mut config);
    build(Outbox::default(), config, Pusher::off())
}

fn build(outbox: Outbox, config: Config, push: Pusher) -> Server {
    let clock = Clock::default();
    let store = CopyStore::memory();
    let state = AppState {
        db: Db::memory().expect("db"),
        mailer: Arc::new(Mailer::Outbox(outbox.clone())),
        clock: clock.clone(),
        config: Arc::new(config),
        store: store.clone(),
        hub: Hub::default(),
        push,
    };
    Server {
        app: router(state),
        store,
        outbox,
        clock,
    }
}

impl Server {
    pub async fn send(&self, request: Request<Body>) -> Reply {
        let response = self.app.clone().oneshot(request).await.expect("response");
        let (status, headers) = (response.status(), response.headers().clone());
        let bytes = to_bytes(response.into_body(), 1 << 20).await.expect("body");
        Reply {
            status,
            headers,
            body: String::from_utf8_lossy(&bytes).into_owned(),
            bytes: bytes.to_vec(),
        }
    }

    /// `POST /v1/login` from `address`.
    pub async fn ask(&self, email: &str, address: &str) -> Reply {
        self.ask_with(
            json!({ "email": email, "device_name": "PC da Ana", "locale": "pt-BR" }),
            address,
        )
        .await
    }

    pub async fn ask_with(&self, body: Value, address: &str) -> Reply {
        self.ask_via(body, address, &[]).await
    }

    /// As `ask_with`, with more headers on the request.
    pub async fn ask_via(&self, body: Value, address: &str, more: &[(&str, &str)]) -> Reply {
        let mut request = Request::post("/v1/login")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-forwarded-for", address);
        for (name, value) in more {
            request = request.header(*name, *value);
        }
        self.send(request.body(Body::from(body.to_string())).expect("request"))
            .await
    }

    pub async fn get(&self, path: &str) -> Reply {
        self.send(Request::get(path).body(Body::empty()).expect("request"))
            .await
    }

    pub async fn authed(&self, method: &str, path: &str, token: &str) -> Reply {
        let request = Request::builder()
            .method(method)
            .uri(path)
            .header(header::AUTHORIZATION, format!("Bearer {token}"))
            .body(Body::empty())
            .expect("request");
        self.send(request).await
    }

    /// `PUT /v1/copies` with the right hash and length.
    pub async fn put_copy(&self, token: &str, data: &[u8]) -> Reply {
        self.put_copy_with(token, data, Some(&sha(data)), Some(data.len()))
            .await
    }

    /// `PUT /v1/copies` with the hash and the declared length left to the test.
    pub async fn put_copy_with(
        &self,
        token: &str,
        data: &[u8],
        hash: Option<&str>,
        length: Option<usize>,
    ) -> Reply {
        let mut request =
            Request::put("/v1/copies").header(header::AUTHORIZATION, format!("Bearer {token}"));
        if let Some(hash) = hash {
            request = request.header("x-botloft-sha256", hash);
        }
        if let Some(length) = length {
            request = request.header(header::CONTENT_LENGTH, length);
        }
        self.send(request.body(Body::from(data.to_vec())).expect("request"))
            .await
    }

    /// `GET` with the token and, maybe, a `Range`.
    pub async fn fetch(
        &self,
        path: &str,
        token: &str,
        range: Option<&str>,
    ) -> (StatusCode, header::HeaderMap, Vec<u8>) {
        let mut request =
            Request::get(path).header(header::AUTHORIZATION, format!("Bearer {token}"));
        if let Some(range) = range {
            request = request.header(header::RANGE, range);
        }
        let response = self
            .app
            .clone()
            .oneshot(request.body(Body::empty()).expect("request"))
            .await
            .expect("response");
        let (status, headers) = (response.status(), response.headers().clone());
        let bytes = to_bytes(response.into_body(), 1 << 22).await.expect("body");
        (status, headers, bytes.to_vec())
    }

    /// The button on the page the e-mail link opens.
    pub async fn press(&self, code: &str) -> Reply {
        let request = Request::post("/v1/login/confirm")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!("code={code}")))
            .expect("request");
        self.send(request).await
    }

    /// The button on the page that deletes an account.
    pub async fn post_delete(&self, code: &str) -> Reply {
        let request = Request::post("/v1/account/delete/confirm")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(format!("code={code}")))
            .expect("request");
        self.send(request).await
    }

    /// The link of the newest e-mail, as a path and the code in it.
    pub fn last_link(&self) -> (String, String) {
        let mail = self.outbox.sent().pop().expect("an e-mail");
        let link = mail
            .body
            .split_whitespace()
            .find(|word| word.starts_with("http"))
            .expect("a link");
        let path = link
            .strip_prefix("http://127.0.0.1:8787")
            .expect("our address");
        let code = path.split("code=").nth(1).expect("a code");
        (path.to_owned(), code.to_owned())
    }

    /// Signs a device in and returns its token.
    pub async fn sign_in(&self, email: &str, address: &str) -> String {
        let asked = self.ask(email, address).await;
        assert_eq!(asked.status, StatusCode::ACCEPTED, "{}", asked.body);
        let request = asked.json()["request"]
            .as_str()
            .expect("request")
            .to_owned();
        let (_, code) = self.last_link();
        assert_eq!(self.press(&code).await.status, StatusCode::OK);
        let polled = self.get(&format!("/v1/login/{request}")).await.json();
        assert_eq!(polled["status"], "approved", "{polled}");
        polled["token"].as_str().expect("token").to_owned()
    }
}

/// The SHA-256 of `data`, as the upload says it.
pub fn sha(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}
