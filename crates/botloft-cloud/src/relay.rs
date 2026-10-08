//! The relay (spec 28.4): `GET /v1/relay` is a WebSocket where the computer
//! and its phones talk. The server routes sealed `body` strings between a
//! phone and the computer it was connected with, and never opens them.
//!
//! Browsers cannot put a header on a WebSocket, so the token goes in the first
//! frame (`{t: "hello", token}`), never in the address.

use std::collections::VecDeque;
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio::time::{MissedTickBehavior, interval, timeout};

use crate::app::AppState;
use crate::devices::{self, Kind, Who};
use crate::hub::Out;
use crate::relay_rows::{self as rows, Pushed};
use crate::tokens;

/// The biggest frame, in either direction.
const MAX_FRAME: usize = 64 * 1024;
/// How long a new socket has to say who it is.
const HELLO_WAIT: Duration = Duration::from_secs(10);
/// How often the server pings, so proxies keep the socket open.
const PING_EVERY: Duration = Duration::from_secs(25);
/// Frames a device may send in a minute.
const FRAMES_PER_MINUTE: usize = 60;

/// `GET /v1/relay`
pub async fn relay(State(state): State<AppState>, upgrade: WebSocketUpgrade) -> Response {
    upgrade
        .max_message_size(MAX_FRAME)
        .max_frame_size(MAX_FRAME)
        .on_upgrade(move |socket| serve(state, socket))
}

async fn put(socket: &mut WebSocket, frame: Value) -> bool {
    socket
        .send(Message::Text(frame.to_string().into()))
        .await
        .is_ok()
}

/// The first frame, checked against the devices.
async fn hello(state: &AppState, socket: &mut WebSocket) -> Option<Who> {
    let first = timeout(HELLO_WAIT, socket.recv()).await.ok()??.ok()?;
    let Message::Text(text) = first else {
        return None;
    };
    let frame: Value = serde_json::from_str(&text).ok()?;
    let token = frame
        .get("t")
        .filter(|kind| *kind == "hello")
        .and(frame.get("token"))
        .and_then(Value::as_str)?;
    let (hash, now) = (tokens::hash(token.trim()), state.clock.now());
    let who = state
        .db
        .run(|conn| devices::who_is(conn, &hash, now))
        .ok()
        .flatten();
    if who.is_none() {
        let _ = put(socket, json!({ "t": "error", "reason": "unauthorized" })).await;
    }
    who
}

async fn serve(state: AppState, mut socket: WebSocket) {
    let Some(who) = hello(&state, &mut socket).await else {
        return;
    };
    let (tx, mut outgoing) = mpsc::channel::<Out>(256);
    let me = state.hub.attach(&who.device_id, tx);
    if !ready(&state, &who, &mut socket).await {
        state.hub.detach(&who.device_id, me);
        return;
    }
    presence(&state, &who, true);

    let mut ping = interval(PING_EVERY);
    ping.set_missed_tick_behavior(MissedTickBehavior::Delay);
    ping.tick().await;
    let mut recent: VecDeque<i64> = VecDeque::new();
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    if !within_limit(&state, &mut recent) {
                        let _ = put(&mut socket, json!({"t": "error", "reason": "rate_limited"})).await;
                        continue;
                    }
                    if let Some(reply) = handle(&state, &who, &text) {
                        let _ = put(&mut socket, reply).await;
                    }
                }
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            out = outgoing.recv() => match out {
                Some(Out::Text(text)) => {
                    if socket.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                }
                Some(Out::Close) | None => {
                    let _ = socket.send(Message::Close(None)).await;
                    break;
                }
            },
            _ = ping.tick() => {
                if !put(&mut socket, json!({"t": "ping"})).await {
                    break;
                }
            }
        }
    }
    state.hub.detach(&who.device_id, me);
    // A newer socket of the same device may have taken over: then it is still there.
    if !state.hub.online(&who.device_id) {
        presence(&state, &who, false);
    }
}

/// At most `FRAMES_PER_MINUTE` frames in any minute (the server's clock).
fn within_limit(state: &AppState, recent: &mut VecDeque<i64>) -> bool {
    let now = state.clock.now();
    while recent.front().is_some_and(|at| *at <= now - 60_000) {
        recent.pop_front();
    }
    if recent.len() >= FRAMES_PER_MINUTE {
        return false;
    }
    recent.push_back(now);
    true
}

/// `{t: "ready"}`, then (for a computer) what its phones sent while it was away.
async fn ready(state: &AppState, who: &Who, socket: &mut WebSocket) -> bool {
    match who.kind {
        Kind::Phone => {
            let peer = who.peer.clone().unwrap_or_default();
            let online = state.hub.online(&peer);
            put(
                socket,
                json!({
                    "t": "ready", "kind": "phone", "device": who.device_id,
                    "peer": peer, "online": online,
                }),
            )
            .await
        }
        Kind::Computer => {
            let now = state.clock.now();
            let loaded = state.db.run(|conn| {
                Ok((
                    rows::phones_of(conn, &who.device_id)?,
                    rows::waiting(conn, &who.device_id, now)?,
                ))
            });
            let Ok((phones, waiting)) = loaded else {
                return false;
            };
            let phones: Vec<Value> = phones
                .into_iter()
                .map(|id| json!({ "online": state.hub.online(&id), "id": id }))
                .collect();
            let first = json!({
                "t": "ready", "kind": "computer", "device": who.device_id, "phones": phones,
            });
            if !put(socket, first).await {
                return false;
            }
            for (from, seq, body) in waiting {
                let frame = json!({ "t": "msg", "from": from, "seq": seq, "body": body });
                if !put(socket, frame).await {
                    return false;
                }
            }
            true
        }
    }
}

/// Tells the other side that this device came or went.
fn presence(state: &AppState, who: &Who, online: bool) {
    let frame = |device: &str| json!({ "t": "presence", "device": device, "online": online });
    match who.kind {
        Kind::Phone => {
            if let Some(peer) = &who.peer {
                state.hub.send(peer, frame(&who.device_id).to_string());
            }
        }
        Kind::Computer => {
            let phones = state
                .db
                .run(|conn| rows::phones_of(conn, &who.device_id))
                .unwrap_or_default();
            for phone in phones {
                state.hub.send(&phone, frame(&who.device_id).to_string());
            }
        }
    }
}

/// One frame from a device. Gives back an error frame for the sender, if any.
fn handle(state: &AppState, who: &Who, text: &str) -> Option<Value> {
    let Ok(frame) = serde_json::from_str::<Value>(text) else {
        return Some(json!({ "t": "error", "reason": "bad_frame" }));
    };
    let kind = frame.get("t").and_then(Value::as_str).unwrap_or_default();
    let seq = frame.get("seq").and_then(Value::as_i64).filter(|s| *s >= 0);
    let body = frame.get("body").and_then(Value::as_str);
    match (who.kind, kind) {
        (Kind::Phone, "msg") => {
            let (Some(seq), Some(body), Some(peer)) = (seq, body, &who.peer) else {
                return Some(json!({ "t": "error", "reason": "bad_frame" }));
            };
            let now = state.clock.now();
            match state
                .db
                .run(|conn| rows::push(conn, &who.device_id, peer, seq, body, now))
            {
                Ok(Pushed::Queued) => {
                    let out =
                        json!({ "t": "msg", "from": who.device_id, "seq": seq, "body": body });
                    state.hub.send(peer, out.to_string());
                    None
                }
                Ok(Pushed::Again) => None,
                Ok(Pushed::Full) => {
                    Some(json!({ "t": "error", "reason": "queue_full", "seq": seq }))
                }
                Err(_) => Some(json!({ "t": "error", "reason": "internal" })),
            }
        }
        (Kind::Computer, "msg") => {
            let (Some(seq), Some(body), Some(to)) =
                (seq, body, frame.get("to").and_then(Value::as_str))
            else {
                return Some(json!({ "t": "error", "reason": "bad_frame" }));
            };
            let known = state
                .db
                .run(|conn| rows::is_phone_of(conn, to, &who.device_id))
                .unwrap_or(false);
            if !known {
                return Some(json!({ "t": "error", "reason": "unknown_device", "seq": seq }));
            }
            let out = json!({ "t": "msg", "seq": seq, "body": body });
            state.hub.send(to, out.to_string());
            None
        }
        (Kind::Computer, "ack") => {
            let (Some(upto), Some(from)) = (
                frame.get("upto").and_then(Value::as_i64),
                frame.get("from").and_then(Value::as_str),
            ) else {
                return Some(json!({ "t": "error", "reason": "bad_frame" }));
            };
            let _ = state
                .db
                .run(|conn| rows::ack(conn, &who.device_id, from, upto));
            None
        }
        // Something waits for the owner: the phones that are not looking are told.
        (Kind::Computer, "wake") => {
            state.push.wake(state, &who.device_id);
            None
        }
        // Pings and pongs need no answer.
        _ => None,
    }
}
