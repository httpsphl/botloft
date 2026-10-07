//! Who is asking, once signed in (spec 27.3): the bearer token of a device.

use axum::Json;
use axum::extract::{FromRequestParts, Path, State};
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use serde_json::{Value, json};

use crate::accounts::{self, Who};
use crate::app::AppState;
use crate::copy_rows;
use crate::error::ApiError;
use crate::tokens;

/// The device behind a request.
pub struct Authed(pub Who);

impl FromRequestParts<AppState> for Authed {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .ok_or_else(ApiError::unauthorized)?;
        let (hash, now) = (tokens::hash(token.trim()), state.clock.now());
        let who = state.db.run(|conn| accounts::who_is(conn, &hash, now))?;
        who.map(Self).ok_or_else(ApiError::unauthorized)
    }
}

/// `GET /v1/me`
pub async fn me(
    State(state): State<AppState>,
    Authed(who): Authed,
) -> Result<Json<Value>, ApiError> {
    let (devices, used) = state.db.run(|conn| {
        Ok((
            accounts::devices(conn, who.account_id)?,
            copy_rows::used(conn, who.account_id)?,
        ))
    })?;
    let devices: Vec<Value> = devices
        .into_iter()
        .map(|device| {
            json!({
                "id": device.id,
                "name": device.name,
                "created_at": device.created_at,
                "last_used_at": device.last_used_at,
                "current": device.id == who.device_id,
            })
        })
        .collect();
    Ok(Json(json!({
        "email": who.email,
        "used": used,
        "quota": state.config.quota_bytes,
        "devices": devices,
    })))
}

/// `POST /v1/logout`: this device's token stops working.
pub async fn logout(
    State(state): State<AppState>,
    Authed(who): Authed,
) -> Result<StatusCode, ApiError> {
    state
        .db
        .run(|conn| accounts::delete_device(conn, who.account_id, &who.device_id))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /v1/devices/<id>`: another device of the same account.
pub async fn remove_device(
    State(state): State<AppState>,
    Authed(who): Authed,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let removed = state
        .db
        .run(|conn| accounts::delete_device(conn, who.account_id, &id))?;
    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found())
    }
}
