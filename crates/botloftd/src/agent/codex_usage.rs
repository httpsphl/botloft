//! The Codex account's usage (spec 30.3.2): what `account/rateLimits/read`
//! answers, as the app's usage dialog shows it for Claude Code.

use botloft_core::protocol::{AccountUsage, AgentKind, AgentUsage, UsageWindow};
use serde_json::Value;

/// From this share used on, the usage is a warning.
const WARN_FROM: i64 = 80;

/// The `rateLimits` of a `account/rateLimits/read` answer. `None` when no
/// window is there (an account with no limit to show).
pub(super) fn parse(limits: &Value, now_ms: i64) -> Option<AgentUsage> {
    let windows: Vec<UsageWindow> = ["primary", "secondary"]
        .into_iter()
        .filter_map(|key| window(&limits[key]))
        .collect();
    if windows.is_empty() {
        return None;
    }
    let reached = !limits["rateLimitReachedType"].is_null();
    let most = ["primary", "secondary"]
        .into_iter()
        .filter_map(|key| limits[key]["usedPercent"].as_i64())
        .max()
        .unwrap_or(0);
    let status = if reached || most >= 100 {
        "rejected"
    } else if most >= WARN_FROM {
        "allowed_warning"
    } else {
        "allowed"
    };
    // The window that is full is the one that decides when it is back.
    let resets_at = windows
        .iter()
        .filter(|window| window.utilization >= 1.0)
        .filter_map(|window| window.resets_at)
        .max()
        .or_else(|| windows.iter().filter_map(|window| window.resets_at).min());
    Some(AgentUsage {
        agent: AgentKind::Codex,
        plan: limits["planType"].as_str().map(str::to_owned),
        usage: AccountUsage {
            status: status.to_owned(),
            resets_at,
            windows,
            observed_at: now_ms,
        },
    })
}

fn window(value: &Value) -> Option<UsageWindow> {
    let used = value["usedPercent"].as_i64()?;
    let minutes = value["windowDurationMins"].as_i64();
    Some(UsageWindow {
        name: window_name(minutes),
        utilization: (used as f64 / 100.0).clamp(0.0, 1.0),
        // Seconds in Codex, milliseconds here.
        resets_at: value["resetsAt"].as_i64().map(|seconds| seconds * 1000),
    })
}

/// The names the app knows for the windows of Claude Code, so the dialog
/// reads the same; any other length is named by its minutes.
fn window_name(minutes: Option<i64>) -> String {
    match minutes {
        Some(300) => "five_hour".to_owned(),
        Some(10_080) => "seven_day".to_owned(),
        Some(43_200) => "thirty_day".to_owned(),
        Some(minutes) => format!("{minutes}_minutes"),
        None => "window".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_full_free_plan_is_a_limit_with_the_day_it_is_back() {
        // What the real Codex 0.148 answered on 2026-10-11.
        let limits = json!({
            "limitId": "codex",
            "primary": { "usedPercent": 100, "windowDurationMins": 43200, "resetsAt": 1794165867 },
            "secondary": null,
            "planType": "free",
            "rateLimitReachedType": "rate_limit_reached",
        });
        let usage = parse(&limits, 5).expect("usage");
        assert_eq!(usage.agent, AgentKind::Codex);
        assert_eq!(usage.plan.as_deref(), Some("free"));
        assert_eq!(usage.usage.status, "rejected");
        assert_eq!(usage.usage.resets_at, Some(1_794_165_867_000));
        assert_eq!(usage.usage.observed_at, 5);
        assert_eq!(usage.usage.windows.len(), 1);
        assert_eq!(usage.usage.windows[0].name, "thirty_day");
        assert!((usage.usage.windows[0].utilization - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn two_windows_keep_their_names_and_warn_from_eighty() {
        let limits = json!({
            "primary": { "usedPercent": 34, "windowDurationMins": 300, "resetsAt": 100 },
            "secondary": { "usedPercent": 82, "windowDurationMins": 10080, "resetsAt": 200 },
            "planType": "plus",
        });
        let usage = parse(&limits, 0).expect("usage").usage;
        assert_eq!(usage.status, "allowed_warning");
        let names: Vec<_> = usage.windows.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, ["five_hour", "seven_day"]);
        // Nothing is full: the nearest reset is the one to say.
        assert_eq!(usage.resets_at, Some(100_000));
    }

    #[test]
    fn an_account_without_a_window_shows_nothing() {
        assert!(parse(&json!({ "primary": null, "secondary": null }), 0).is_none());
        assert!(parse(&json!({}), 0).is_none());
    }

    #[test]
    fn an_odd_window_is_named_by_its_minutes() {
        let limits = json!({ "primary": { "usedPercent": 1, "windowDurationMins": 90 } });
        let usage = parse(&limits, 0).expect("usage").usage;
        assert_eq!(usage.windows[0].name, "90_minutes");
        assert_eq!(usage.windows[0].resets_at, None);
    }
}
