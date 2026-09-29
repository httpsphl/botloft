//! `routine_runs` table (spec 20.7). A run that sends a message is saved
//! with that message, its delivery and the chat item, in one transaction.

use botloft_core::ids::{MessageId, RoutineId, RoutineRunId};
use botloft_core::protocol::{ChatItem, Delivery, Message, RoutineRun, RunStatus};
use rusqlite::{OptionalExtension, Row, params};

use crate::messages::optional_column;
use crate::{Result, Store, parse_column};

pub(crate) const RUN_COLUMNS: &str = "id, routine_id, scheduled_for, status, reason, skipped_count, message_id, created_at, finished_at";

pub(crate) fn run_from_row(row: &Row<'_>) -> rusqlite::Result<RoutineRun> {
    Ok(RoutineRun {
        id: parse_column(row, 0)?,
        routine_id: parse_column(row, 1)?,
        scheduled_for: row.get(2)?,
        status: parse_column(row, 3)?,
        reason: optional_column(row, 4)?,
        skipped_count: u32::try_from(row.get::<_, i64>(5)?).unwrap_or(0),
        message_id: optional_column(row, 6)?,
        created_at: row.get(7)?,
        finished_at: row.get(8)?,
    })
}

impl Store {
    /// Records a run that sends nothing: a skip.
    pub fn insert_run(&self, run: &RoutineRun) -> Result<()> {
        Self::insert_run_in(&self.conn, run)
    }

    /// Records a run with the message it sends, the message's delivery and
    /// the recipient's chat item, atomically. Returns the item.
    pub fn fire_run(
        &self,
        run: &RoutineRun,
        message: &Message,
        delivery: &Delivery,
    ) -> Result<ChatItem> {
        let tx = self.conn.unchecked_transaction()?;
        let item = Self::insert_message_in(&tx, message, delivery)?;
        Self::insert_run_in(&tx, run)?;
        tx.commit()?;
        Ok(item)
    }

    fn insert_run_in(conn: &rusqlite::Connection, run: &RoutineRun) -> Result<()> {
        conn.execute(
            &format!(
                "INSERT INTO routine_runs ({RUN_COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            ),
            params![
                run.id.as_str(),
                run.routine_id.as_str(),
                run.scheduled_for,
                run.status.as_str(),
                run.reason.map(|reason| reason.as_str()),
                i64::from(run.skipped_count),
                run.message_id.as_ref().map(MessageId::as_str),
                run.created_at,
                run.finished_at,
            ],
        )?;
        Ok(())
    }

    /// How many runs of the routine are still open.
    pub fn open_runs(&self, routine: &RoutineId) -> Result<u32> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM routine_runs WHERE routine_id = ?1 AND status = 'queued'",
            [routine.as_str()],
            |row| row.get(0),
        )?;
        Ok(u32::try_from(count).unwrap_or(0))
    }

    /// The run that sent `message`, if a routine sent it.
    pub fn run_of_message(&self, message: &MessageId) -> Result<Option<RoutineRun>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {RUN_COLUMNS} FROM routine_runs WHERE message_id = ?1"),
                [message.as_str()],
                run_from_row,
            )
            .optional()?)
    }

    /// Ends an open run. `None` if it was no longer open.
    pub fn finish_run(
        &self,
        id: &RoutineRunId,
        status: RunStatus,
        now: i64,
    ) -> Result<Option<RoutineRun>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "UPDATE routine_runs SET status = ?2, finished_at = ?3 \
                     WHERE id = ?1 AND status = 'queued' RETURNING {RUN_COLUMNS}"
                ),
                params![id.as_str(), status.as_str(), now],
                run_from_row,
            )
            .optional()?)
    }

    /// Open runs whose message will never arrive: its delivery is dead.
    pub fn runs_with_dead_delivery(&self) -> Result<Vec<RoutineRun>> {
        let columns = RUN_COLUMNS
            .split(", ")
            .map(|column| format!("r.{column}"))
            .collect::<Vec<_>>()
            .join(", ");
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {columns} FROM routine_runs r \
             JOIN deliveries d ON d.message_id = r.message_id \
             WHERE r.status = 'queued' AND d.state = 'dead'"
        ))?;
        let rows = stmt.query_map([], run_from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Open runs whose message was already read: their turn began, and
    /// only the process that had it could end it.
    pub fn stale_runs(&self) -> Result<Vec<RoutineRun>> {
        let columns = RUN_COLUMNS
            .split(", ")
            .map(|column| format!("r.{column}"))
            .collect::<Vec<_>>()
            .join(", ");
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {columns} FROM routine_runs r \
             JOIN deliveries d ON d.message_id = r.message_id \
             WHERE r.status = 'queued' AND d.read_at IS NOT NULL"
        ))?;
        let rows = stmt.query_map([], run_from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Runs of a routine, newest first; `before` pages back.
    pub fn runs(
        &self,
        routine: &RoutineId,
        before: Option<&RoutineRunId>,
        limit: u32,
    ) -> Result<Vec<RoutineRun>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {RUN_COLUMNS} FROM routine_runs \
             WHERE routine_id = ?1 \
               AND (?2 IS NULL OR rowid < (SELECT rowid FROM routine_runs WHERE id = ?2)) \
             ORDER BY rowid DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(
            params![routine.as_str(), before.map(RoutineRunId::as_str), limit],
            run_from_row,
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
