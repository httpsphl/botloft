//! The computer's side of the relay (spec 28.4): one WebSocket to the server
//! while a phone is connected or being connected, back again when it drops.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use botloft_core::protocol::MobileRelay;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::mpsc;
use tokio::time::{MissedTickBehavior, interval, timeout};
use tokio_tungstenite::tungstenite::Message;
use tracing::debug;

use super::identity;
use crate::state::Daemon;

/// How long the server has to answer the hello.
const READY_WAIT: Duration = Duration::from_secs(10);

/// Keeps the relay open for as long as it is wanted. Started once, with the
/// daemon.
pub async fn run(daemon: Arc<Daemon>) {
    let mobile = &daemon.mobile;
    let mut delay = mobile.retry.min;
    loop {
        if !mobile.wanted(&daemon) {
            mobile.set_relay(&daemon, MobileRelay::Off);
            mobile.wake.notified().await;
            delay = mobile.retry.min;
            continue;
        }
        mobile.set_relay(&daemon, MobileRelay::Connecting);
        if session(&daemon).await {
            delay = mobile.retry.min;
        }
        mobile.set_live(None);
        if mobile.wanted(&daemon) {
            mobile.set_relay(&daemon, MobileRelay::Connecting);
            tokio::select! {
                () = tokio::time::sleep(delay) => {}
                () = mobile.wake.notified() => {}
            }
            delay = (delay * 2).min(mobile.retry.max);
        }
    }
}

/// One connection, until it ends. True if the server accepted it.
async fn session(daemon: &Arc<Daemon>) -> bool {
    let mobile = &daemon.mobile;
    let Ok((server, credentials)) = identity(daemon) else {
        return false;
    };
    let Ok((socket, _)) = tokio_tungstenite::connect_async(server.relay_url()).await else {
        return false;
    };
    let (mut sink, mut stream) = socket.split();
    let hello = json!({ "t": "hello", "token": credentials.token }).to_string();
    if sink.send(Message::Text(hello.into())).await.is_err() {
        return false;
    }
    let ready = timeout(READY_WAIT, async {
        while let Some(Ok(message)) = stream.next().await {
            if let Message::Text(text) = message {
                return serde_json::from_str::<Value>(&text).ok();
            }
        }
        None
    })
    .await
    .ok()
    .flatten();
    let Some(ready) = ready.filter(|ready| ready["t"] == "ready" && ready["kind"] == "computer")
    else {
        return false;
    };

    let (out, mut outgoing) = mpsc::unbounded_channel::<String>();
    let listed: Vec<(String, bool)> = ready["phones"]
        .as_array()
        .map(|phones| {
            phones
                .iter()
                .filter_map(|phone| {
                    Some((phone["id"].as_str()?.to_owned(), phone["online"] == true))
                })
                .collect()
        })
        .unwrap_or_default();
    let ours: HashSet<String> = mobile.phones(daemon).into_iter().map(|p| p.id).collect();
    mobile.set_online(
        listed
            .iter()
            .filter(|(id, online)| *online && ours.contains(id))
            .map(|(id, _)| id.clone())
            .collect(),
    );
    mobile.set_live(Some(out));
    mobile.set_relay(daemon, MobileRelay::Connected);

    // A phone the server lists and this computer has no keys for was taken
    // off here while the server was out of reach: it goes off there too. (Not
    // while one is being accepted: its device is new, and the keys wait.)
    if !mobile.awaiting_phone() {
        let strangers: Vec<String> = listed
            .into_iter()
            .map(|(id, _)| id)
            .filter(|id| !ours.contains(id))
            .collect();
        for id in strangers {
            let (server, token) = (
                identity(daemon).map(|(server, _)| server),
                credentials.token.clone(),
            );
            tokio::spawn(async move {
                if let Ok(server) = server {
                    let _ = server.remove_device(&token, &id).await;
                }
            });
        }
    }

    let mut events = daemon.subscribe();
    let mut tick = interval(daemon.cloud.settings.poll);
    tick.set_missed_tick_behavior(MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            frame = stream.next() => match frame {
                Some(Ok(Message::Text(text))) => handle(daemon, &text).await,
                Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            Some(text) = outgoing.recv() => {
                if sink.send(Message::Text(text.into())).await.is_err() {
                    break;
                }
            }
            event = events.recv() => match event {
                Ok(event) => mobile.on_event(daemon, &event),
                Err(RecvError::Lagged(_)) => {}
                Err(RecvError::Closed) => break,
            },
            _ = tick.tick() => mobile.check_joined(daemon).await,
            () = mobile.wake.notified() => {
                if !mobile.wanted(daemon) {
                    let _ = sink.send(Message::Close(None)).await;
                    break;
                }
            }
        }
    }
    true
}

/// A frame from the server.
async fn handle(daemon: &Arc<Daemon>, text: &str) {
    let mobile = &daemon.mobile;
    let Ok(frame) = serde_json::from_str::<Value>(text) else {
        return;
    };
    let field = |name: &str| frame[name].as_str();
    match field("t") {
        Some("msg") => {
            if let (Some(from), Some(seq), Some(body)) =
                (field("from"), frame["seq"].as_u64(), field("body"))
            {
                mobile.receive(daemon, from, seq, body).await;
            }
        }
        Some("presence") => {
            if let Some(device) = field("device") {
                mobile.presence(daemon, device, frame["online"] == true);
            }
        }
        Some("pairing") => mobile.check_joined(daemon).await,
        Some("paired") => {
            if let (Some(pairing), Some(device)) = (field("pairing"), field("device")) {
                mobile.paired(daemon, pairing, device);
            }
        }
        Some("revoked") => {
            if let Some(device) = field("device") {
                mobile.revoked(daemon, device);
            }
        }
        Some("error") => debug!(
            reason = field("reason").unwrap_or_default(),
            "the relay refused a frame"
        ),
        _ => {}
    }
}
