//! What the account's answer and Claude Code's login file say (spec 8.13).

use botloft_core::protocol::UsageWindow;
use serde_json::Value;

use super::Missed;

/// The login in `.credentials.json`, unless it has expired.
pub(super) fn token(text: &str, now: i64) -> Result<String, Missed> {
    let file: Value = serde_json::from_str(text).map_err(|_| Missed::NoLogin)?;
    let oauth = &file["claudeAiOauth"];
    let token = oauth["accessToken"]
        .as_str()
        .filter(|token| !token.is_empty())
        .ok_or(Missed::NoLogin)?;
    if oauth["expiresAt"]
        .as_i64()
        .is_some_and(|expires| expires <= now)
    {
        return Err(Missed::Expired);
    }
    Ok(token.to_owned())
}

/// The windows in the account's answer; `None` when it has none it knows.
pub(super) fn windows(body: &Value) -> Option<Vec<UsageWindow>> {
    let found: Vec<UsageWindow> = match body["limits"].as_array() {
        Some(limits) => limits.iter().filter_map(limit).collect(),
        None => [
            "five_hour",
            "seven_day",
            "seven_day_opus",
            "seven_day_sonnet",
        ]
        .into_iter()
        .filter_map(|name| {
            let window = &body[name];
            Some(UsageWindow {
                name: name.to_owned(),
                utilization: (window["utilization"].as_f64()? / 100.0).clamp(0.0, 1.0),
                resets_at: time(&window["resets_at"]),
                model: None,
            })
        })
        .collect(),
    };
    (!found.is_empty()).then_some(found)
}

/// One of `limits`: the week of every model, a model's own week, or a kind
/// Botloft does not know, by its name.
fn limit(limit: &Value) -> Option<UsageWindow> {
    let kind = limit["kind"].as_str()?;
    let name = match kind {
        "weekly_all" | "weekly_scoped" => "seven_day",
        other => other,
    };
    Some(UsageWindow {
        name: name.to_owned(),
        utilization: (limit["percent"].as_f64()? / 100.0).clamp(0.0, 1.0),
        resets_at: time(&limit["resets_at"]),
        model: limit["scope"]["model"]["display_name"]
            .as_str()
            .map(str::to_owned),
    })
}

/// An ISO 8601 time as Unix ms.
fn time(value: &Value) -> Option<i64> {
    value
        .as_str()?
        .parse::<jiff::Timestamp>()
        .ok()
        .map(|at| at.as_millisecond())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn the_limits_become_windows_with_the_model_of_a_scoped_week() {
        // The shape the account answered on 2026-11-14 (spec 19).
        let body = json!({
            "five_hour": null,
            "seven_day": { "utilization": 80.0, "resets_at": "2026-11-15T13:00:00.170135+00:00" },
            "limits": [
                { "kind": "weekly_all", "group": "weekly", "percent": 80,
                  "resets_at": "2026-11-15T13:00:00.170135+00:00", "scope": null },
                { "kind": "weekly_scoped", "group": "weekly", "percent": 30,
                  "resets_at": "2026-11-15T13:00:00.170259+00:00",
                  "scope": { "model": { "id": null, "display_name": "Fable" }, "surface": null } },
            ],
        });
        let found = windows(&body).expect("windows");
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].name, "seven_day");
        assert_eq!(found[0].model, None);
        assert!((found[0].utilization - 0.8).abs() < 1e-9);
        assert_eq!(found[0].resets_at, Some(1_794_747_600_170));
        assert_eq!(found[1].name, "seven_day");
        assert_eq!(found[1].model.as_deref(), Some("Fable"));
        assert!((found[1].utilization - 0.3).abs() < 1e-9);
    }

    #[test]
    fn without_limits_the_named_windows_count() {
        let body = json!({
            "five_hour": { "utilization": 12.5, "resets_at": null },
            "seven_day": { "utilization": 40.0, "resets_at": "2026-11-15T13:00:00Z" },
            "seven_day_opus": null,
        });
        let found = windows(&body).expect("windows");
        let names: Vec<_> = found.iter().map(|window| window.name.as_str()).collect();
        assert_eq!(names, ["five_hour", "seven_day"]);
        assert!((found[0].utilization - 0.125).abs() < 1e-9);
        assert!(windows(&json!({ "seven_day": null })).is_none());
    }

    #[test]
    fn an_expired_or_missing_login_is_not_used() {
        let file = |expires: i64| {
            json!({ "claudeAiOauth": { "accessToken": "secret", "expiresAt": expires } })
                .to_string()
        };
        assert_eq!(token(&file(2_000), 1_000).ok().as_deref(), Some("secret"));
        assert!(matches!(token(&file(1_000), 1_000), Err(Missed::Expired)));
        assert!(matches!(token("{}", 1_000), Err(Missed::NoLogin)));
        assert!(matches!(token("not json", 1_000), Err(Missed::NoLogin)));
    }
}
