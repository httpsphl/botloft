//! Who is asking, once signed in (spec 27.3): the bearer token of a device.
//! A phone's token reaches the relay and little else (spec 28.2).

use axum::Json;
use axum::extract::{FromRequestParts, Path, State};
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use serde_json::{Value, json};

use crate::app::AppState;
use crate::copy_rows;
use crate::devices::{self, Kind, Removed, Who};
use crate::error::ApiError;
use crate::tokens;

/// The device behind a request, whatever it is.
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
        let who = state.db.run(|conn| devices::who_is(conn, &hash, now))?;
        who.map(Self).ok_or_else(ApiError::unauthorized)
    }
}

/// A computer: the copies, the other devices and the account are only its.
pub struct Computer(pub Who);

impl FromRequestParts<AppState> for Computer {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let Authed(who) = Authed::from_request_parts(parts, state).await?;
        if who.kind == Kind::Computer {
            Ok(Self(who))
        } else {
            Err(ApiError::wrong_device())
        }
    }
}

/// `GET /v1/me`. A phone sees only whose it is and which computer it serves.
pub async fn me(
    State(state): State<AppState>,
    Authed(who): Authed,
) -> Result<Json<Value>, ApiError> {
    if who.kind == Kind::Phone {
        return Ok(Json(json!({
            "email": who.email,
            "kind": who.kind.tag(),
            "device": who.device_id,
            "peer": who.peer,
        })));
    }
    let (devices, used) = state.db.run(|conn| {
        Ok((
            devices::devices(conn, who.account_id)?,
            copy_rows::used(conn, who.account_id)?,
        ))
    })?;
    let devices: Vec<Value> = devices
        .into_iter()
        .map(|device| {
            json!({
                "id": device.id,
                "name": device.name,
                "kind": device.kind,
                "peer": device.peer,
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

/// Closes the sockets of devices that are gone, and tells the computer of
/// each phone that left (spec 28.4), so it forgets that phone's keys.
pub fn farewell(state: &AppState, gone: &[Removed]) {
    for device in gone {
        state.hub.kick(&device.id);
    }
    for device in gone {
        if let Some(peer) = &device.peer {
            state.hub.send(
                peer,
                json!({ "t": "revoked", "device": device.id }).to_string(),
            );
        }
    }
}

/// `POST /v1/logout`: this device's token stops working (a phone's too, which
/// is how it disconnects from its side).
pub async fn logout(
    State(state): State<AppState>,
    Authed(who): Authed,
) -> Result<StatusCode, ApiError> {
    let gone = state
        .db
        .run(|conn| devices::delete_device(conn, who.account_id, &who.device_id))?;
    farewell(&state, &gone);
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /v1/devices/<id>`: another device of the same account, with the
/// phones that belong to it.
pub async fn remove_device(
    State(state): State<AppState>,
    Computer(who): Computer,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let gone = state
        .db
        .run(|conn| devices::delete_device(conn, who.account_id, &id))?;
    if gone.is_empty() {
        return Err(ApiError::not_found());
    }
    farewell(&state, &gone);
    Ok(StatusCode::NO_CONTENT)
}
