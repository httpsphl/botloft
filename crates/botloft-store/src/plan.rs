//! `turn_costs` and `plan_readings` (spec 8.7): what each bot's turns cost
//! at API list prices, and how the owner's plan usage moved. The share of
//! the plan each bot uses is learned from the two.

use std::collections::HashMap;

use botloft_core::ids::BotId;
use rusqlite::{OptionalExtension, params};

use crate::{Result, Store, parse_column};

/// How long plan readings are kept.
const READINGS_KEPT_MS: i64 = 60 * 24 * 60 * 60 * 1000;

/// A usage window of the plan at one moment.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanReading {
    pub at: i64,
    /// Share used, 0 to 1.
    pub utilization: f64,
    pub resets_at: Option<i64>,
}

impl Store {
    /// What one turn of `bot` cost at `at`.
    pub fn add_turn_cost(&self, bot: &BotId, at: i64, cost: f64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO turn_costs (bot_id, at, cost) VALUES (?1, ?2, ?3)",
            params![bot.as_str(), at, cost],
        )?;
        Ok(())
    }

    /// Each bot's cost from `since` on, archived bots included.
    pub fn bot_costs(&self, since: i64) -> Result<HashMap<BotId, f64>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT bot_id, SUM(cost) FROM turn_costs WHERE at >= ?1 GROUP BY bot_id",
        )?;
        let rows = stmt.query_map([since], |row| Ok((parse_column(row, 0)?, row.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Every bot's cost after `from` and up to `to`.
    pub fn costs_between(&self, from: i64, to: i64) -> Result<f64> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(SUM(cost), 0) FROM turn_costs WHERE at > ?1 AND at <= ?2",
            [from, to],
            |row| row.get(0),
        )?)
    }

    /// Records the window `name` at `at`, unless it reads as it did last
    /// time. Readings older than two months go.
    pub fn add_plan_reading(&self, name: &str, reading: &PlanReading) -> Result<()> {
        let last: Option<(f64, Option<i64>)> = self
            .conn
            .query_row(
                "SELECT utilization, resets_at FROM plan_readings WHERE name = ?1 \
                 ORDER BY at DESC, rowid DESC LIMIT 1",
                [name],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if last == Some((reading.utilization, reading.resets_at)) {
            return Ok(());
        }
        self.conn.execute(
            "INSERT INTO plan_readings (name, at, utilization, resets_at) VALUES (?1, ?2, ?3, ?4)",
            params![name, reading.at, reading.utilization, reading.resets_at],
        )?;
        self.conn.execute(
            "DELETE FROM plan_readings WHERE at < ?1",
            [reading.at - READINGS_KEPT_MS],
        )?;
        Ok(())
    }

    /// The readings of window `name` from `since` on, oldest first.
    pub fn plan_readings(&self, name: &str, since: i64) -> Result<Vec<PlanReading>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT at, utilization, resets_at FROM plan_readings \
             WHERE name = ?1 AND at >= ?2 ORDER BY at, rowid",
        )?;
        let rows = stmt.query_map(params![name, since], |row| {
            Ok(PlanReading {
                at: row.get(0)?,
                utilization: row.get(1)?,
                resets_at: row.get(2)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::PlanReading;
    use crate::tests::Fixture;

    #[test]
    fn costs_add_up_per_bot_and_between_times() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        fx.store.add_turn_cost(scout, 10, 0.5).expect("cost");
        fx.store.add_turn_cost(scout, 20, 0.25).expect("cost");
        fx.store.add_turn_cost(writer, 30, 1.0).expect("cost");
        let costs = fx.store.bot_costs(15).expect("costs");
        assert_eq!(costs.get(scout), Some(&0.25));
        assert_eq!(costs.get(writer), Some(&1.0));
        assert!((fx.store.costs_between(10, 30).expect("between") - 1.25).abs() < 1e-9);
        assert_eq!(fx.store.costs_between(30, 40).expect("none"), 0.0);
    }

    #[test]
    fn a_reading_is_kept_only_when_it_changes() {
        let fx = Fixture::new();
        let read = |at, utilization| PlanReading {
            at,
            utilization,
            resets_at: Some(1_000),
        };
        for (at, utilization) in [(1, 0.30), (2, 0.30), (3, 0.31), (4, 0.31)] {
            fx.store
                .add_plan_reading("seven_day", &read(at, utilization))
                .expect("reading");
        }
        fx.store
            .add_plan_reading("five_hour", &read(5, 0.5))
            .expect("other window");
        let kept = fx.store.plan_readings("seven_day", 0).expect("readings");
        assert_eq!(kept, [read(1, 0.30), read(3, 0.31)]);
    }
}
