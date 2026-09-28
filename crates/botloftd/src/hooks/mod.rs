//! Claude Code hooks (spec 7.5 and 7.6). Each bot runs `botloftd hook
//! <event>` (see [`client`]), which forwards the hook to `POST /hooks/<event>`
//! with the bot's token; [`handle`] turns it into a supervisor [`Hook`].

pub mod client;

use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::state::Daemon;
use crate::supervisor::{Hook, Inbox};

/// Hook events registered in each bot's settings, with the argument
/// `botloftd hook` receives for each.
pub const EVENTS: &[(&str, &str)] = &[
    ("SessionStart", "session-start"),
    ("UserPromptSubmit", "prompt-submit"),
    ("Stop", "stop"),
    ("StopFailure", "stop-failure"),
    ("Notification", "notification"),
    ("SessionEnd", "session-end"),
];

/// Body of `POST /hooks/<event>`.
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookRequest {
    /// The JSON Claude Code wrote to the hook's stdin.
    #[serde(default)]
    pub payload: Value,
    /// `CLAUDE_CODE_MESSAGING_SOCKET`, sent by `session-start` only.
    #[serde(default)]
    pub messaging_socket: Option<String>,
    /// `CLAUDE_CODE_MESSAGING_TOKEN`, sent by `session-start` only.
    #[serde(default)]
    pub messaging_token: Option<String>,
}

/// Reads the payload fields Claude Code documents for each event.
pub fn parse(event: &str, request: HookRequest) -> Option<Hook> {
    let field = |name: &str| {
        request
            .payload
            .get(name)
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned()
    };
    Some(match event {
        "session-start" => Hook::SessionStart {
            inbox: match (request.messaging_socket, request.messaging_token) {
                (Some(socket), Some(token)) if !socket.is_empty() && !token.is_empty() => {
                    Some(Inbox { socket, token })
                }
                _ => None,
            },
        },
        "prompt-submit" => Hook::PromptSubmit,
        "stop" => Hook::Stop,
        "stop-failure" => Hook::StopFailure {
            error: field("error"),
        },
        "notification" => Hook::Notification {
            kind: field("notification_type"),
        },
        "session-end" => Hook::SessionEnd,
        _ => return None,
    })
}

/// `POST /hooks/{event}`. Only the token of a running generation is valid.
pub async fn handle(
    State(daemon): State<Arc<Daemon>>,
    Path(event): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let Some((bot, generation)) = token.and_then(|token| daemon.supervisor.hook_owner(token))
    else {
        return StatusCode::UNAUTHORIZED;
    };
    let request = if body.is_empty() {
        HookRequest::default()
    } else {
        match serde_json::from_slice(&body) {
            Ok(request) => request,
            Err(_) => return StatusCode::BAD_REQUEST,
        }
    };
    let Some(hook) = parse(&event, request) else {
        return StatusCode::NOT_FOUND;
    };
    daemon.supervisor.on_hook(&bot, generation, hook);
    StatusCode::NO_CONTENT
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn request(payload: Value) -> HookRequest {
        HookRequest {
            payload,
            ..HookRequest::default()
        }
    }

    #[test]
    fn reads_the_documented_payload_fields() {
        let failure = request(json!({ "hook_event_name": "StopFailure", "error": "rate_limit" }));
        assert_eq!(
            parse("stop-failure", failure),
            Some(Hook::StopFailure {
                error: "rate_limit".into()
            })
        );
        let note = request(json!({ "notification_type": "permission_prompt", "message": "x" }));
        assert_eq!(
            parse("notification", note),
            Some(Hook::Notification {
                kind: "permission_prompt".into()
            })
        );
        assert_eq!(
            parse("prompt-submit", request(Value::Null)),
            Some(Hook::PromptSubmit)
        );
        assert_eq!(parse("bogus", request(Value::Null)), None);
    }

    #[test]
    fn session_start_carries_the_inbox_when_both_parts_are_there() {
        let full = HookRequest {
            payload: json!({ "source": "startup" }),
            messaging_socket: Some(r"\\.\pipe\claude-1".into()),
            messaging_token: Some("t".into()),
        };
        let Some(Hook::SessionStart { inbox: Some(inbox) }) = parse("session-start", full) else {
            panic!("expected an inbox");
        };
        assert_eq!(inbox.socket, r"\\.\pipe\claude-1");
        let partial = HookRequest {
            messaging_socket: Some("s".into()),
            ..HookRequest::default()
        };
        assert_eq!(
            parse("session-start", partial),
            Some(Hook::SessionStart { inbox: None })
        );
    }

    #[test]
    fn every_event_has_a_parser() {
        for (_, arg) in EVENTS {
            assert!(parse(arg, HookRequest::default()).is_some(), "{arg}");
        }
    }
}
