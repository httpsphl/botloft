//! Routines that wait for a signal from a bot of their crew (spec 20.13).

use botloft_core::ids::{BotId, CrewId, RoutineId};
use botloft_core::protocol::Routine;
use rusqlite::params;

use crate::routines::{COLUMNS, from_row};
use crate::{Result, Store};

impl Store {
    /// The enabled routines of active bots in `crew` that wait for a
    /// signal: all of them, or only those for `name`. In creation order.
    pub fn signal_routines(&self, crew: &CrewId, name: Option<&str>) -> Result<Vec<Routine>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM routines \
             WHERE enabled AND archived_at IS NULL \
               AND json_extract(schedule, '$.kind') = 'signal' \
               AND (?2 IS NULL OR json_extract(schedule, '$.name') = ?2) \
               AND bot_id IN (SELECT id FROM bots WHERE crew_id = ?1 AND archived_at IS NULL) \
             ORDER BY created_at, id"
        ))?;
        let rows = stmt.query_map(params![crew.as_str(), name], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The waits still open (spec 20.14): enabled, of active bots in
    /// active crews, oldest first.
    pub fn wait_routines(&self) -> Result<Vec<Routine>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM routines              WHERE enabled AND archived_at IS NULL                AND json_extract(schedule, '$.kind') = 'wait'                AND bot_id IN (SELECT b.id FROM bots b JOIN crews k ON k.id = b.crew_id                               WHERE b.archived_at IS NULL AND k.archived_at IS NULL)              ORDER BY created_at, id"
        ))?;
        let rows = stmt.query_map([], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// How many waits the bot has open, enabled or not.
    pub fn open_waits(&self, bot: &BotId) -> Result<u32> {
        Ok(self.conn.query_row(
            "SELECT count(*) FROM routines              WHERE bot_id = ?1 AND archived_at IS NULL                AND json_extract(schedule, '$.kind') = 'wait'",
            [bot.as_str()],
            |row| row.get(0),
        )?)
    }

    /// When a signal last made the routine run (a skip does not count).
    pub fn last_signal_run_at(&self, routine: &RoutineId) -> Result<Option<i64>> {
        Ok(self.conn.query_row(
            "SELECT MAX(created_at) FROM routine_runs \
             WHERE routine_id = ?1 AND signal_name IS NOT NULL AND status != 'skipped'",
            [routine.as_str()],
            |row| row.get(0),
        )?)
    }
}
