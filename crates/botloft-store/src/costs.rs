//! `turn_costs` (spec 8.7): what each bot's turns would cost at API list
//! prices, a row per model, with the routine whose run the turn carried.

use std::collections::HashMap;

use botloft_core::ids::{BotId, RoutineRunId};
use botloft_core::protocol::CostGroup;
use rusqlite::params;

use crate::{Result, Store, parse_column};

/// One model's part of what a turn cost.
#[derive(Debug, Clone, Copy)]
pub struct TurnCost<'a> {
    pub bot: &'a BotId,
    pub at: i64,
    /// US dollars at API list prices.
    pub cost: f64,
    /// The model's id, `None` when Claude Code named none.
    pub model: Option<&'a str>,
    /// The routine run the turn carried, if any.
    pub run: Option<&'a RoutineRunId>,
}

impl Store {
    /// Keeps one part of what a turn cost; the run is kept as its routine.
    pub fn add_turn_cost(&self, part: &TurnCost<'_>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO turn_costs (bot_id, at, cost, model, routine_id) \
             VALUES (?1, ?2, ?3, ?4, (SELECT routine_id FROM routine_runs WHERE id = ?5))",
            params![
                part.bot.as_str(),
                part.at,
                part.cost,
                part.model,
                part.run.map(RoutineRunId::as_str),
            ],
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

    /// The cost per model from `since` on, the dearest first. The rows kept
    /// before the model was have the key `""`.
    pub fn costs_by_model(&self, since: i64) -> Result<Vec<CostGroup>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT COALESCE(model, ''), SUM(cost) FROM turn_costs WHERE at >= ?1 \
             GROUP BY 1 ORDER BY 2 DESC, 1",
        )?;
        let rows = stmt.query_map([since], |row| {
            Ok(CostGroup {
                key: row.get(0)?,
                cost: row.get(1)?,
                name: None,
                bot_name: None,
                color: None,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The cost per routine from `since` on, the dearest first, with the
    /// routine's name and bot. What no routine carried has the key `""`; a
    /// deleted routine has no name.
    pub fn costs_by_routine(&self, since: i64) -> Result<Vec<CostGroup>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT COALESCE(c.routine_id, ''), SUM(c.cost), r.name, b.name, b.color \
             FROM turn_costs c \
             LEFT JOIN routines r ON r.id = c.routine_id \
             LEFT JOIN bots b ON b.id = r.bot_id \
             WHERE c.at >= ?1 GROUP BY 1 ORDER BY 2 DESC, 1",
        )?;
        let rows = stmt.query_map([since], |row| {
            Ok(CostGroup {
                key: row.get(0)?,
                cost: row.get(1)?,
                name: row.get(2)?,
                bot_name: row.get(3)?,
                color: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Every cost from `since` on with its time, oldest first: the daemon
    /// groups them by day in the computer's time zone.
    pub fn cost_times(&self, since: i64) -> Result<Vec<(i64, f64)>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT at, cost FROM turn_costs WHERE at >= ?1 ORDER BY at, rowid")?;
        let rows = stmt.query_map([since], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::ids::RoutineId;
    use botloft_core::protocol::{Missed, Overlap, Routine, RoutineRun, RunStatus, Schedule};

    use super::*;
    use crate::tests::Fixture;

    /// A routine of the fixture's first bot, with one run.
    fn routine_with_run(fx: &Fixture, name: &str) -> (RoutineId, RoutineRunId) {
        let routine = Routine {
            id: RoutineId::generate(),
            bot_id: fx.bots[0].id.clone(),
            name: name.into(),
            prompt: "Summarize".into(),
            schedule: Schedule::Interval { minutes: 60 },
            timezone: "UTC".into(),
            overlap: Overlap::Skip,
            missed: Missed::RunOnce,
            enabled: true,
            next_run_at: None,
            last_run: None,
            fail_streak: 0,
            created_at: 0,
            updated_at: 0,
            archived_at: None,
        };
        fx.store.insert_routine(&routine).expect("routine");
        let run = RoutineRun {
            id: RoutineRunId::generate(),
            routine_id: routine.id.clone(),
            scheduled_for: 1,
            status: RunStatus::Queued,
            reason: None,
            skipped_count: 0,
            message_id: None,
            created_at: 1,
            finished_at: None,
            signal: None,
            nothing_new: false,
            waited: None,
        };
        fx.store.insert_run(&run).expect("run");
        (routine.id, run.id)
    }

    #[test]
    fn costs_add_up_per_bot_model_and_routine() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        let (morning, run) = routine_with_run(&fx, "Morning");
        let add = |bot, at, cost, model, run| {
            fx.store
                .add_turn_cost(&TurnCost {
                    bot,
                    at,
                    cost,
                    model,
                    run,
                })
                .expect("cost");
        };
        add(scout, 10, 0.5, None, None);
        add(scout, 20, 0.25, Some("claude-opus-5-5"), Some(&run));
        add(scout, 20, 0.05, Some("claude-haiku-5-5"), Some(&run));
        add(writer, 30, 1.0, Some("claude-opus-5-5"), None);

        let bots = fx.store.bot_costs(15).expect("bots");
        assert!((bots[scout] - 0.3).abs() < 1e-9);
        assert!((bots[writer] - 1.0).abs() < 1e-9);

        let models = fx.store.costs_by_model(0).expect("models");
        let keys: Vec<_> = models.iter().map(|group| group.key.as_str()).collect();
        assert_eq!(keys, ["claude-opus-5-5", "", "claude-haiku-5-5"]);
        assert!((models[0].cost - 1.25).abs() < 1e-9);

        let routines = fx.store.costs_by_routine(0).expect("routines");
        assert_eq!(routines.len(), 2);
        assert_eq!(routines[0].key, "", "the turns of no routine cost the most");
        assert!((routines[0].cost - 1.5).abs() < 1e-9);
        assert_eq!(routines[1].key, morning.as_str());
        assert_eq!(routines[1].name.as_deref(), Some("Morning"));
        assert_eq!(
            routines[1].bot_name.as_deref(),
            Some(fx.bots[0].name.as_str())
        );
        assert!((routines[1].cost - 0.3).abs() < 1e-9);

        let times = fx.store.cost_times(20).expect("times");
        assert_eq!(times.len(), 3);
        assert_eq!(times[2], (30, 1.0));
    }

    #[test]
    fn a_run_that_is_gone_leaves_no_routine() {
        let fx = Fixture::new();
        let gone = RoutineRunId::generate();
        fx.store
            .add_turn_cost(&TurnCost {
                bot: &fx.bots[0].id,
                at: 1,
                cost: 0.1,
                model: None,
                run: Some(&gone),
            })
            .expect("cost");
        let routines = fx.store.costs_by_routine(0).expect("routines");
        assert_eq!(routines[0].key, "");
        assert_eq!(routines[0].name, None);
    }
}
