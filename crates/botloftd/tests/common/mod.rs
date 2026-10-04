//! Test harness: a daemon on an ephemeral port with temporary folders, and a
//! small JSON-RPC client over a real WebSocket.

#![allow(dead_code)] // each test file uses a different part of the harness

pub mod bots;
pub mod browsing;
pub mod context;
pub mod mcp;
pub mod routines;
pub mod site;
pub mod stream;
pub mod supervised;

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use botloft_store::Store;
use botloftd::browser::BrowserSettings;
use botloftd::clock::ManualClock;
use botloftd::config::Config;
use botloftd::courier::{self, CourierSettings};
use botloftd::paths::Paths;
use botloftd::runtime::claude::Claude;
use botloftd::runtime::fake::FakeRuntime;
use botloftd::secrets::TokenHash;
use botloftd::server;
use botloftd::service::tasks::TaskSettings;
use botloftd::settings::LiveSettings;
use botloftd::state::{BotSettings, Daemon, DaemonOptions};
use botloftd::supervisor::{self, ClaudeSource, SupervisorSettings};
use botloftd::trash::FakeTrash;
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
    pub daemon: Arc<Daemon>,
    pub runtime: FakeRuntime,
    pub clock: Arc<ManualClock>,
    /// Where folders go instead of the Recycle Bin.
    pub trash: FakeTrash,
    _dir: TempDir,
}

/// Backoff short enough for tests that run on the real clock.
pub fn test_settings() -> SupervisorSettings {
    SupervisorSettings {
        claude: ClaudeSource::Fixed(Claude {
            path: PathBuf::from(r"C:\Claude\claude.exe"),
            version: "2.1.284".to_owned(),
        }),
        backoff_initial: Duration::from_millis(40),
        backoff_max: Duration::from_millis(200),
        fresh_start_if_dies_within: Duration::from_secs(15),
        ready_after: Duration::from_millis(20),
    }
}

pub fn bot_settings() -> BotSettings {
    BotSettings {
        attachment_max_bytes: 1024 * 1024,
        max_per_crew: 3,
    }
}

/// Polls often on the real clock; retries are timed by the manual clock.
pub fn courier_settings() -> CourierSettings {
    CourierSettings {
        poll_interval: Duration::from_millis(10),
        lease: Duration::from_secs(15),
        max_attempts: 3,
        retry_backoff_initial: Duration::from_secs(1),
        retry_backoff_max: Duration::from_secs(4),
    }
}

/// A daemon with fakes for everything outside the process, and no server.
pub struct Parts {
    pub daemon: Arc<Daemon>,
    pub runtime: FakeRuntime,
    pub clock: Arc<ManualClock>,
    pub paths: Paths,
    pub trash: FakeTrash,
    pub dir: TempDir,
}

pub fn new_daemon(settings: SupervisorSettings) -> Parts {
    new_daemon_waiting(settings, APPROVAL_WAIT)
}

/// Approvals time out fast enough for a test to wait for it.
const APPROVAL_WAIT: Duration = Duration::from_secs(3);
/// For tests with a real browser, whose requests may come late on a busy
/// CI runner: long enough that none expires before the test answers it.
const PATIENT_APPROVAL_WAIT: Duration = Duration::from_secs(30);

pub fn new_daemon_waiting(settings: SupervisorSettings, approval_wait: Duration) -> Parts {
    let dir = tempfile::tempdir().expect("tempdir");
    let paths = Paths::new(dir.path().join("home"), dir.path().join("workspaces"));
    let runtime = FakeRuntime::new();
    let clock = Arc::new(ManualClock::new());
    let trash = FakeTrash::new(dir.path().join("bin"));
    let daemon = Daemon::new(DaemonOptions {
        paths: paths.clone(),
        port: 45710,
        store: Store::open_in_memory().expect("store"),
        owner_token: TokenHash::of(TOKEN),
        runtime: Arc::new(runtime.clone()),
        supervisor: settings,
        clock: Arc::clone(&clock) as _,
        courier: courier_settings(),
        tasks: TaskSettings {
            max_hops: 3,
            default_deadline: Duration::from_secs(120 * 60),
        },
        bots: bot_settings(),
        browser: BrowserSettings::default(),
        settings: LiveSettings::new(None, &Config::default()).with_approval_wait(approval_wait),
        trash: Arc::new(trash.clone()),
    });
    Parts {
        daemon,
        runtime,
        clock,
        paths,
        trash,
        dir,
    }
}

impl TestDaemon {
    /// Server only: bots are never started and nothing is delivered.
    pub async fn start() -> Self {
        Self::launch(false, APPROVAL_WAIT).await
    }

    /// Server, supervisor and courier, with bots running on a
    /// [`FakeRuntime`].
    pub async fn start_supervised() -> Self {
        Self::launch(true, APPROVAL_WAIT).await
    }

    /// As [`TestDaemon::start_supervised`], with approvals that wait long
    /// enough for a real browser on a slow machine.
    pub async fn start_supervised_patient() -> Self {
        Self::launch(true, PATIENT_APPROVAL_WAIT).await
    }

    async fn launch(supervised: bool, approval_wait: Duration) -> Self {
        let parts = new_daemon_waiting(test_settings(), approval_wait);
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");
        if supervised {
            tokio::spawn(supervisor::run(Arc::clone(&parts.daemon)));
            tokio::spawn(courier::run(Arc::clone(&parts.daemon)));
        }
        tokio::spawn(server::serve(
            Arc::clone(&parts.daemon),
            listener,
            std::future::pending(),
        ));
        Self {
            addr,
            paths: parts.paths,
            daemon: parts.daemon,
            runtime: parts.runtime,
            clock: parts.clock,
            trash: parts.trash,
            _dir: parts.dir,
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
            json!({ "token": token, "client": { "name": "tests", "version": "0" }, "protocol": 2 }),
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

    /// The notifications named `method` that came before the last one
    /// waited for, without waiting for more.
    pub fn queued(&self, method: &str) -> Vec<Value> {
        let named = self.notifications.iter().filter(|n| n["method"] == method);
        named.map(|n| n["params"].clone()).collect()
    }

    /// Drops the notifications named `method` that came so far.
    pub fn forget(&mut self, method: &str) {
        self.notifications.retain(|n| n["method"] != method);
    }

    /// True when the server closes the connection without sending more.
    pub async fn closed(&mut self) -> bool {
        self.recv().await.is_none()
    }
}
