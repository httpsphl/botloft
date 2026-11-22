//! `session.hello`, the first request of every connection (spec 11.1): the
//! owner's token or a device's own (spec 42.6), and what the app says about
//! the owner.

use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use botloft_core::ids::DeviceId;
use botloft_core::protocol::{HelloParams, HelloResult, PROTOCOL_VERSION, error_code, method};
use futures_util::StreamExt;
use futures_util::stream::SplitStream;
use serde_json::Value;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use super::jsonrpc::{self, RpcError};
use crate::secrets::TokenHash;
use crate::state::Daemon;

/// How long a new connection may take to send `session.hello`.
const HELLO_TIMEOUT: Duration = Duration::from_secs(10);

/// Who is at the other end of a connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Peer {
    /// The owner's token: the app on this computer, or `botloftd pair`.
    Owner,
    /// A device with a token of its own, from another computer (spec 42.6).
    Device(DeviceId),
}

impl Peer {
    /// Whether this is `device`.
    pub(super) fn is(&self, device: &DeviceId) -> bool {
        matches!(self, Self::Device(own) if own == device)
    }
}

pub(super) async fn authenticate(
    stream: &mut SplitStream<WebSocket>,
    outbox: &mpsc::Sender<String>,
    daemon: &Daemon,
) -> Option<Peer> {
    let Ok(Some(Ok(Message::Text(text)))) =
        tokio::time::timeout(HELLO_TIMEOUT, stream.next()).await
    else {
        return None;
    };
    let request = match jsonrpc::parse(&text) {
        Ok(request) => request,
        Err((id, err)) => {
            let _ = outbox.send(jsonrpc::failure(&id, &err)).await;
            return None;
        }
    };
    let id = request.id.clone().unwrap_or(Value::Null);
    let (frame, peer) = match hello(daemon, request) {
        Ok((ok, peer)) => (
            jsonrpc::success(&id, serde_json::to_value(ok).unwrap_or(Value::Null)),
            Some(peer),
        ),
        Err(err) => (jsonrpc::failure(&id, &err), None),
    };
    let _ = outbox.send(frame).await;
    peer
}

fn hello(daemon: &Daemon, request: jsonrpc::Request) -> Result<(HelloResult, Peer), RpcError> {
    let unauthenticated = |message: &str| RpcError::new(error_code::NOT_AUTHENTICATED, message);
    if request.method != method::SESSION_HELLO {
        return Err(unauthenticated("call session.hello first"));
    }
    let params: HelloParams = serde_json::from_value(request.params.unwrap_or(Value::Null))
        .map_err(|_| unauthenticated("session.hello needs token, client and protocol"))?;
    let peer = if daemon.is_owner_token(&params.token) {
        Peer::Owner
    } else if let Some(device) = known_device(daemon, &params.token) {
        Peer::Device(device)
    } else {
        warn!(client = %params.client.name, "rejected a connection with a wrong token");
        return Err(unauthenticated("invalid token"));
    };
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
    if let Some(locale) = &params.client.locale {
        daemon.desktop.set_locale(locale);
        crate::service::maker::localize(daemon, locale);
    }
    if let Some(zone) = &params.client.time_zone {
        keep_owner_zone(daemon, zone);
    }
    let device_id = match &peer {
        Peer::Owner => None,
        Peer::Device(device) => Some(device.clone()),
    };
    let result = HelloResult {
        daemon_version: env!("CARGO_PKG_VERSION").to_owned(),
        protocol: PROTOCOL_VERSION,
        device_id,
    };
    Ok((result, peer))
}

/// The device whose own token this is, now seen (spec 42.6).
fn known_device(daemon: &Daemon, token: &str) -> Option<DeviceId> {
    let hash = TokenHash::of(token).to_hex();
    match daemon.store().device_seen(&hash, daemon.clock.now_ms()) {
        Ok(device) => device,
        Err(err) => {
            warn!(%err, "could not look up a device's token");
            None
        }
    }
}

/// Keeps the zone the app says the owner is in, for the routines and waits
/// the bots make (spec 20.12, 42.2). A name jiff does not know is left out.
fn keep_owner_zone(daemon: &Daemon, zone: &str) {
    if jiff::tz::TimeZone::get(zone).is_err() {
        debug!(zone, "the app sent a time zone this daemon does not know");
        return;
    }
    let store = daemon.store();
    if store.owner_time_zone().ok().flatten().as_deref() == Some(zone) {
        return;
    }
    if let Err(err) = store.set_owner_time_zone(zone) {
        warn!("could not keep the owner's time zone: {err}");
    }
}
