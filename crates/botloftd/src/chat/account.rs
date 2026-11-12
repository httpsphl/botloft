//! What the account behind a bot's turns says (spec 7.2 and 8.1): an API
//! error that ended a turn, and the usage of the owner's Claude plan.

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    AccountUsage, ChatBody, NoticeCode, NoticeItem, NoticeLevel, UsageWindow,
};
use serde_json::Value;

use super::{claude, items};
use crate::seat::Seat;
use crate::service::resume;
use crate::state::Daemon;

/// Errors that restarting cannot fix: Claude Code needs the owner.
const SIGN_IN_ERRORS: &[&str] = &[
    "authentication_failed",
    "oauth_org_not_allowed",
    "billing_error",
    "account_on_hold",
];
/// The model does not exist or the account cannot use it (spec 7.4).
const MODEL_NOT_FOUND: &str = "model_not_found";

/// An API error ended the turn: say so, and stop or pause the bot when
/// only the owner or time can fix it (spec 7.2).
pub(super) fn failed_turn(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    error: &str,
    event: &Value,
) {
    let detail = claude::blocks(event)
        .iter()
        .filter_map(|block| block["text"].as_str())
        .collect::<Vec<_>>()
        .join(" ");
    // The code lets the app say it in the owner's language; the text stays
    // for older apps and for the conversation list.
    let (code, text) = if SIGN_IN_ERRORS.contains(&error) {
        (
            NoticeCode::SignedOut,
            "Claude Code is not signed in or the account cannot be used. Sign in, then restart \
             the bot."
                .to_owned(),
        )
    } else if error == "rate_limit" {
        (
            NoticeCode::UsageLimit,
            "The account reached its usage limit. Messages wait until it resets, and then the \
             agent carries on where it stopped."
                .to_owned(),
        )
    } else if error == MODEL_NOT_FOUND {
        (
            NoticeCode::ModelUnavailable,
            "Claude Code could not use this bot's model: it may not exist or not be on the              account's plan. Pick another model."
                .to_owned(),
        )
    } else if detail.is_empty() {
        (NoticeCode::TurnFailed, error.to_owned())
    } else {
        (NoticeCode::TurnFailed, detail)
    };
    items::add(
        daemon,
        bot,
        ChatBody::Notice(NoticeItem {
            level: NoticeLevel::Error,
            code: Some(code),
            text,
        }),
    );
    if SIGN_IN_ERRORS.contains(&error) {
        daemon.supervisor.signed_out(bot, generation);
    } else if error == "rate_limit" {
        let resets_at = daemon.usage().and_then(|usage| usage.resets_at);
        resume::cut_turn(daemon, bot, generation, resets_at);
    }
}

pub(super) fn rate_limit(daemon: &Daemon, bot: &Seat, generation: u64, event: &Value) {
    let info = &event["rate_limit_info"];
    let seconds = |value: &Value| value.as_i64().map(|s| s * 1000);
    let mut windows: Vec<UsageWindow> = info["unifiedWindows"]
        .as_object()
        .map(|windows| {
            windows
                .iter()
                .map(|(name, window)| UsageWindow {
                    name: name.clone(),
                    utilization: window["utilization"].as_f64().unwrap_or_default(),
                    resets_at: seconds(&window["resetsAt"]),
                })
                .collect()
        })
        .unwrap_or_default();
    windows.sort_by(|a, b| a.name.cmp(&b.name));
    let status = info["status"].as_str().unwrap_or("unknown").to_owned();
    let resets_at = seconds(&info["resetsAt"]);
    let extra_usage = using_extra_usage(info);
    // Past a window of the plan, an account with paid extra usage keeps
    // working on it: `rejected` stops the bots only without it (spec 19).
    let limited = status == "rejected" && !extra_usage;
    let now = daemon.clock.now_ms();
    crate::service::plan::observed(daemon, &windows, now);
    daemon.set_usage(AccountUsage {
        status,
        resets_at,
        windows,
        observed_at: now,
        extra_usage,
    });
    if limited {
        let until = resume::until(daemon, resets_at);
        daemon.supervisor.rate_limited(bot, generation, until);
    }
}

/// Claude Code says the turns run on the account's paid extra usage with
/// either of two flags (seen in 2.1.309, spec 19).
fn using_extra_usage(info: &Value) -> bool {
    ["isUsingOverage", "overageInUse"]
        .iter()
        .any(|flag| info[*flag].as_bool() == Some(true))
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::using_extra_usage;

    #[test]
    fn either_flag_means_extra_usage() {
        assert!(using_extra_usage(&json!({ "isUsingOverage": true })));
        assert!(using_extra_usage(&json!({ "overageInUse": true })));
        // Extra usage that is only allowed is not in use yet.
        assert!(!using_extra_usage(
            &json!({ "overageStatus": "allowed", "isUsingOverage": false })
        ));
        assert!(!using_extra_usage(&json!({})));
    }
}
