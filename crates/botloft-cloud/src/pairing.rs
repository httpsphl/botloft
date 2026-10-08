//! Connecting a phone (spec 28.3): the computer opens a pairing, the phone
//! joins with its public key and a proof, the computer accepts, and the phone
//! collects its token once. The server carries these values and checks their
//! shape; it cannot forge a proof, because it never sees the QR's secret.

use std::net::SocketAddr;

use axum::Json;
use axum::extract::{ConnectInfo, Extension, Path, State};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::app::AppState;
use crate::error::ApiError;
use crate::limits::{self, PAIR_PER_ADDRESS};
use crate::login::{clean_device, client_address};
use crate::pairing_rows::{self as rows, Collected, Join, PAIR_TTL_MS, Seen};
use crate::session::Computer;
use crate::tokens;

/// A base64url value without padding of exactly `len` characters, or at most
/// `max` when `len` is 0.
fn b64url(text: &str, len: usize, max: usize) -> bool {
    let shaped = text
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_');
    shaped
        && if len > 0 {
            text.len() == len
        } else {
            !text.is_empty() && text.len() <= max
        }
}

/// 16 bytes, which is 22 characters.
const ID_LEN: usize = 22;
/// A 32-byte HMAC.
const PROOF_LEN: usize = 43;
/// An uncompressed P-256 point is 65 bytes, 87 characters; leave room.
const KEY_MAX: usize = 100;

fn tell_computer(state: &AppState, computer: &str, id: &str) {
    state
        .hub
        .send(computer, json!({ "t": "pairing", "id": id }).to_string());
}

#[derive(Deserialize)]
pub struct Open {
    id: String,
}

/// `POST /v1/pairings`: the computer opens a pairing for its QR code.
pub async fn open(
    State(state): State<AppState>,
    Computer(who): Computer,
    Json(Open { id }): Json<Open>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if !b64url(&id, ID_LEN, 0) {
        return Err(ApiError::bad_pairing());
    }
    let now = state.clock.now();
    let opened = state
        .db
        .run(|conn| rows::open(conn, &id, who.account_id, &who.device_id, now))?;
    if !opened {
        return Err(ApiError::rate_limited(60));
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({ "expires_in": PAIR_TTL_MS / 1000 })),
    ))
}

/// `GET /v1/pairings/<id>`: the computer looks at whether a phone joined.
pub async fn status(
    State(state): State<AppState>,
    Computer(who): Computer,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let now = state.clock.now();
    let seen = state
        .db
        .run(|conn| rows::seen(conn, &id, &who.device_id, now))?;
    Ok(Json(match seen {
        None => json!({ "status": "expired" }),
        Some(Seen::Waiting) => json!({ "status": "waiting" }),
        Some(Seen::Joined {
            phone_name,
            phone_pub,
            proof,
        }) => json!({
            "status": "joined",
            "device_name": phone_name,
            "phone_pub": phone_pub,
            "proof": proof,
        }),
    }))
}

#[derive(Deserialize)]
pub struct JoinBody {
    phone_pub: String,
    #[serde(default)]
    device_name: String,
    proof: String,
}

/// `POST /v1/pairings/<id>/join`: the phone, which has no token yet. The
/// answer holds a secret only this phone knows, for collecting its token.
pub async fn join(
    State(state): State<AppState>,
    headers: HeaderMap,
    connection: Option<Extension<ConnectInfo<SocketAddr>>>,
    Path(id): Path<String>,
    Json(body): Json<JoinBody>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if !b64url(&id, ID_LEN, 0)
        || !b64url(&body.phone_pub, 0, KEY_MAX)
        || !b64url(&body.proof, PROOF_LEN, 0)
    {
        return Err(ApiError::bad_pairing());
    }
    let address = client_address(&state, &headers, connection.map(|Extension(info)| info));
    let (key, now) = (tokens::hash(&address), state.clock.now());
    let taken = state
        .db
        .run(|conn| limits::take(conn, now, &[(&PAIR_PER_ADDRESS, &key)]))?;
    taken.map_err(ApiError::rate_limited)?;

    let poll = tokens::random()?;
    let name = clean_device(&body.device_name);
    let joined = state.db.run(|conn| {
        rows::join(
            conn,
            &id,
            &Join {
                phone_pub: &body.phone_pub,
                phone_name: &name,
                proof: &body.proof,
                poll_hash: &tokens::hash(&poll),
            },
            now,
        )
    })?;
    let computer = joined.ok_or_else(ApiError::pair_expired)?;
    tell_computer(&state, &computer, &id);
    Ok((StatusCode::ACCEPTED, Json(json!({ "poll": poll }))))
}

#[derive(Deserialize)]
pub struct Accept {
    daemon_pub: String,
    proof2: String,
}

/// `POST /v1/pairings/<id>/accept`: the computer accepts the phone that joined.
pub async fn accept(
    State(state): State<AppState>,
    Computer(who): Computer,
    Path(id): Path<String>,
    Json(body): Json<Accept>,
) -> Result<StatusCode, ApiError> {
    if !b64url(&body.daemon_pub, 0, KEY_MAX) || !b64url(&body.proof2, PROOF_LEN, 0) {
        return Err(ApiError::bad_pairing());
    }
    let now = state.clock.now();
    let accepted = state.db.run(|conn| {
        rows::accept(
            conn,
            &id,
            &who.device_id,
            &body.daemon_pub,
            &body.proof2,
            now,
        )
    })?;
    if accepted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found())
    }
}

/// `DELETE /v1/pairings/<id>`: the computer cancels, or refuses the phone.
pub async fn cancel(
    State(state): State<AppState>,
    Computer(who): Computer,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let removed = state
        .db
        .run(|conn| rows::remove(conn, &id, &who.device_id))?;
    if removed {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found())
    }
}

/// `GET /v1/pairings/<id>/result`: the phone polls with its secret as the
/// bearer. Once the computer accepted, the first poll takes the token.
pub async fn result(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let poll = headers
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .ok_or_else(ApiError::unauthorized)?;
    let (poll_hash, now) = (tokens::hash(poll.trim()), state.clock.now());
    let found = state
        .db
        .run(|conn| rows::collect(conn, &id, &poll_hash, now))?;
    match found {
        Collected::Gone => Ok(Json(json!({ "status": "expired" }))),
        Collected::Pending => Ok(Json(json!({ "status": "pending" }))),
        Collected::Ready(ready) => {
            let (token, device) = (
                tokens::random()?,
                format!("dev_{}", &tokens::random()?[..16]),
            );
            let token_hash = tokens::hash(&token);
            let finished = state
                .db
                .run(|conn| rows::finish(conn, &id, &ready, &device, &token_hash, now))?;
            if !finished {
                // Two polls at once: the other one took it.
                return Ok(Json(json!({ "status": "expired" })));
            }
            // The computer learns the phone's id, which only the phone had.
            state.hub.send(
                &ready.computer,
                json!({ "t": "paired", "pairing": id, "device": device }).to_string(),
            );
            Ok(Json(json!({
                "status": "approved",
                "token": token,
                "device": device,
                "peer": ready.computer,
                "daemon_pub": ready.daemon_pub,
                "proof2": ready.proof2,
            })))
        }
    }
}
