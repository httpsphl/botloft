//! A small Chrome DevTools Protocol client over one WebSocket (spec 21):
//! commands with ids, answered in any order, and every event in order for
//! one reader. Attached pages use the flat mode, with `sessionId` on each
//! message.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::tungstenite::Message;
use tracing::debug;

/// How long a command may take. Navigation answers once the request is
/// sent, not when the page loads, so this is generous.
const CALL_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum CdpError {
    #[error("the browser closed")]
    Closed,
    #[error("the browser did not answer {0} in time")]
    Timeout(String),
    #[error("{method}: {message}")]
    Protocol { method: String, message: String },
    #[error("cannot connect to the browser: {0}")]
    Connect(String),
}

/// An event from the browser; `session` is the page it came from, `None`
/// for the browser itself.
#[derive(Debug, Clone)]
pub struct CdpEvent {
    pub method: String,
    pub params: Value,
    pub session: Option<String>,
}

type Answer = Result<Value, String>;
type Waiting = Arc<Mutex<HashMap<u64, oneshot::Sender<Answer>>>>;

#[derive(Clone)]
pub struct Cdp {
    outgoing: mpsc::UnboundedSender<String>,
    waiting: Waiting,
    closed: Arc<AtomicBool>,
    next_id: Arc<AtomicU64>,
}

fn lock(waiting: &Waiting) -> MutexGuard<'_, HashMap<u64, oneshot::Sender<Answer>>> {
    waiting
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Cdp {
    /// Connects to `url` (`ws://127.0.0.1:<port>/devtools/browser/<id>`).
    /// The events end when the connection closes.
    pub async fn connect(url: &str) -> Result<(Self, mpsc::UnboundedReceiver<CdpEvent>), CdpError> {
        let (socket, _) = tokio_tungstenite::connect_async(url)
            .await
            .map_err(|err| CdpError::Connect(err.to_string()))?;
        let (mut sink, mut stream) = socket.split();
        let (outgoing, mut queued) = mpsc::unbounded_channel::<String>();
        let (events, events_rx) = mpsc::unbounded_channel();
        let waiting = Waiting::default();
        let closed = Arc::new(AtomicBool::new(false));

        tokio::spawn(async move {
            while let Some(text) = queued.recv().await {
                if sink.send(Message::text(text)).await.is_err() {
                    break;
                }
            }
            let _ = sink.close().await;
        });

        let reader_waiting = Arc::clone(&waiting);
        let reader_closed = Arc::clone(&closed);
        tokio::spawn(async move {
            while let Some(Ok(message)) = stream.next().await {
                let text = match message {
                    Message::Text(text) => text,
                    Message::Close(_) => break,
                    _ => continue,
                };
                let Ok(mut value) = serde_json::from_str::<Value>(text.as_str()) else {
                    continue;
                };
                if let Some(id) = value.get("id").and_then(Value::as_u64) {
                    if let Some(waiter) = lock(&reader_waiting).remove(&id) {
                        let _ = waiter.send(answer(&mut value));
                    }
                } else if let Some(method) = value.get("method").and_then(Value::as_str) {
                    let event = CdpEvent {
                        method: method.to_owned(),
                        session: value
                            .get("sessionId")
                            .and_then(Value::as_str)
                            .map(str::to_owned),
                        params: value.get_mut("params").map(Value::take).unwrap_or_default(),
                    };
                    let _ = events.send(event);
                }
            }
            debug!("browser: devtools connection closed");
            reader_closed.store(true, Ordering::SeqCst);
            // Dropping the senders fails every command still waiting.
            lock(&reader_waiting).clear();
        });

        Ok((
            Self {
                outgoing,
                waiting,
                closed,
                next_id: Arc::new(AtomicU64::new(1)),
            },
            events_rx,
        ))
    }

    pub fn is_closed(&self) -> bool {
        self.closed.load(Ordering::SeqCst)
    }

    /// Sends a command, to the page `session` or to the browser, and waits
    /// for its result.
    pub async fn call(
        &self,
        session: Option<&str>,
        method: &str,
        params: Value,
    ) -> Result<Value, CdpError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (answer, waiting) = oneshot::channel();
        lock(&self.waiting).insert(id, answer);
        if self.is_closed()
            || self
                .outgoing
                .send(frame(id, session, method, params))
                .is_err()
        {
            lock(&self.waiting).remove(&id);
            return Err(CdpError::Closed);
        }
        match tokio::time::timeout(CALL_TIMEOUT, waiting).await {
            Ok(Ok(Ok(result))) => Ok(result),
            Ok(Ok(Err(message))) => Err(CdpError::Protocol {
                method: method.to_owned(),
                message,
            }),
            Ok(Err(_)) => Err(CdpError::Closed),
            Err(_) => {
                lock(&self.waiting).remove(&id);
                Err(CdpError::Timeout(method.to_owned()))
            }
        }
    }

    /// Sends a command without waiting for its result.
    pub fn send(&self, session: Option<&str>, method: &str, params: Value) {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let _ = self.outgoing.send(frame(id, session, method, params));
    }
}

fn frame(id: u64, session: Option<&str>, method: &str, params: Value) -> String {
    let mut message = json!({ "id": id, "method": method, "params": params });
    if let Some(session) = session {
        message["sessionId"] = json!(session);
    }
    message.to_string()
}

fn answer(value: &mut Value) -> Answer {
    match value.get("error") {
        Some(error) => Err(error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown error")
            .to_owned()),
        None => Ok(value.get_mut("result").map(Value::take).unwrap_or_default()),
    }
}
