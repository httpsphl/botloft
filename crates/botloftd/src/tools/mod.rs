//! The MCP server the bots use (spec 10): `POST /mcp`, authenticated with
//! the bot token of a running generation.
//!
//! It speaks MCP 2026-07-28, where every request is stateless and carries
//! its own version, and the earlier revisions that open with `initialize`.
//! Claude Code tries `server/discover` first and falls back to
//! `initialize` when that fails, so both paths lead to the same tools.

mod browser;
mod browser_args;
mod browser_catalog;
mod browser_help;
mod browser_reply;
mod browser_sites;
mod calls;
mod catalog;
mod desktop;
mod desktop_catalog;
mod desktop_list;
mod era;
mod question;
mod routine;
mod routine_catalog;
mod routine_change;
mod share;
mod signal;
mod suggest;

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use botloft_core::ids::BotId;
use botloft_core::protocol::error_code;
use serde_json::{Value, json};
use tracing::debug;

use self::era::{Era, LEGACY, era, supported};
pub(crate) use self::routine::check_changed as check_changed_routine;
pub(crate) use self::routine_change::check_changed as check_changed_routine_change;
use crate::approvals;
use crate::rpc::jsonrpc::{self, Request};
use crate::state::Daemon;

/// How long clients may reuse `server/discover` and `tools/list`.
const CACHE_TTL_MS: u64 = 60 * 60 * 1000;

const INSTRUCTIONS: &str = "Tools to work with your Botloft crew: see who is in it, send notes \
    or tasks to other bots, report the result of tasks assigned to you and, for the crew's chief, \
    suggest new bots. schedule_routine asks the owner for work at set times. ask_owner asks the owner a question without waiting; the answer arrives later as a message. send_signal tells the crew something happened, and the routines waiting for it run. share_file shows the \
    owner files you made, as cards in the chat they can open and save. The browser_ tools drive your own web browser, which the owner can watch \
    live and take over when you ask with browser_ask_owner. The desktop_ tools read the apps open on \
    the owner's own computer, app by app as the owner allows. The owner writes to you directly; messages from other bots and from Botloft start with \
    [botloft].";

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
    let Some((bot, generation)) = token.and_then(|token| daemon.supervisor.token_owner(token))
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
    let answered = match era(&headers, &request) {
        // They hold the request until the owner answers (spec 10.1, 10.2, 20.12).
        Ok(era) if called(&request) == Some(catalog::PERMISSION_PROMPT) => {
            permission(&daemon, &bot, generation, &request)
                .await
                .map(|result| decorate(era, &request, result))
        }
        Ok(era) if called(&request) == Some(catalog::SUGGEST_BOT) => {
            let result = suggest::suggest(&daemon, &bot, generation, arguments(&request)).await;
            Ok(decorate(era, &request, result))
        }
        Ok(era) if called(&request) == Some(catalog::SCHEDULE_ROUTINE) => {
            let result = routine::schedule(&daemon, &bot, generation, arguments(&request)).await;
            Ok(decorate(era, &request, result))
        }
        Ok(era) if called(&request) == Some(routine_change::CHANGE_ROUTINE) => {
            let args = arguments(&request);
            let result = routine_change::change(&daemon, &bot, generation, args).await;
            Ok(decorate(era, &request, result))
        }
        Ok(era) if called(&request) == Some(routine_change::DELETE_ROUTINE) => {
            let args = arguments(&request);
            let result = routine_change::delete(&daemon, &bot, generation, args).await;
            Ok(decorate(era, &request, result))
        }
        // They act in a browser and may wait for the owner (spec 21.5).
        Ok(era) if called(&request).is_some_and(|name| name.starts_with(browser_args::PREFIX)) => {
            let name = called(&request).unwrap_or_default().to_owned();
            let result = browser::call(&daemon, &bot, generation, &name, arguments(&request)).await;
            Ok(decorate(era, &request, result))
        }
        // They read the owner's desktop and may wait for the owner (spec 24.2).
        Ok(era) if called(&request).is_some_and(|name| name.starts_with(desktop::PREFIX)) => {
            let name = called(&request).unwrap_or_default().to_owned();
            let result = desktop::call(&daemon, &bot, generation, &name, arguments(&request)).await;
            Ok(decorate(era, &request, result))
        }
        Ok(era) => answer(&daemon, &bot, era, &request),
        Err(failure) => Err(failure),
    };
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

/// The tool a `tools/call` names.
fn called(request: &Request) -> Option<&str> {
    if request.method != "tools/call" {
        return None;
    }
    request
        .params
        .as_ref()
        .and_then(|params| params.get("name"))
        .and_then(Value::as_str)
}

fn arguments(request: &Request) -> Value {
    request
        .params
        .as_ref()
        .and_then(|params| params.get("arguments"))
        .cloned()
        .unwrap_or_else(|| json!({}))
}

async fn permission(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    request: &Request,
) -> Result<Value, Failure> {
    let args: approvals::PromptArgs =
        serde_json::from_value(arguments(request)).map_err(|err| {
            Failure::new(
                StatusCode::OK,
                error_code::INVALID_PARAMS,
                format!("invalid arguments: {err}"),
            )
        })?;
    let decision = approvals::prompt(daemon, bot, generation, args).await;
    Ok(json!({ "content": [{ "type": "text", "text": decision }], "isError": false }))
}

fn server_info() -> Value {
    json!({ "name": "botloft", "version": env!("CARGO_PKG_VERSION") })
}

fn answer(daemon: &Daemon, bot: &BotId, era: Era, request: &Request) -> Result<Value, Failure> {
    let params = request.params.clone().unwrap_or_else(|| json!({}));
    let result = match (era, request.method.as_str()) {
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
                "serverInfo": server_info(),
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
    Ok(decorate(era, request, result))
}

/// What 2026-07-28 adds to every result.
fn decorate(era: Era, request: &Request, mut result: Value) -> Value {
    if era == Era::Modern {
        result["resultType"] = json!("complete");
        result["_meta"] = json!({ "io.modelcontextprotocol/serverInfo": server_info() });
        // 2026-07-28 requires caching hints on these; clients reject the
        // result without them. The tools only change with the daemon.
        if matches!(request.method.as_str(), "server/discover" | "tools/list") {
            result["ttlMs"] = json!(CACHE_TTL_MS);
            result["cacheScope"] = json!("public");
        }
    }
    result
}
