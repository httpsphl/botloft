//! `routines` table (spec 20.7). Their runs are in `routine_runs.rs`.

use botloft_core::ids::{BotId, RoutineId};
use botloft_core::protocol::{Routine, Schedule};
use rusqlite::types::Type;
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

const COLUMNS: &str = "id, bot_id, name, prompt, schedule, timezone, overlap, missed, enabled, \
     next_run_at, created_at, updated_at, archived_at";
fn from_row(row: &Row<'_>) -> rusqlite::Result<Routine> {
    let schedule: String = row.get(4)?;
    let schedule: Schedule = serde_json::from_str(&schedule)
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(4, Type::Text, Box::new(err)))?;
    Ok(Routine {
        id: parse_column(row, 0)?,
        bot_id: parse_column(row, 1)?,
        name: row.get(2)?,
        prompt: row.get(3)?,
        schedule,
        timezone: row.get(5)?,
        overlap: parse_column(row, 6)?,
        missed: parse_column(row, 7)?,
        enabled: row.get(8)?,
        next_run_at: row.get(9)?,
        last_run: None,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
        archived_at: row.get(12)?,
    })
}

fn schedule_json(schedule: &Schedule) -> String {
    serde_json::to_string(schedule).unwrap_or_default()
}

impl Store {
    pub fn insert_routine(&self, routine: &Routine) -> Result<()> {
        self.conn.execute(
            &format!(
                "INSERT INTO routines ({COLUMNS}) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            ),
            params![
                routine.id.as_str(),
                routine.bot_id.as_str(),
                routine.name,
                routine.prompt,
                schedule_json(&routine.schedule),
                routine.timezone,
                routine.overlap.as_str(),
                routine.missed.as_str(),
                routine.enabled,
                routine.next_run_at,
                routine.created_at,
                routine.updated_at,
                routine.archived_at,
            ],
        )?;
        Ok(())
    }

    /// Saves every field except the id, the bot and the creation time.
    pub fn update_routine(&self, routine: &Routine) -> Result<()> {
        self.conn.execute(
            "UPDATE routines SET name = ?2, prompt = ?3, schedule = ?4, timezone = ?5, \
             overlap = ?6, missed = ?7, enabled = ?8, next_run_at = ?9, updated_at = ?10, \
             archived_at = ?11 WHERE id = ?1",
            params![
                routine.id.as_str(),
                routine.name,
                routine.prompt,
                schedule_json(&routine.schedule),
                routine.timezone,
                routine.overlap.as_str(),
                routine.missed.as_str(),
                routine.enabled,
                routine.next_run_at,
                routine.updated_at,
                routine.archived_at,
            ],
        )?;
        Ok(())
    }

    /// The routine with its latest run.
    pub fn routine(&self, id: &RoutineId) -> Result<Option<Routine>> {
        let found = self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM routines WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?;
        found.map(|routine| self.with_last_run(routine)).transpose()
    }

    /// Routines in creation order, optionally of one bot, with their latest run.
    pub fn routines(&self, bot: Option<&BotId>, include_archived: bool) -> Result<Vec<Routine>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM routines \
             WHERE (?1 IS NULL OR bot_id = ?1) AND (?2 OR archived_at IS NULL) \
             ORDER BY created_at, id"
        ))?;
        let rows = stmt.query_map(params![bot.map(BotId::as_str), include_archived], from_row)?;
        let routines = rows.collect::<rusqlite::Result<Vec<_>>>()?;
        routines
            .into_iter()
            .map(|routine| self.with_last_run(routine))
            .collect()
    }

    /// Enabled routines whose time came, soonest first.
    pub fn due_routines(&self, now: i64) -> Result<Vec<Routine>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM routines \
             WHERE enabled AND archived_at IS NULL AND next_run_at <= ?1 \
             ORDER BY next_run_at, id"
        ))?;
        let rows = stmt.query_map([now], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The soonest time any enabled routine comes due.
    pub fn next_routine_at(&self) -> Result<Option<i64>> {
        Ok(self.conn.query_row(
            "SELECT MIN(next_run_at) FROM routines WHERE enabled AND archived_at IS NULL",
            [],
            |row| row.get(0),
        )?)
    }

    /// Archives the routines of a bot that was archived. Returns their ids.
    pub fn archive_routines_of(&self, bot: &BotId, at: i64) -> Result<Vec<RoutineId>> {
        let mut stmt = self.conn.prepare(
            "UPDATE routines SET archived_at = ?2, updated_at = ?2, next_run_at = NULL \
             WHERE bot_id = ?1 AND archived_at IS NULL RETURNING id",
        )?;
        let rows = stmt.query_map(params![bot.as_str(), at], |row| parse_column(row, 0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    fn with_last_run(&self, mut routine: Routine) -> Result<Routine> {
        routine.last_run = self.runs(&routine.id, None, 1)?.pop();
        Ok(routine)
    }
}

#[cfg(test)]
mod tests;
