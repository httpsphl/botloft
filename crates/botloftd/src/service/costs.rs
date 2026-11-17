//! What each turn would cost at API list prices (spec 8.7). Claude Code adds
//! up its whole process in `total_cost_usd` and, per model, in the
//! `costUSD` of `modelUsage`; a turn's cost is how much they grew. The cost
//! is a way to compare the bots: the owner's plan does not charge it.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use botloft_store::TurnCost;
use serde_json::Value;
use tracing::warn;

use crate::seat::Seat;
use crate::state::Daemon;

/// What a process had cost by a turn's end, in all and per model.
#[derive(Debug, Default, Clone, PartialEq)]
struct Totals {
    total: f64,
    models: HashMap<String, f64>,
}

impl Totals {
    /// The totals a `result` reports; `None` when it says no cost.
    fn of(result: &Value) -> Option<Self> {
        let total = result["total_cost_usd"].as_f64()?;
        let models = result["modelUsage"]
            .as_object()
            .map(|models| {
                models
                    .iter()
                    .filter_map(|(model, usage)| Some((model.clone(), usage["costUSD"].as_f64()?)))
                    .collect()
            })
            .unwrap_or_default();
        Some(Self { total, models })
    }
}

/// What each seat's running process had cost by its last turn. Each
/// conversation of a bot runs a process of its own (spec 40), so the count
/// is per seat: per bot, two conversations taking turns read each other's
/// totals as a new process and counted them again.
#[derive(Default)]
pub struct CostMeter {
    last: Mutex<HashMap<Seat, (u64, Totals)>>,
}

impl CostMeter {
    /// The cost of the turn that just ended, per model (`None` when Claude
    /// Code named none), from the process totals. Parts that cost nothing
    /// are left out.
    fn turn(&self, seat: &Seat, generation: u64, now: Totals) -> Vec<(Option<String>, f64)> {
        let mut last = self.last.lock().unwrap_or_else(PoisonError::into_inner);
        let before = match last.get(seat) {
            Some((seen, before)) if *seen == generation => before.clone(),
            _ => Totals::default(),
        };
        let mut parts: Vec<(Option<String>, f64)> = if now.models.is_empty() {
            vec![(None, grown(before.total, now.total))]
        } else {
            now.models
                .iter()
                .map(|(model, cost)| {
                    let was = before.models.get(model).copied().unwrap_or_default();
                    (Some(model.clone()), grown(was, *cost))
                })
                .collect()
        };
        parts.retain(|(_, cost)| *cost > 0.0);
        parts.sort_by(|a, b| a.0.cmp(&b.0));
        last.insert(seat.clone(), (generation, now));
        parts
    }
}

/// How much a running total grew; a lower one started over and counts whole.
fn grown(before: f64, now: f64) -> f64 {
    if now >= before { now - before } else { now }
}

/// A turn ended (or a compaction, which also calls the model): what it cost
/// goes to `turn_costs`, a row per model, with the routine whose run the
/// turn carried. The sum comes back for the chat's turn line, `None` when
/// Claude Code said no cost or it was nothing.
pub fn turn_ended(daemon: &Daemon, seat: &Seat, generation: u64, result: &Value) -> Option<f64> {
    let totals = Totals::of(result)?;
    let parts = daemon.costs.turn(seat, generation, totals);
    let run = daemon.routines.run_of_turn(seat);
    let at = daemon.clock.now_ms();
    let store = daemon.store();
    for (model, cost) in &parts {
        let part = TurnCost {
            bot: &seat.bot,
            at,
            cost: *cost,
            model: model.as_deref(),
            run: run.as_ref(),
        };
        if let Err(err) = store.add_turn_cost(&part) {
            warn!(bot = %seat.bot, "could not save what a turn cost: {err}");
        }
    }
    let sum: f64 = parts.iter().map(|(_, cost)| cost).sum();
    (sum > 0.0).then_some(sum)
}

#[cfg(test)]
mod tests {
    use botloft_core::ids::{BotId, ConversationId};
    use serde_json::json;

    use super::*;

    fn seat(bot: &BotId, conversation: ConversationId) -> Seat {
        Seat::new(bot.clone(), conversation)
    }

    fn totals(result: Value) -> Totals {
        Totals::of(&result).expect("totals")
    }

    #[test]
    fn a_turn_costs_what_each_model_grew_by() {
        let meter = CostMeter::default();
        let bot = BotId::generate();
        let main = seat(&bot, ConversationId::main_of(&bot));
        let first = totals(json!({
            "total_cost_usd": 0.5,
            "modelUsage": { "claude-opus-5-5": { "costUSD": 0.5 } },
        }));
        assert_eq!(
            meter.turn(&main, 1, first),
            [(Some("claude-opus-5-5".to_owned()), 0.5)]
        );
        // A subagent on Haiku joins; the main model grew by a quarter.
        let second = totals(json!({
            "total_cost_usd": 0.85,
            "modelUsage": {
                "claude-opus-5-5": { "costUSD": 0.75 },
                "claude-haiku-5-5": { "costUSD": 0.1 },
            },
        }));
        let parts = meter.turn(&main, 1, second);
        assert_eq!(parts.len(), 2);
        assert!((parts[0].1 - 0.1).abs() < 1e-9, "haiku first: {parts:?}");
        assert!((parts[1].1 - 0.25).abs() < 1e-9, "opus: {parts:?}");
        // A new process starts over.
        let fresh = totals(json!({ "total_cost_usd": 0.1 }));
        assert_eq!(meter.turn(&main, 2, fresh), [(None, 0.1)]);
    }

    #[test]
    fn two_conversations_of_one_bot_keep_their_own_totals() {
        let meter = CostMeter::default();
        let bot = BotId::generate();
        let main = seat(&bot, ConversationId::main_of(&bot));
        let other = seat(&bot, ConversationId::generate());
        let total = |cost: f64| totals(json!({ "total_cost_usd": cost }));
        assert_eq!(meter.turn(&main, 10, total(1.0)), [(None, 1.0)]);
        assert_eq!(meter.turn(&other, 11, total(0.5)), [(None, 0.5)]);
        // Back in the main conversation, only what it grew by.
        let parts = meter.turn(&main, 10, total(1.25));
        assert_eq!(parts.len(), 1);
        assert!((parts[0].1 - 0.25).abs() < 1e-9);
        // A turn that cost nothing has no parts.
        assert!(meter.turn(&other, 11, total(0.5)).is_empty());
    }
}
