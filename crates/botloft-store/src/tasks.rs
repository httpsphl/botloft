//! `tasks` table (spec 9.4). A task is inserted with the message that asks
//! for it, in [`Store::insert_message`], and settled together with the
//! message that reports the outcome.

use botloft_core::ids::{BotId, CrewId, TaskId};
use botloft_core::protocol::{ChatItem, Delivery, Message, Task, TaskStatus};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::messages::optional_column;
use crate::{Result, Store, parse_column};

const COLUMNS: &str = "id, crew_id, requester_bot_id, assignee_bot_id, status, deadline_at, \
                       hops, origin_task_id, result, created_at, updated_at";

/// Longest list `tasks` returns.
const LIST_LIMIT: u32 = 500;

fn from_row(row: &Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: parse_column(row, 0)?,
        crew_id: parse_column(row, 1)?,
        requester_bot_id: parse_column(row, 2)?,
        assignee_bot_id: parse_column(row, 3)?,
        status: parse_column(row, 4)?,
        deadline_at: row.get(5)?,
        hops: row.get(6)?,
        origin_task_id: optional_column(row, 7)?,
        result: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

/// Which tasks `tasks` returns, newest first.
#[derive(Debug, Clone, Copy, Default)]
pub struct TaskFilter<'a> {
    pub crew: Option<&'a CrewId>,
    pub assignee: Option<&'a BotId>,
    pub requester: Option<&'a BotId>,
    /// Only tasks in one of these; any status when empty.
    pub statuses: &'a [TaskStatus],
}

/// `,open,expired,`: matched with `instr`, so one parameter holds the set.
fn status_set(statuses: &[TaskStatus]) -> Option<String> {
    (!statuses.is_empty()).then(|| {
        let names: Vec<_> = statuses.iter().map(|status| status.as_str()).collect();
        format!(",{},", names.join(","))
    })
}

impl Store {
    pub(crate) fn insert_task_in(conn: &Connection, task: &Task) -> Result<()> {
        conn.execute(
            &format!(
                "INSERT INTO tasks ({COLUMNS}) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
            ),
            params![
                task.id.as_str(),
                task.crew_id.as_str(),
                task.requester_bot_id.as_str(),
                task.assignee_bot_id.as_str(),
                task.status.as_str(),
                task.deadline_at,
                task.hops,
                task.origin_task_id.as_ref().map(TaskId::as_str),
                task.result,
                task.created_at,
                task.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn task(&self, id: &TaskId) -> Result<Option<Task>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM tasks WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Newest first, at most 500.
    pub fn tasks(&self, filter: TaskFilter<'_>) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM tasks \
             WHERE (?1 IS NULL OR crew_id = ?1) \
               AND (?2 IS NULL OR assignee_bot_id = ?2) \
               AND (?3 IS NULL OR requester_bot_id = ?3) \
               AND (?4 IS NULL OR instr(?4, ',' || status || ',') > 0) \
             ORDER BY rowid DESC LIMIT ?5"
        ))?;
        let rows = stmt.query_map(
            params![
                filter.crew.map(CrewId::as_str),
                filter.assignee.map(BotId::as_str),
                filter.requester.map(BotId::as_str),
                status_set(filter.statuses),
                LIST_LIMIT,
            ],
            from_row,
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Open tasks whose deadline has passed, oldest deadline first.
    pub fn overdue_tasks(&self, now: i64) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM tasks WHERE status = 'open' AND deadline_at <= ?1 \
             ORDER BY deadline_at, rowid"
        ))?;
        let rows = stmt.query_map([now], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Moves a task from one of `from` to `to`, keeps `result` when given
    /// and saves the message that reports it, all or nothing. Returns the
    /// task and the recipient's chat item; `None`, with nothing saved, if
    /// the task was in no state of `from`.
    pub fn settle_task(
        &self,
        id: &TaskId,
        from: &[TaskStatus],
        to: TaskStatus,
        result: Option<&str>,
        now: i64,
        report: (&Message, &Delivery),
    ) -> Result<Option<(Task, ChatItem)>> {
        let tx = self.conn.unchecked_transaction()?;
        let settled = tx
            .query_row(
                &format!(
                    "UPDATE tasks SET status = ?2, result = COALESCE(?3, result), updated_at = ?4 \
                     WHERE id = ?1 AND instr(?5, ',' || status || ',') > 0 RETURNING {COLUMNS}"
                ),
                params![id.as_str(), to.as_str(), result, now, status_set(from)],
                from_row,
            )
            .optional()?;
        let Some(settled) = settled else {
            return Ok(None);
        };
        let item = Self::insert_message_in(&tx, report.0, report.1)?;
        tx.commit()?;
        Ok(Some((settled, item)))
    }
}

#[cfg(test)]
mod tests;
