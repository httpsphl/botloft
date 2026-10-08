//! What a phone asks of the push (spec 28.8): the server's public key, and to
//! be told, or not, through the address its browser gave it.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app::AppState;
use crate::devices::Kind;
use crate::error::ApiError;
use crate::push;
use crate::session::Authed;

/// Only a phone, and only if the server sends pushes at all.
fn phone(state: &AppState, who: &crate::devices::Who) -> Result<(), ApiError> {
    if who.kind != Kind::Phone {
        return Err(ApiError::wrong_device());
    }
    if state.push.public_key().is_none() {
        return Err(ApiError::no_push());
    }
    Ok(())
}

/// `GET /v1/push/key`: what the browser needs to subscribe to this server.
pub async fn key(
    State(state): State<AppState>,
    Authed(who): Authed,
) -> Result<Json<Value>, ApiError> {
    phone(&state, &who)?;
    Ok(Json(json!({ "key": state.push.public_key() })))
}

#[derive(Deserialize)]
pub struct Subscription {
    endpoint: String,
}

/// `POST /v1/push/subscribe`: the address of this phone's push service.
pub async fn subscribe(
    State(state): State<AppState>,
    Authed(who): Authed,
    Json(body): Json<Subscription>,
) -> Result<StatusCode, ApiError> {
    phone(&state, &who)?;
    if body.endpoint.len() > 2048 || !state.push.allows(&body.endpoint) {
        return Err(ApiError::bad_endpoint());
    }
    let now = state.clock.now();
    state
        .db
        .run(|conn| push::subscribe(conn, &who.device_id, &body.endpoint, now))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /v1/push/subscribe`
pub async fn unsubscribe(
    State(state): State<AppState>,
    Authed(who): Authed,
) -> Result<StatusCode, ApiError> {
    if who.kind != Kind::Phone {
        return Err(ApiError::wrong_device());
    }
    state.db.run(|conn| push::forget(conn, &who.device_id))?;
    Ok(StatusCode::NO_CONTENT)
}
