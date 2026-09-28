//! The MCP server the bots use (spec 10): `POST /mcp`, authenticated with
//! the bot token of a running generation.
//!
//! It speaks MCP 2026-07-28, where every request is stateless and carries
//! its own version, and the earlier revisions that open with `initialize`.
//! Claude Code tries `server/discover` first and falls back to
//! `initialize` when that fails, so both paths lead to the same tools.

mod calls;
mod catalog;

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::ids::BotId;
use botloft_core::protocol::error_code;
use serde_json::{Value, json};
use tracing::debug;

use crate::rpc::jsonrpc::{self, Request};
use crate::state::Daemon;

/// The revision without `initialize`.
const MODERN: &str = "2026-07-28";
/// Revisions that open with `initialize`, newest first.
const LEGACY: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26"];
/// MCP error codes (2026-07-28).
const HEADER_MISMATCH: i32 = -32020;
const UNSUPPORTED_VERSION: i32 = -32022;
/// How long clients may reuse `server/discover` and `tools/list`.
const CACHE_TTL_MS: u64 = 60 * 60 * 1000;

const INSTRUCTIONS: &str = "Tools to work with your Botloft crew: see who is in it, send notes \
    or tasks to other bots, and report the result of tasks assigned to you. Messages from the \
    owner and from other bots arrive as prompts that start with [botloft].";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Era {
    Modern,
    Legacy,
}

/// A JSON-RPC error with the HTTP status it goes out with.
struct Failure {
    status: StatusCode,
    code: i32,
    message: String,
    data: Option<Value>,
}

impl Failure {
    fn new(status: StatusCode, code: i32, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            data: None,
        }
    }

    fn mismatch(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, HEADER_MISMATCH, message)
    }

    fn into_response(self, id: &Value) -> Response {
        let mut error = json!({ "code": self.code, "message": self.message });
        if let Some(data) = self.data {
            error["data"] = data;
        }
        let body = json!({ "jsonrpc": "2.0", "id": id, "error": error });
        json_response(self.status, body.to_string())
    }
}

/// `POST /mcp`.
pub async fn handle(
    State(daemon): State<Arc<Daemon>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // Bots never send an Origin; a browser always does (DNS rebinding).
    if headers.contains_key(header::ORIGIN) {
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let Some((bot, _generation)) = token.and_then(|token| daemon.supervisor.hook_owner(token))
    else {
        debug!(
            has_token = token.is_some(),
            "mcp request without a live bot token"
        );
        let mut response = StatusCode::UNAUTHORIZED.into_response();
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        return response;
    };
    let parsed = std::str::from_utf8(&body)
        .map_err(|_| {
            (
                Value::Null,
                jsonrpc::RpcError::new(error_code::PARSE_ERROR, "parse error"),
            )
        })
        .and_then(jsonrpc::parse);
    let request = match parsed {
        Ok(request) => request,
        Err((id, err)) => {
            return Failure::new(StatusCode::BAD_REQUEST, err.code, err.message).into_response(&id);
        }
    };
    // A notification (such as `notifications/initialized`) needs no answer.
    let Some(id) = request.id.clone() else {
        debug!(bot = %bot, method = %request.method, "mcp notification");
        return StatusCode::ACCEPTED.into_response();
    };
    let answered = era(&headers, &request).and_then(|era| answer(&daemon, &bot, era, &request));
    // Method names only: arguments carry message bodies.
    match answered {
        Ok(result) => {
            debug!(bot = %bot, method = %request.method, "mcp request");
            json_response(StatusCode::OK, jsonrpc::success(&id, result))
        }
        Err(failure) => {
            debug!(bot = %bot, method = %request.method, status = %failure.status, code = failure.code, "mcp request refused");
            failure.into_response(&id)
        }
    }
}

/// `GET` and `DELETE /mcp`: no server-sent stream and no sessions.
pub async fn not_allowed() -> StatusCode {
    debug!("mcp: refused a GET or DELETE");
    StatusCode::METHOD_NOT_ALLOWED
}

fn json_response(status: StatusCode, body: String) -> Response {
    (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// Which revision the request follows, checking what 2026-07-28 requires.
fn era(headers: &HeaderMap, request: &Request) -> Result<Era, Failure> {
    match header_value(headers, "mcp-protocol-version") {
        // `initialize`, and clients from before the header existed.
        None => Ok(Era::Legacy),
        Some(MODERN) => check_modern(headers, request).map(|()| Era::Modern),
        Some(version) if LEGACY.contains(&version) => Ok(Era::Legacy),
        Some(version) => {
            let mut failure = Failure::new(
                StatusCode::BAD_REQUEST,
                UNSUPPORTED_VERSION,
                "Unsupported protocol version",
            );
            failure.data = Some(json!({ "supported": supported(), "requested": version }));
            Err(failure)
        }
    }
}

fn supported() -> Vec<&'static str> {
    std::iter::once(MODERN)
        .chain(LEGACY.iter().copied())
        .collect()
}

/// The headers must match the body, and the body must carry its metadata.
fn check_modern(headers: &HeaderMap, request: &Request) -> Result<(), Failure> {
    let params = request.params.as_ref();
    let meta = params.and_then(|params| params.get("_meta"));
    let field =
        |name: &str| meta.and_then(|meta| meta.get(format!("io.modelcontextprotocol/{name}")));
    let (Some(version), Some(_)) = (field("protocolVersion"), field("clientCapabilities")) else {
        return Err(Failure::new(
            StatusCode::BAD_REQUEST,
            error_code::INVALID_PARAMS,
            "_meta needs io.modelcontextprotocol/protocolVersion and clientCapabilities",
        ));
    };
    if version.as_str() != Some(MODERN) {
        return Err(Failure::mismatch(
            "MCP-Protocol-Version does not match the protocol version in _meta",
        ));
    }
    if header_value(headers, "mcp-method") != Some(request.method.as_str()) {
        return Err(Failure::mismatch(
            "Mcp-Method is missing or does not match the method",
        ));
    }
    if request.method == "tools/call" {
        let name = params
            .and_then(|params| params.get("name"))
            .and_then(Value::as_str);
        let header = header_value(headers, "mcp-name").and_then(decode_header);
        if name.is_none() || header.as_deref() != name {
            return Err(Failure::mismatch(
                "Mcp-Name is missing or does not match the tool name",
            ));
        }
    }
    Ok(())
}

/// Undoes the `=?base64?...?=` form clients use for values that are not
/// plain ASCII. `None` when that form does not decode.
fn decode_header(value: &str) -> Option<String> {
    match value
        .strip_prefix("=?base64?")
        .and_then(|rest| rest.strip_suffix("?="))
    {
        Some(encoded) => BASE64
            .decode(encoded)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok()),
        None => Some(value.to_owned()),
    }
}

fn answer(daemon: &Daemon, bot: &BotId, era: Era, request: &Request) -> Result<Value, Failure> {
    let params = request.params.clone().unwrap_or_else(|| json!({}));
    let server_info = json!({ "name": "botloft", "version": env!("CARGO_PKG_VERSION") });
    let mut result = match (era, request.method.as_str()) {
        (_, "ping") => json!({}),
        (Era::Modern, "server/discover") => json!({
            "supportedVersions": supported(),
            "capabilities": { "tools": {} },
            "instructions": INSTRUCTIONS,
        }),
        (Era::Legacy, "initialize") => {
            let requested = params.get("protocolVersion").and_then(Value::as_str);
            let version = requested
                .filter(|version| LEGACY.contains(version))
                .unwrap_or(LEGACY[0]);
            json!({
                "protocolVersion": version,
                "capabilities": { "tools": {} },
                "serverInfo": server_info,
                "instructions": INSTRUCTIONS,
            })
        }
        (_, "tools/list") => json!({ "tools": catalog::tools() }),
        (_, "tools/call") => calls::call(daemon, bot, &params)?,
        (era, method) => {
            // 2026-07-28 answers an unknown method with 404, which tells a
            // client this is a modern server that lacks the method.
            let status = match era {
                Era::Modern => StatusCode::NOT_FOUND,
                Era::Legacy => StatusCode::OK,
            };
            return Err(Failure::new(
                status,
                error_code::METHOD_NOT_FOUND,
                format!("Method not found: {method}"),
            ));
        }
    };
    if era == Era::Modern {
        result["resultType"] = json!("complete");
        result["_meta"] = json!({ "io.modelcontextprotocol/serverInfo": server_info });
        // 2026-07-28 requires caching hints on these; clients reject the
        // result without them. The tools only change with the daemon.
        if matches!(request.method.as_str(), "server/discover" | "tools/list") {
            result["ttlMs"] = json!(CACHE_TTL_MS);
            result["cacheScope"] = json!("public");
        }
    }
    Ok(result)
}
