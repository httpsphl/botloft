//! The app's connection: one WebSocket per client, JSON-RPC 2.0 over text
//! frames (spec 11). The first request must be `session.hello`.

mod dispatch;
pub mod jsonrpc;
mod watching;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use botloft_core::protocol::{
    HelloParams, HelloResult, PROTOCOL_VERSION, error_code, method, notification,
};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::{debug, warn};

use self::jsonrpc::RpcError;
use self::watching::Watch;
use crate::state::{Daemon, Event};

/// How long a new connection may take to send `session.hello`.
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);
/// Outgoing frames queued per connection.
const OUTBOX: usize = 256;

pub async fn serve_connection(socket: WebSocket, daemon: Arc<Daemon>) {
    let (sink, mut stream) = socket.split();
    let (outbox, queued) = mpsc::channel::<String>(OUTBOX);
    // Live browser frames skip the queue: only the newest one waits.
    let (frames, newest) = watch::channel::<Option<String>>(None);
    let writer = tokio::spawn(write(sink, queued, newest));

    // Subscribe before answering hello so no change slips in between.
    let events = daemon.subscribe();
    if authenticate(&mut stream, &outbox, &daemon).await {
        run(stream, &outbox, &frames, events, &daemon).await;
    }
    drop(outbox);
    drop(frames);
    let _ = writer.await;
}

async fn write(
    mut sink: SplitSink<WebSocket, Message>,
    mut queued: mpsc::Receiver<String>,
    mut frames: watch::Receiver<Option<String>>,
) {
    let mut frames_open = true;
    loop {
        let text = tokio::select! {
            biased;
            text = queued.recv() => match text {
                Some(text) => text,
                None => break,
            },
            changed = frames.changed(), if frames_open => {
                if changed.is_err() {
                    frames_open = false;
                    continue;
                }
                match frames.borrow_and_update().clone() {
                    Some(frame) => frame,
                    None => continue,
                }
            }
        };
        if sink.send(Message::Text(text.into())).await.is_err() {
            return;
        }
    }
    let _ = sink.send(Message::Close(None)).await;
}

async fn authenticate(
    stream: &mut SplitStream<WebSocket>,
    outbox: &mpsc::Sender<String>,
    daemon: &Daemon,
) -> bool {
    let Ok(Some(Ok(Message::Text(text)))) =
        tokio::time::timeout(HELLO_TIMEOUT, stream.next()).await
    else {
        return false;
    };
    let request = match jsonrpc::parse(&text) {
        Ok(request) => request,
        Err((id, err)) => {
            let _ = outbox.send(jsonrpc::failure(&id, &err)).await;
            return false;
        }
    };
    let id = request.id.clone().unwrap_or(Value::Null);
    let result = hello(daemon, request);
    let frame = match &result {
        Ok(ok) => jsonrpc::success(&id, serde_json::to_value(ok).unwrap_or(Value::Null)),
        Err(err) => jsonrpc::failure(&id, err),
    };
    let _ = outbox.send(frame).await;
    result.is_ok()
}

fn hello(daemon: &Daemon, request: jsonrpc::Request) -> Result<HelloResult, RpcError> {
    let unauthenticated = |message: &str| RpcError::new(error_code::NOT_AUTHENTICATED, message);
    if request.method != method::SESSION_HELLO {
        return Err(unauthenticated("call session.hello first"));
    }
    let params: HelloParams = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|_| unauthenticated("session.hello needs token, client and protocol"))?;
    if !daemon.is_owner_token(&params.token) {
        warn!(client = %params.client.name, "rejected a connection with a wrong token");
        return Err(unauthenticated("invalid token"));
    }
    if params.protocol != PROTOCOL_VERSION {
        return Err(RpcError::new(
            error_code::VALIDATION,
            format!(
                "protocol {} is not supported; this daemon speaks {PROTOCOL_VERSION}",
                params.protocol
            ),
        ));
    }
    debug!(client = %params.client.name, version = %params.client.version, "app connected");
    Ok(HelloResult {
        daemon_version: env!("CARGO_PKG_VERSION").to_owned(),
        protocol: PROTOCOL_VERSION,
    })
}

async fn run(
    mut stream: SplitStream<WebSocket>,
    outbox: &mpsc::Sender<String>,
    frames: &watch::Sender<Option<String>>,
    mut events: broadcast::Receiver<Event>,
    daemon: &Daemon,
) {
    let mut watch = Watch::default();
    loop {
        let frame = tokio::select! {
            message = stream.next() => match message {
                Some(Ok(Message::Text(text))) => handle(daemon, &text, &mut watch, frames),
                Some(Ok(Message::Close(_)) | Err(_)) | None => return,
                Some(Ok(_)) => None,
            },
            event = events.recv() => match event {
                Ok(event) => Some(to_notification(&event)),
                Err(broadcast::error::RecvError::Lagged(skipped)) => {
                    // The client's view is stale; it reloads on reconnect.
                    warn!(skipped, "closing an app connection that fell behind");
                    return;
                }
                Err(broadcast::error::RecvError::Closed) => return,
            },
            frame = watch.next_frame() => {
                frames.send_replace(frame);
                None
            }
        };
        if let Some(frame) = frame
            && outbox.send(frame).await.is_err()
        {
            return;
        }
    }
}

fn handle(
    daemon: &Daemon,
    text: &str,
    watch: &mut Watch,
    frames: &watch::Sender<Option<String>>,
) -> Option<String> {
    let request = match jsonrpc::parse(text) {
        Ok(request) => request,
        Err((id, err)) => return Some(jsonrpc::failure(&id, &err)),
    };
    debug!(method = %request.method, "rpc request");
    let id = request.id.clone()?;
    // Watching a browser belongs to this connection (spec 21.7).
    let result = match request.method.as_str() {
        method::BROWSER_WATCH | method::BROWSER_UNWATCH => {
            frames.send_replace(None);
            watch.request(daemon, &request.method, request.params)
        }
        // So are the size of its page and what the owner does in it (spec
        // 21.10).
        method::BROWSER_RESIZE
        | method::BROWSER_TAKE
        | method::BROWSER_RELEASE
        | method::BROWSER_INPUT
        | method::BROWSER_RELOAD
        | method::BROWSER_NEW_TAB
        | method::BROWSER_SWITCH_TAB
        | method::BROWSER_OPEN => watch.request(daemon, &request.method, request.params),
        name => dispatch::dispatch(daemon, name, request.params),
    };
    Some(match result {
        Ok(value) => jsonrpc::success(&id, value),
        Err(err) => jsonrpc::failure(&id, &err),
    })
}

fn to_notification(event: &Event) -> String {
    let (name, params) = match event {
        Event::CrewChanged(crew) => (notification::CREW_CHANGED, serde_json::to_value(crew)),
        Event::CrewDeleted(crew) => (notification::CREW_DELETED, serde_json::to_value(crew)),
        Event::BotChanged(bot) => (notification::BOT_CHANGED, serde_json::to_value(bot)),
        Event::BotDeleted(bot) => (notification::BOT_DELETED, serde_json::to_value(bot)),
        Event::FolderRecycled(folder) => {
            (notification::FOLDER_RECYCLED, serde_json::to_value(folder))
        }
        Event::BotState(state) => (notification::BOT_STATE, serde_json::to_value(state)),
        Event::ChatItem(item) => (notification::CHAT_ITEM, serde_json::to_value(item)),
        Event::ChatDelta(delta) => (notification::CHAT_DELTA, serde_json::to_value(delta)),
        Event::MessageCreated(message) => {
            (notification::MESSAGE_CREATED, serde_json::to_value(message))
        }
        Event::DeliveryChanged(delivery) => (
            notification::DELIVERY_CHANGED,
            serde_json::to_value(delivery),
        ),
        Event::TaskChanged(task) => (notification::TASK_CHANGED, serde_json::to_value(task)),
        Event::RoutineChanged(routine) => {
            (notification::ROUTINE_CHANGED, serde_json::to_value(routine))
        }
        Event::RoutineRun(run) => (notification::ROUTINE_RUN, serde_json::to_value(run)),
        Event::BrowserChanged(state) => {
            (notification::BROWSER_CHANGED, serde_json::to_value(state))
        }
        Event::BrowserAction(action) => {
            (notification::BROWSER_ACTION, serde_json::to_value(action))
        }
        Event::ScreenDraft(draft) => (notification::SCREEN_DRAFT, serde_json::to_value(draft)),
    };
    jsonrpc::notification(name, params.unwrap_or(Value::Null))
}
