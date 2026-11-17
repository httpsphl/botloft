//! What the daemon asks a bot's process about its session, on stdin and
//! outside the conversation (spec 9.2): what it runs with and how full it
//! is, and to stop what it does. Claude Code answers each request with a
//! `control_response` on stdout, during a turn too; none of it reaches the
//! model.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::seat::Seat;
use bytes::Bytes;
use serde_json::{Value, json};
use tracing::debug;

use crate::state::Daemon;
use crate::{context, service};

/// Request ids start with what was asked, so the answer says what it is.
const SETTINGS: &str = "botloft-settings-";
const CONTEXT: &str = "botloft-context-";
const MCP: &str = "botloft-mcp-";
const STOP: &str = "botloft-stop-";
const INTERRUPT: &str = "botloft-interrupt-";

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

/// Asks whether each connected tool of the bot's process is reachable
/// (spec 25.5).
pub(crate) fn mcp_request() -> Bytes {
    request(MCP, "mcp_status")
}

/// Asks the process to stop what it is doing (spec 9.2): the turn that runs,
/// with its tools and the subagents it left in the background. Messages it
/// has not taken up yet stay in its queue and run next.
pub(crate) fn interrupt_request() -> Bytes {
    request(INTERRUPT, "interrupt")
}

/// Asks the process to end one of its background tasks (spec 33.4). It
/// answers with a new task list; the answer itself says nothing more.
pub(crate) fn stop_task_request(task_id: &str) -> Bytes {
    let id = format!("{STOP}{}", NEXT.fetch_add(1, Ordering::Relaxed));
    let line = json!({
        "type": "control_request",
        "request_id": id,
        "request": { "subtype": "stop_task", "task_id": task_id },
    });
    Bytes::from(format!("{line}\n"))
}

/// A `control_response` from the bot's process. A request this Claude Code
/// does not know fails, and what it would have told stays unknown.
pub(super) fn answered(daemon: &Daemon, bot: &Seat, event: &Value) {
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
    } else if id.starts_with(MCP) {
        service::mcp_state::reported(daemon, bot, &body["mcpServers"]);
    }
}
