//! What every endpoint answers when it fails: `{reason, message}` (spec 27.4).

use axum::Json;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    reason: &'static str,
    message: &'static str,
    retry_after: Option<u64>,
}

impl ApiError {
    const fn new(status: StatusCode, reason: &'static str, message: &'static str) -> Self {
        Self {
            status,
            reason,
            message,
            retry_after: None,
        }
    }

    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Sign in again to use this account.",
        )
    }

    pub fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", "There is nothing here.")
    }

    pub fn wrong_device() -> Self {
        Self::new(
            StatusCode::FORBIDDEN,
            "wrong_device",
            "This device is not allowed to do that.",
        )
    }

    pub fn bad_pairing() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "bad_pairing",
            "That is not a valid pairing request.",
        )
    }

    pub fn pair_expired() -> Self {
        Self::new(
            StatusCode::GONE,
            "pair_expired",
            "This code was used or has run out. Ask for a new one.",
        )
    }

    pub fn no_push() -> Self {
        Self::new(
            StatusCode::NOT_FOUND,
            "no_push",
            "This server does not send notices.",
        )
    }

    pub fn bad_endpoint() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "bad_endpoint",
            "That is not an address of a push service this server uses.",
        )
    }

    pub fn bad_email() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "bad_email",
            "That does not look like an e-mail address.",
        )
    }

    pub fn rate_limited(retry_after: u64) -> Self {
        Self {
            retry_after: Some(retry_after),
            ..Self::new(
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Too many tries. Wait a little and try again.",
            )
        }
    }

    pub fn mail_failed() -> Self {
        Self::new(
            StatusCode::BAD_GATEWAY,
            "mail_failed",
            "The e-mail could not be sent. Try again in a moment.",
        )
    }

    pub fn length_required() -> Self {
        Self::new(
            StatusCode::LENGTH_REQUIRED,
            "length_required",
            "The upload must say how big it is.",
        )
    }

    pub fn bad_length() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "bad_length",
            "The upload is not the size it said, or it was cut off.",
        )
    }

    pub fn bad_hash() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "bad_hash",
            "The upload does not match its SHA-256.",
        )
    }

    pub fn too_big() -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "too_big",
            "That copy is bigger than a copy may be.",
        )
    }

    pub fn quota() -> Self {
        Self::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "quota",
            "The account has no room for this copy. Delete an old one first.",
        )
    }

    pub fn range_not_satisfiable() -> Self {
        Self::new(
            StatusCode::RANGE_NOT_SATISFIABLE,
            "bad_range",
            "That part of the copy does not exist.",
        )
    }

    pub fn internal() -> Self {
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "Something went wrong on our side.",
        )
    }

    pub fn reason(&self) -> &'static str {
        self.reason
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        // Never the query or the values: they can hold an e-mail address.
        tracing::error!("database: {err}");
        Self::internal()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut body = json!({ "reason": self.reason, "message": self.message });
        if let Some(seconds) = self.retry_after {
            body["retry_after"] = seconds.into();
        }
        let mut response = (self.status, Json(body)).into_response();
        if let Some(seconds) = self.retry_after
            && let Ok(value) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }
        response
    }
}
