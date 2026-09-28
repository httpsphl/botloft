//! Test harness: a daemon on an ephemeral port with temporary folders, and a
//! small JSON-RPC client over a real WebSocket.

#![allow(dead_code)] // each test file uses a different part of the harness

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use botloft_store::Store;
use botloftd::paths::Paths;
use botloftd::secrets::TokenHash;
use botloftd::server;
use botloftd::state::{Daemon, DaemonOptions};
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tempfile::TempDir;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::{self, Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

pub const TOKEN: &str = "test-owner-token";
const WAIT: Duration = Duration::from_secs(5);

pub struct TestDaemon {
    pub addr: SocketAddr,
    pub paths: Paths,
    _dir: TempDir,
}

impl TestDaemon {
    pub async fn start() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths = Paths::new(dir.path().join("home"), dir.path().join("workspaces"));
        let daemon = Arc::new(Daemon::new(DaemonOptions {
            paths: paths.clone(),
            port: 45710,
            bin: PathBuf::from(r"C:\Botloft\bin\botloftd.exe"),
            store: Store::open_in_memory().expect("store"),
            owner_token: TokenHash::of(TOKEN),
        }));
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        tokio::spawn(server::serve(daemon, listener, std::future::pending()));
        Self {
            addr,
            paths,
            _dir: dir,
        }
    }

    pub fn url(&self) -> String {
        format!("ws://{}/rpc", self.addr)
    }

    pub async fn client(&self) -> Client {
        Client::connect(&self.url(), None).await.expect("connect")
    }

    /// A client that already passed `session.hello`.
    pub async fn session(&self) -> Client {
        let mut client = self.client().await;
        client.hello(TOKEN).await.expect("hello");
        client
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RpcFailure {
    pub code: i64,
    pub message: String,
}

pub struct Client {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    next_id: u64,
    notifications: VecDeque<Value>,
}

impl Client {
    pub async fn connect(url: &str, origin: Option<&str>) -> Result<Self, tungstenite::Error> {
        let mut request = url.into_client_request()?;
        if let Some(origin) = origin {
            let value = origin.parse().expect("header value");
            request.headers_mut().insert("Origin", value);
        }
        let (ws, _) = connect_async(request).await?;
        Ok(Self {
            ws,
            next_id: 1,
            notifications: VecDeque::new(),
        })
    }

    pub async fn send_raw(&mut self, text: &str) {
        self.ws.send(Message::text(text)).await.expect("send");
    }

    /// The next JSON frame, or `None` once the server closed the connection.
    pub async fn recv(&mut self) -> Option<Value> {
        loop {
            let frame = tokio::time::timeout(WAIT, self.ws.next())
                .await
                .expect("timed out waiting for the server");
            match frame {
                Some(Ok(Message::Text(text))) => {
                    return Some(serde_json::from_str(text.as_str()).expect("server sent JSON"));
                }
                Some(Ok(Message::Close(_)) | Err(_)) | None => return None,
                Some(Ok(_)) => {}
            }
        }
    }

    /// Sends a request and waits for its response, queueing notifications
    /// that arrive in between. `Value::Null` params are left out.
    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, RpcFailure> {
        let id = self.next_id;
        self.next_id += 1;
        let mut request = json!({ "jsonrpc": "2.0", "id": id, "method": method });
        if !params.is_null() {
            request["params"] = params;
        }
        self.send_raw(&request.to_string()).await;
        loop {
            let frame = self
                .recv()
                .await
                .expect("connection closed before the response");
            if frame.get("id") == Some(&json!(id)) {
                return match frame.get("error") {
                    Some(error) => Err(RpcFailure {
                        code: error["code"].as_i64().expect("error code"),
                        message: error["message"].as_str().unwrap_or_default().to_owned(),
                    }),
                    None => Ok(frame["result"].clone()),
                };
            }
            assert!(frame.get("method").is_some(), "unexpected frame {frame}");
            self.notifications.push_back(frame);
        }
    }

    pub async fn hello(&mut self, token: &str) -> Result<Value, RpcFailure> {
        self.call(
            "session.hello",
            json!({ "token": token, "client": { "name": "tests", "version": "0" }, "protocol": 1 }),
        )
        .await
    }

    /// Waits for the next notification named `method`, returning its params.
    pub async fn notification(&mut self, method: &str) -> Value {
        if let Some(pos) = self
            .notifications
            .iter()
            .position(|n| n["method"] == method)
        {
            let found = self.notifications.remove(pos).expect("queued");
            return found["params"].clone();
        }
        loop {
            let frame = self.recv().await.expect("connection closed");
            if frame["method"] == method {
                return frame["params"].clone();
            }
            self.notifications.push_back(frame);
        }
    }

    /// True when the server closes the connection without sending more.
    pub async fn closed(&mut self) -> bool {
        self.recv().await.is_none()
    }
}
