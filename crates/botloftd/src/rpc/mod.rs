//! The app's connection: one WebSocket per client, JSON-RPC 2.0 over text
//! frames (spec 11). The first request must be `session.hello`.

mod desktop_watch;
mod dispatch;
mod hello;
pub mod jsonrpc;
mod notify;
mod watching;

use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use botloft_core::protocol::{error_code, method};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use tokio::sync::{broadcast, mpsc, watch};
use tracing::{debug, info, warn};

use self::desktop_watch::DesktopWatch;
use self::hello::{Peer, authenticate};
use self::jsonrpc::RpcError;
use self::notify::to_notification;
use self::watching::Watch;
use crate::state::{Daemon, Event};

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
    if let Some(peer) = authenticate(&mut stream, &outbox, &daemon).await {
        // An open app here is part of the owner being there (spec 24.8);
        // one on another computer is not.
        let here = peer == Peer::Owner;
        if here {
            daemon.desktop.app_opened();
        }
        run(stream, &outbox, &frames, events, &daemon, &peer).await;
        if here {
            daemon.desktop.app_closed();
        }
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

async fn run(
    mut stream: SplitStream<WebSocket>,
    outbox: &mpsc::Sender<String>,
    frames: &watch::Sender<Option<String>>,
    mut events: broadcast::Receiver<Event>,
    daemon: &Arc<Daemon>,
    peer: &Peer,
) {
    let mut watch = Watch::default();
    let mut desktop = DesktopWatch::default();
    loop {
        let frame = tokio::select! {
            message = stream.next() => match message {
                Some(Ok(Message::Text(text))) => match route(daemon, peer, &text, &mut watch, &mut desktop, frames) {
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
                // What was already queued, the answer to the revoke
                // included, still goes out before the close.
                Ok(Event::DeviceRevoked(ref device)) if peer.is(device) => {
                    info!("closing the connection of a revoked device");
                    return;
                }
                Ok(event) => to_notification(&event),
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
            frame = desktop.next_frame() => {
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

/// Requests that the app never sets against notifications: their answer may
/// come after newer notifications, so they wait neither for the requests
/// before them nor hold back the notifications meanwhile.
const ASIDE: &[&str] = &[
    method::FILES_LIST,
    method::FILES_READ,
    method::FILES_MEDIA,
    // Pieces of a file for an app on another computer (spec 42.7).
    method::FILES_CHUNK,
    method::BACKUP_CHUNK,
    method::SCREENS_LIST,
    method::ATTACHMENTS_READ,
    method::USAGE_TOKENS,
    method::USAGE_COSTS,
    method::CHAT_SEARCH,
    // Reading the owner's whole history takes a while (spec 36.7).
    method::IMPORTS_SCAN,
    method::IMPORTS_CREATE,
    // They wait for the network, a copy's transfer for minutes (spec 27.5).
    method::CLOUD_STATUS,
    method::CLOUD_SIGNIN,
    method::CLOUD_SIGNOUT,
    method::CLOUD_UPLOAD,
    method::CLOUD_COPIES,
    method::CLOUD_DOWNLOAD,
    method::CLOUD_DELETE,
    method::CLOUD_DELETE_ACCOUNT,
    // The phone: they ask the server (spec 28.5).
    method::MOBILE_PAIR_START,
    method::MOBILE_PAIR_CANCEL,
    method::MOBILE_PAIR_CONFIRM,
    method::MOBILE_REVOKE,
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
    peer: &Peer,
    text: &str,
    watch: &mut Watch,
    desktop: &mut DesktopWatch,
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
    // A new device is paired where Botloft runs, so a device that leaks
    // cannot make more of itself (spec 42.6).
    if request.method == method::DEVICES_PAIR && *peer != Peer::Owner {
        let refused = Err(crate::service::ApiError::Rule {
            reason: "pair_where_botloft_runs",
            message: "a new device is paired on the computer where Botloft runs, with \
                      botloftd pair"
                .to_owned(),
        }
        .into());
        return Routed::Reply(Some(reply(&id, refused)));
    }
    // Nobody sits at a server's screen (spec 42.2).
    if daemon.server
        && let Some(what) = crate::server_mode::needs_screen(&request.method)
    {
        let refused = Err(crate::service::ApiError::OnAServer(what).into());
        return Routed::Reply(Some(reply(&id, refused)));
    }
    // Watching a browser belongs to this connection (spec 21.7).
    let result = match request.method.as_str() {
        // Answered here, ahead of nothing: the app hears that the
        // connection lives (spec 11).
        method::SESSION_PING => Ok(Value::Null),
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
        | method::BROWSER_OPEN
        | method::BROWSER_TEACH => watch.request(daemon, &request.method, request.params),
        // So is watching a bot's desktop panel (spec 24.9).
        method::DESKTOP_WATCH | method::DESKTOP_UNWATCH => {
            frames.send_replace(None);
            desktop.request(daemon, &request.method, request.params)
        }
        // A window of its own outlives the request (spec 21.11).
        method::BROWSER_WINDOW => watch.window(daemon, request.params),
        // The browser starting outlives the request too (spec 21.10).
        method::BROWSER_START => watch.start(daemon, request.params),
        // So does a login window (spec 35.2).
        method::LOGINS_OPEN => logins_open(daemon, request.params),
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

/// `logins.open`: its window outlives the request (spec 35.2).
fn logins_open(daemon: &Arc<Daemon>, params: Option<Value>) -> Result<Value, RpcError> {
    let params = serde_json::from_value(params.unwrap_or(Value::Null))
        .map_err(|err| RpcError::new(error_code::INVALID_PARAMS, err.to_string()))?;
    let opened = crate::logins::open(daemon, params)?;
    serde_json::to_value(opened)
        .map_err(|err| RpcError::new(error_code::INTERNAL_ERROR, err.to_string()))
}
