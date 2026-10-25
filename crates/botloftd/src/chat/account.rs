//! What the account behind a bot's turns says (spec 7.2 and 8.1): an API
//! error that ended a turn, and the usage of the owner's Claude plan.

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    AccountUsage, ChatBody, NoticeCode, NoticeItem, NoticeLevel, UsageWindow,
};
use serde_json::Value;

use super::{claude, items};
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
/// When a rate limit gives no reset time.
const DEFAULT_LIMIT_MS: i64 = 5 * 60 * 1000;

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
    // A failed sign-in with a key in the environment is the key's fault.
    let key = error == "authentication_failed" && crate::platform::api_key_in_env();
    let (code, text) = notice_for(error, detail, key);
    items::add(
        daemon,
        bot,
        ChatBody::Notice(NoticeItem {
            level: NoticeLevel::Error,
            code: Some(code),
            text,
        }),
    );
    after_notice(daemon, bot, generation, error);
}

/// What the chat says about an API error. The code lets the app say it in
/// the owner's language; the text stays for older apps and for the
/// conversation list. `key_failed`: Claude Code signed in with an
/// `ANTHROPIC_API_KEY` from the environment, and it did not work.
fn notice_for(error: &str, detail: String, key_failed: bool) -> (NoticeCode, String) {
    if key_failed {
        (
            NoticeCode::ApiKeyFailed,
            "Claude Code used the ANTHROPIC_API_KEY saved in this computer's environment instead \
             of the Claude account, and the key did not work. Remove or fix it, then restart the \
             bot."
                .to_owned(),
        )
    } else if SIGN_IN_ERRORS.contains(&error) {
        (
            NoticeCode::SignedOut,
            "Claude Code is not signed in or the account cannot be used. Sign in, then restart \
             the bot."
                .to_owned(),
        )
    } else if error == "rate_limit" {
        (
            NoticeCode::UsageLimit,
            "The account reached its usage limit. Messages wait until it resets.".to_owned(),
        )
    } else if error == MODEL_NOT_FOUND {
        (
            NoticeCode::ModelUnavailable,
            "Claude Code could not use this bot's model: it may not exist or not be on the \
             account's plan. Pick another model."
                .to_owned(),
        )
    } else if detail.is_empty() {
        (NoticeCode::TurnFailed, error.to_owned())
    } else {
        (NoticeCode::TurnFailed, detail)
    }
}

/// Stops or pauses the bot when only the owner or time can fix the error.
fn after_notice(daemon: &Daemon, bot: &BotId, generation: u64, error: &str) {
    if SIGN_IN_ERRORS.contains(&error) {
        daemon.supervisor.signed_out(bot, generation);
    } else if error == "rate_limit" {
        let until = daemon
            .usage()
            .and_then(|usage| usage.resets_at)
            .unwrap_or_else(|| daemon.clock.now_ms() + DEFAULT_LIMIT_MS);
        daemon.supervisor.rate_limited(bot, generation, until);
    }
}

pub(super) fn rate_limit(daemon: &Daemon, bot: &BotId, generation: u64, event: &Value) {
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
    let limited = !status.starts_with("allowed");
    let now = daemon.clock.now_ms();
    crate::service::plan::observed(daemon, &windows, now);
    daemon.set_usage(AccountUsage {
        status,
        resets_at,
        windows,
        observed_at: now,
    });
    if limited {
        let until = resets_at.unwrap_or_else(|| daemon.clock.now_ms() + DEFAULT_LIMIT_MS);
        daemon.supervisor.rate_limited(bot, generation, until);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_failed_key_in_the_environment_is_told_apart_from_signing_out() {
        let (code, text) = notice_for("authentication_failed", String::new(), true);
        assert_eq!(code, NoticeCode::ApiKeyFailed);
        assert!(text.contains("ANTHROPIC_API_KEY"), "{text}");
        let (code, _) = notice_for("authentication_failed", String::new(), false);
        assert_eq!(code, NoticeCode::SignedOut);
        let (code, _) = notice_for("billing_error", String::new(), false);
        assert_eq!(code, NoticeCode::SignedOut);
        // No run of spaces left inside a sentence.
        let (_, text) = notice_for(MODEL_NOT_FOUND, String::new(), false);
        assert!(!text.contains("  "), "{text}");
    }
}
