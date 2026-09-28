//! HTTP server on 127.0.0.1: `GET /health`, the `/rpc` WebSocket for the
//! app, and `POST /hooks/{event}` and `POST /mcp` for the bots.

use std::future::Future;
use std::sync::Arc;

use axum::Router;
use axum::extract::{DefaultBodyLimit, State, WebSocketUpgrade};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use botloft_core::protocol::PROTOCOL_VERSION;
use serde_json::json;
use tokio::net::TcpListener;

use crate::state::Daemon;
use crate::{hooks, rpc, tools};

/// Browser origins allowed to open `/rpc` (spec 11.1): the Tauri app in
/// production and the Vite dev server. Clients that send no `Origin`, such
/// as native tools and tests, are allowed; they still need the owner token.
pub const ALLOWED_ORIGINS: &[&str] = &[
    "http://tauri.localhost",
    "tauri://localhost",
    "http://localhost:1420",
];

/// Largest WebSocket message accepted from the app.
const MAX_MESSAGE_BYTES: usize = 4 << 20;
/// Largest hook body; payloads are small JSON objects.
const MAX_HOOK_BYTES: usize = 1 << 20;
/// Largest MCP request: a message of 100 000 characters fits with room.
const MAX_MCP_BYTES: usize = 2 << 20;

pub fn router(daemon: Arc<Daemon>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/rpc", get(rpc_upgrade))
        .route(
            "/hooks/{event}",
            post(hooks::handle).layer(DefaultBodyLimit::max(MAX_HOOK_BYTES)),
        )
        .route(
            "/mcp",
            post(tools::handle)
                .get(tools::not_allowed)
                .delete(tools::not_allowed)
                .layer(DefaultBodyLimit::max(MAX_MCP_BYTES)),
        )
        .with_state(daemon)
}

/// Serves until `shutdown` resolves.
pub async fn serve(
    daemon: Arc<Daemon>,
    listener: TcpListener,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> std::io::Result<()> {
    axum::serve(listener, router(daemon))
        .with_graceful_shutdown(shutdown)
        .await
}

async fn health() -> impl IntoResponse {
    axum::Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "protocol": PROTOCOL_VERSION,
    }))
}

async fn rpc_upgrade(
    State(daemon): State<Arc<Daemon>>,
    headers: HeaderMap,
    upgrade: WebSocketUpgrade,
) -> Response {
    if let Some(origin) = headers.get(header::ORIGIN) {
        let allowed = origin
            .to_str()
            .is_ok_and(|origin| ALLOWED_ORIGINS.contains(&origin));
        if !allowed {
            return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
        }
    }
    upgrade
        .max_message_size(MAX_MESSAGE_BYTES)
        .on_upgrade(move |socket| rpc::serve_connection(socket, daemon))
}
