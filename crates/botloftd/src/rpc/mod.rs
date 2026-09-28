//! The app's connection: one WebSocket per client, JSON-RPC 2.0 over text
//! frames (spec 11). The first request must be `session.hello`.

mod dispatch;
pub mod jsonrpc;
mod terminal;

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use botloft_core::protocol::{
    HelloParams, HelloResult, PROTOCOL_VERSION, error_code, method, notification,
};
use futures_util::stream::SplitStream;
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc};
use tracing::{debug, warn};

use self::jsonrpc::RpcError;
use crate::state::{Daemon, Event};

/// How long a new connection may take to send `session.hello`.
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);
/// Outgoing frames queued per connection.
const OUTBOX: usize = 256;

pub async fn serve_connection(socket: WebSocket, daemon: Arc<Daemon>) {
    let (mut sink, mut stream) = socket.split();
    let (outbox, mut queued) = mpsc::channel::<String>(OUTBOX);
    let writer = tokio::spawn(async move {
        while let Some(text) = queued.recv().await {
            if sink.send(Message::Text(text.into())).await.is_err() {
                return;
            }
        }
        let _ = sink.send(Message::Close(None)).await;
    });

    // Subscribe before answering hello so no change slips in between.
    let events = daemon.subscribe();
    if authenticate(&mut stream, &outbox, &daemon).await {
        let mut streams = terminal::Streams::default();
        run(stream, &outbox, events, &daemon, &mut streams).await;
        streams.stop_all();
    }
    drop(outbox);
    let _ = writer.await;
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
    mut events: broadcast::Receiver<Event>,
    daemon: &Daemon,
    streams: &mut terminal::Streams,
) {
    loop {
        let frame = tokio::select! {
            message = stream.next() => match message {
                Some(Ok(Message::Text(text))) => handle(daemon, &text, outbox, streams).await,
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
        };
        if let Some(frame) = frame
            && outbox.send(frame).await.is_err()
        {
            return;
        }
    }
}

async fn handle(
    daemon: &Daemon,
    text: &str,
    outbox: &mpsc::Sender<String>,
    streams: &mut terminal::Streams,
) -> Option<String> {
    let request = match jsonrpc::parse(text) {
        Ok(request) => request,
        Err((id, err)) => return Some(jsonrpc::failure(&id, &err)),
    };
    debug!(method = %request.method, "rpc request");
    let id = request.id.clone();
    let result = match request.method.as_str() {
        method::TERMINAL_ATTACH => {
            // The response must go out before the first terminal.data.
            return terminal::attach(daemon, request, outbox, streams).await;
        }
        method::TERMINAL_DETACH => terminal::detach(request.params, streams),
        _ => dispatch::dispatch(daemon, &request.method, request.params),
    };
    let id = id?;
    Some(match result {
        Ok(value) => jsonrpc::success(&id, value),
        Err(err) => jsonrpc::failure(&id, &err),
    })
}

fn to_notification(event: &Event) -> String {
    let (name, params) = match event {
        Event::CrewChanged(crew) => (notification::CREW_CHANGED, serde_json::to_value(crew)),
        Event::BotChanged(bot) => (notification::BOT_CHANGED, serde_json::to_value(bot)),
        Event::BotState(state) => (notification::BOT_STATE, serde_json::to_value(state)),
    };
    jsonrpc::notification(name, params.unwrap_or(Value::Null))
}
