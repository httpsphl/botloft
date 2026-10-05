//! How much of the owner's weekly Claude plan each bot uses (spec 8.7).
//! Claude Code says what each turn cost at API list prices; the plan says
//! how full its seven-day window is, rounded to a point. The rise of the
//! window over what the bots spent meanwhile gives the plan's share per
//! dollar, and each bot's share is its cost times that. The cost is never
//! shown as money: the owner pays for the plan, not the API.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use botloft_core::ids::BotId;
use botloft_core::protocol::UsageWindow;
use botloft_store::{PlanReading, Store};
use serde_json::Value;
use tracing::warn;

use crate::state::Daemon;

/// The window the share is of.
pub const WEEK: &str = "seven_day";
/// How far back the plan's rises are learned from.
const LEARN_MS: i64 = 35 * 24 * 60 * 60 * 1000;
/// Rises smaller than this, in all, say too little: the plan reads in
/// whole points.
const LEARN_MIN_RISE: f64 = 0.02;
/// Readings of one window share its reset time; a little drift is the same.
const SAME_RESET_MS: i64 = 60 * 60 * 1000;

/// What each bot's running process had cost by its last turn: Claude
/// Code's `total_cost_usd` adds up the whole process.
#[derive(Default)]
pub struct CostMeter {
    last: Mutex<HashMap<BotId, (u64, f64)>>,
}

impl CostMeter {
    /// The cost of the turn that just ended, from the process total.
    fn turn(&self, bot: &BotId, generation: u64, total: f64) -> f64 {
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        let before = match last.get(bot) {
            Some((seen, before)) if *seen == generation && *before <= total => *before,
            _ => 0.0,
        };
        last.insert(bot.clone(), (generation, total));
        total - before
    }
}

/// A turn ended: what it cost goes to `turn_costs`.
pub fn turn_ended(daemon: &Daemon, bot: &BotId, generation: u64, result: &Value) {
    let Some(total) = result["total_cost_usd"].as_f64() else {
        return;
    };
    let cost = daemon.costs.turn(bot, generation, total);
    if cost <= 0.0 {
        return;
    }
    let now = daemon.clock.now_ms();
    if let Err(err) = daemon.store().add_turn_cost(bot, now, cost) {
        warn!(bot = %bot, "could not save what a turn cost: {err}");
    }
}

/// The plan's windows as Claude Code reported them.
pub fn observed(daemon: &Daemon, windows: &[UsageWindow], at: i64) {
    let store = daemon.store();
    for window in windows {
        let reading = PlanReading {
            at,
            utilization: window.utilization,
            resets_at: window.resets_at,
        };
        if let Err(err) = store.add_plan_reading(&window.name, &reading) {
            warn!("could not save the plan's usage: {err}");
        }
    }
}

/// The weekly plan's share per dollar of the bots' cost, once the plan
/// has risen enough while they worked. A rise with nothing spent between
/// two readings was someone else's use and is left out.
pub fn rate(store: &Store, now: i64) -> Option<f64> {
    let readings = store.plan_readings(WEEK, now - LEARN_MS).ok()?;
    let (mut rise, mut spent) = (0.0, 0.0);
    for pair in readings.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        let same = match (a.resets_at, b.resets_at) {
            (Some(x), Some(y)) => (x - y).abs() <= SAME_RESET_MS,
            _ => false,
        };
        if !same || b.utilization < a.utilization {
            continue;
        }
        let cost = store.costs_between(a.at, b.at).ok()?;
        if cost > 0.0 {
            rise += b.utilization - a.utilization;
            spent += cost;
        }
    }
    (rise >= LEARN_MIN_RISE && spent > 0.0).then(|| rise / spent)
}

/// Each bot's share of the weekly plan from `since` on, 0 to 1, or `None`
/// before the rate is known.
pub fn shares(store: &Store, since: i64, now: i64) -> Option<HashMap<BotId, f64>> {
    let rate = rate(store, now)?;
    let costs = store.bot_costs(since).ok()?;
    Some(
        costs
            .into_iter()
            .map(|(bot, cost)| (bot, cost * rate))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_turn_costs_what_the_process_total_grew_by() {
        let meter = CostMeter::default();
        let bot = BotId::generate();
        assert!((meter.turn(&bot, 1, 0.5) - 0.5).abs() < 1e-9);
        assert!((meter.turn(&bot, 1, 0.75) - 0.25).abs() < 1e-9);
        // A new process starts over.
        assert!((meter.turn(&bot, 2, 0.1) - 0.1).abs() < 1e-9);
    }
}
