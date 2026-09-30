//! What the stream says about the conversation on Claude Code's side (spec
//! 7.3): when it is on disk and can be resumed, and when the one a process
//! was told to resume is not there.

use botloft_core::ids::BotId;
use serde_json::Value;
use tracing::warn;

use crate::state::Daemon;

/// How the error of a missing conversation begins (seen with 2.1.284,
/// spec 19); the session id follows.
const NO_CONVERSATION: &str = "No conversation found";

/// A turn's message came back, with the session id like every event. Claude
/// Code has the conversation on disk from here on, not from `system/init`,
/// so this is what the next start resumes.
pub(super) fn began(daemon: &Daemon, bot: &BotId, event: &Value) {
    let Some(session) = event["session_id"].as_str() else {
        return;
    };
    // Only the bot's own turns: a subagent works inside one of them.
    if !event["parent_tool_use_id"].is_null() {
        return;
    }
    let store = daemon.store();
    let known = store.session_id(bot).ok().flatten();
    if known.as_deref() != Some(session)
        && let Err(err) = store.set_session_id(bot, Some(session))
    {
        warn!(bot = %bot, "could not save the session id: {err}");
    }
    drop(store);
    daemon.supervisor.remember_session(bot, session);
}

/// Whether this `result` is Claude Code saying it could not find the
/// conversation to resume. It comes with no turn before it, and the process
/// exits right after.
pub(super) fn missing(event: &Value) -> bool {
    event["subtype"] == "error_during_execution"
        && event["errors"].as_array().is_some_and(|errors| {
            errors
                .iter()
                .filter_map(Value::as_str)
                .any(|error| error.starts_with(NO_CONVERSATION))
        })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn failed(subtype: &str, error: &str) -> Value {
        json!({ "type": "result", "subtype": subtype, "is_error": true, "errors": [error] })
    }

    #[test]
    fn only_a_conversation_that_was_not_found_counts_as_missing() {
        let not_found = "No conversation found with session ID: 7de9b8c8";
        assert!(missing(&failed("error_during_execution", not_found)));
        assert!(!missing(&failed("error_during_execution", "other")));
        assert!(!missing(&failed("error_max_turns", not_found)));
        let done = json!({ "type": "result", "subtype": "success", "is_error": false });
        assert!(!missing(&done));
    }
}
