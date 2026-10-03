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
    daemon: &Arc<Daemon>,
) {
    let mut watch = Watch::default();
    loop {
        let frame = tokio::select! {
            message = stream.next() => match message {
                Some(Ok(Message::Text(text))) => match route(daemon, &text, &mut watch, frames) {
                    Routed::Reply(frame) => frame,
                    Routed::InOrder(request) => answer(daemon, request).await,
                    Routed::Aside(request) => {
                        aside(daemon, request, outbox.clone());
                        None
                    }
                },
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

/// Reads that the app never sets against notifications: their answer may
/// come after newer notifications, so they wait neither for the requests
/// before them nor hold back the notifications meanwhile.
const ASIDE: [&str; 6] = [
    method::FILES_LIST,
    method::FILES_READ,
    method::SCREENS_LIST,
    method::ATTACHMENTS_READ,
    method::USAGE_TOKENS,
    method::CHAT_SEARCH,
];

enum Routed {
    /// Answered here, or nothing to answer.
    Reply(Option<String>),
    /// Answered before anything else goes out (see [`answer`]).
    InOrder(jsonrpc::Request),
    /// Answered whenever it is ready (see [`ASIDE`]).
    Aside(jsonrpc::Request),
}

fn route(
    daemon: &Arc<Daemon>,
    text: &str,
    watch: &mut Watch,
    frames: &watch::Sender<Option<String>>,
) -> Routed {
    let request = match jsonrpc::parse(text) {
        Ok(request) => request,
        Err((id, err)) => return Routed::Reply(Some(jsonrpc::failure(&id, &err))),
    };
    debug!(method = %request.method, "rpc request");
    let Some(id) = request.id.clone() else {
        return Routed::Reply(None);
    };
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
        // A window of its own outlives the request (spec 21.11).
        method::BROWSER_WINDOW => watch.window(daemon, request.params),
        name if ASIDE.contains(&name) => return Routed::Aside(request),
        _ => return Routed::InOrder(request),
    };
    Routed::Reply(Some(reply(&id, result)))
}

/// Answers on the blocking pool, so the request holds no runtime thread.
/// The connection waits for it: the app takes an answer as newer than
/// every notification before it (spec 11.1).
async fn answer(daemon: &Arc<Daemon>, request: jsonrpc::Request) -> Option<String> {
    let daemon = Arc::clone(daemon);
    let id = request.id.clone().unwrap_or(Value::Null);
    let answered = tokio::task::spawn_blocking(move || {
        dispatch::dispatch(&daemon, &request.method, request.params)
    })
    .await;
    let result = answered.unwrap_or_else(|_| {
        Err(RpcError::new(
            error_code::INTERNAL_ERROR,
            "the request failed inside the daemon",
        ))
    });
    Some(reply(&id, result))
}

/// Answers an [`ASIDE`] read on its own.
fn aside(daemon: &Arc<Daemon>, request: jsonrpc::Request, outbox: mpsc::Sender<String>) {
    let daemon = Arc::clone(daemon);
    tokio::spawn(async move {
        if let Some(frame) = answer(&daemon, request).await {
            let _ = outbox.send(frame).await;
        }
    });
}

fn reply(id: &Value, result: Result<Value, RpcError>) -> String {
    match result {
        Ok(value) => jsonrpc::success(id, value),
        Err(err) => jsonrpc::failure(id, &err),
    }
}

fn to_notification(event: &Event) -> String {
    use jsonrpc::notification as note;
    match event {
        Event::CrewChanged(crew) => note(notification::CREW_CHANGED, crew),
        Event::CrewDeleted(crew) => note(notification::CREW_DELETED, crew),
        Event::BotChanged(bot) => note(notification::BOT_CHANGED, bot),
        Event::BotDeleted(bot) => note(notification::BOT_DELETED, bot),
        Event::FolderRecycled(folder) => note(notification::FOLDER_RECYCLED, folder),
        Event::BotState(state) => note(notification::BOT_STATE, state),
        Event::BotContext(context) => note(notification::BOT_CONTEXT, context),
        Event::BotRules(rules) => note(notification::BOT_RULES, rules),
        Event::ChatItem(item) => note(notification::CHAT_ITEM, item),
        Event::ChatDelta(delta) => note(notification::CHAT_DELTA, delta),
        Event::MessageCreated(message) => note(notification::MESSAGE_CREATED, message),
        Event::DeliveryChanged(delivery) => note(notification::DELIVERY_CHANGED, delivery),
        Event::TaskChanged(task) => note(notification::TASK_CHANGED, task),
        Event::RoutineChanged(routine) => note(notification::ROUTINE_CHANGED, routine),
        Event::RoutineRun(run) => note(notification::ROUTINE_RUN, run),
        Event::BrowserChanged(state) => note(notification::BROWSER_CHANGED, state),
        Event::BrowserAction(action) => note(notification::BROWSER_ACTION, action),
        Event::ScreenDraft(draft) => note(notification::SCREEN_DRAFT, draft),
        Event::QuestionChanged(question) => note(notification::QUESTION_CHANGED, question),
    }
}
