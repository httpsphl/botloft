//! Which MCP revision a request follows, and the header checks 2026-07-28
//! requires.

use axum::http::{HeaderMap, StatusCode};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64;
use botloft_core::protocol::error_code;
use serde_json::{Value, json};

use super::Failure;
use crate::rpc::jsonrpc::Request;

/// The revision without `initialize`.
pub(super) const MODERN: &str = "2026-07-28";
/// Revisions that open with `initialize`, newest first.
pub(super) const LEGACY: &[&str] = &["2025-11-25", "2025-06-18", "2025-03-26"];
/// MCP error codes (2026-07-28).
const HEADER_MISMATCH: i32 = -32020;
const UNSUPPORTED_VERSION: i32 = -32022;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Era {
    Modern,
    Legacy,
}

fn mismatch(message: &str) -> Failure {
    Failure::new(StatusCode::BAD_REQUEST, HEADER_MISMATCH, message)
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

/// Which revision the request follows, checking what 2026-07-28 requires.
pub(super) fn era(headers: &HeaderMap, request: &Request) -> Result<Era, Failure> {
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

pub(super) fn supported() -> Vec<&'static str> {
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
        return Err(mismatch(
            "MCP-Protocol-Version does not match the protocol version in _meta",
        ));
    }
    if header_value(headers, "mcp-method") != Some(request.method.as_str()) {
        return Err(mismatch(
            "Mcp-Method is missing or does not match the method",
        ));
    }
    if request.method == "tools/call" {
        let name = params
            .and_then(|params| params.get("name"))
            .and_then(Value::as_str);
        let header = header_value(headers, "mcp-name").and_then(decode_header);
        if name.is_none() || header.as_deref() != name {
            return Err(mismatch(
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
