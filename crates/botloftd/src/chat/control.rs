//! What the daemon asks a bot's process about its session, on stdin and
//! outside the conversation (spec 9.2): what it runs with and how full it
//! is. Claude Code answers each request with a `control_response` on stdout,
//! during a turn too; none of it reaches the model.

use std::sync::atomic::{AtomicU64, Ordering};

use botloft_core::ids::BotId;
use bytes::Bytes;
use serde_json::{Value, json};
use tracing::debug;

use crate::state::Daemon;
use crate::{context, service};

/// Request ids start with what was asked, so the answer says what it is.
const SETTINGS: &str = "botloft-settings-";
const CONTEXT: &str = "botloft-context-";

static NEXT: AtomicU64 = AtomicU64::new(1);

fn request(prefix: &str, subtype: &str) -> Bytes {
    let id = format!("{prefix}{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let line = json!({
        "type": "control_request",
        "request_id": id,
        "request": { "subtype": subtype },
    });
    Bytes::from(format!("{line}\n"))
}

/// Asks which model and effort the session applies (spec 7.4).
pub(crate) fn settings_request() -> Bytes {
    request(SETTINGS, "get_settings")
}

/// Asks how full the conversation is (spec 8.6).
pub(crate) fn context_request() -> Bytes {
    request(CONTEXT, "get_context_usage")
}

/// A `control_response` from the bot's process. A request this Claude Code
/// does not know fails, and what it would have told stays unknown.
pub(super) fn answered(daemon: &Daemon, bot: &BotId, event: &Value) {
    let response = &event["response"];
    let id = response["request_id"].as_str().unwrap_or_default();
    if response["subtype"] != "success" {
        debug!(bot = %bot, id, "control request failed");
        return;
    }
    let body = &response["response"];
    if id.starts_with(SETTINGS) {
        service::models::applied(daemon, bot, &body["applied"]);
    } else if id.starts_with(CONTEXT) {
        context::answered(daemon, bot, body);
    }
}
