//! `usage.tokens` and `usage.costs` (spec 8.7, 11.2): what the bots used,
//! in tokens and at API list prices.

use std::collections::BTreeMap;

use botloft_core::protocol::{BotTokens, CostBy, CostGroup, UsageCostsParams, UsageTokensParams};
use jiff::tz::TimeZone;

use super::{ApiError, ApiResult};
use crate::state::Daemon;

/// Each bot's tokens and cost since `since`, archived bots included.
pub fn tokens(daemon: &Daemon, params: UsageTokensParams) -> ApiResult<Vec<BotTokens>> {
    if params.since < 0 {
        return Err(ApiError::validation("since must not be negative"));
    }
    Ok(daemon.store().bot_usage(params.since)?)
}

/// The cost since `since` per model, per day or per routine.
pub fn costs(daemon: &Daemon, params: UsageCostsParams) -> ApiResult<Vec<CostGroup>> {
    if params.since < 0 {
        return Err(ApiError::validation("since must not be negative"));
    }
    let store = daemon.store();
    Ok(match params.by {
        CostBy::Model => store.costs_by_model(params.since)?,
        CostBy::Routine => store.costs_by_routine(params.since)?,
        CostBy::Day => by_day(&store.cost_times(params.since)?, &TimeZone::system()),
    })
}

/// The costs summed per day in `zone`, the newest day first.
fn by_day(costs: &[(i64, f64)], zone: &TimeZone) -> Vec<CostGroup> {
    let mut days: BTreeMap<String, f64> = BTreeMap::new();
    for (at, cost) in costs {
        let Ok(time) = jiff::Timestamp::from_millisecond(*at) else {
            continue;
        };
        let day = time.to_zoned(zone.clone()).date().to_string();
        *days.entry(day).or_default() += cost;
    }
    days.into_iter()
        .rev()
        .map(|(key, cost)| CostGroup {
            key,
            cost,
            name: None,
            bot_name: None,
            color: None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_follow_the_time_zone() {
        // 2026-11-14 01:30 UTC is still the 13th in São Paulo (UTC-3).
        let late = 1_794_619_800_000;
        let costs = [(late, 0.5), (late + 3 * 3_600_000, 0.25)];
        let zone = TimeZone::get("America/Sao_Paulo").expect("zone");
        let days = by_day(&costs, &zone);
        let keys: Vec<_> = days.iter().map(|day| day.key.as_str()).collect();
        assert_eq!(keys, ["2026-11-14", "2026-11-13"]);
        let utc = by_day(&costs, &TimeZone::UTC);
        assert_eq!(utc.len(), 1);
        assert!((utc[0].cost - 0.75).abs() < 1e-9);
    }
}
