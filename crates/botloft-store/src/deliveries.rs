//! `deliveries` table: the courier's queue (spec 9.1). Every transition is
//! one conditional UPDATE, so a delivery never changes state twice by
//! accident.

use botloft_core::ids::{BotId, DeliveryId};
use botloft_core::protocol::{Delivery, DeliveryBacklog, DeliveryState};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{Result, Store, parse_column, to_sql_int};

const COLUMNS: &str =
    "id, message_id, bot_id, state, attempts, next_attempt_at, last_error, read_at, updated_at";

/// Longest list `deliveries` returns.
const LIST_LIMIT: u32 = 500;

fn from_row(row: &Row<'_>) -> rusqlite::Result<Delivery> {
    Ok(Delivery {
        id: parse_column(row, 0)?,
        message_id: parse_column(row, 1)?,
        bot_id: parse_column(row, 2)?,
        state: parse_column(row, 3)?,
        attempts: row.get(4)?,
        next_attempt_at: row.get(5)?,
        last_error: row.get(6)?,
        read_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

/// How an attempt, or a decision not to try, ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeliveryOutcome<'a> {
    /// Written to the process of `generation` with `turn_uuid` (spec 9.2).
    Sent { generation: u64, turn_uuid: &'a str },
    /// The bot is not ready: try again at `until`, without counting an attempt.
    Defer { until: i64 },
    /// Counts an attempt. `retry_at: None` gives up.
    Failed {
        error: &'a str,
        retry_at: Option<i64>,
    },
    /// Gives up without counting an attempt, e.g. the bot was archived.
    Dead { error: &'a str },
}

impl Store {
    pub(crate) fn insert_delivery_in(conn: &Connection, delivery: &Delivery) -> Result<()> {
        conn.execute(
            &format!(
                "INSERT INTO deliveries ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
            ),
            params![
                delivery.id.as_str(),
                delivery.message_id.as_str(),
                delivery.bot_id.as_str(),
                delivery.state.as_str(),
                delivery.attempts,
                delivery.next_attempt_at,
                delivery.last_error,
                delivery.read_at,
                delivery.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delivery(&self, id: &DeliveryId) -> Result<Option<Delivery>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM deliveries WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Most recently updated first, at most 500.
    pub fn deliveries(
        &self,
        state: Option<DeliveryState>,
        bot: Option<&BotId>,
    ) -> Result<Vec<Delivery>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM deliveries \
             WHERE (?1 IS NULL OR state = ?1) AND (?2 IS NULL OR bot_id = ?2) \
             ORDER BY updated_at DESC, rowid DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(
            params![
                state.map(DeliveryState::as_str),
                bot.map(BotId::as_str),
                LIST_LIMIT
            ],
            from_row,
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The delivery each bot may send now: its oldest pending one, if it is
    /// due and nothing else of that bot is being sent. Keeps each bot's
    /// messages in order, one at a time.
    pub fn due_deliveries(&self, now: i64) -> Result<Vec<Delivery>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM deliveries AS d \
             WHERE d.state = 'pending' AND d.next_attempt_at <= ?1 \
               AND d.rowid = (SELECT MIN(rowid) FROM deliveries \
                              WHERE bot_id = d.bot_id AND state = 'pending') \
               AND NOT EXISTS (SELECT 1 FROM deliveries \
                               WHERE bot_id = d.bot_id AND state = 'sending') \
             ORDER BY d.rowid"
        ))?;
        let rows = stmt.query_map([now], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Moves a pending delivery to `sending` until `lease_until`. `None` if
    /// it was no longer pending.
    pub fn claim_delivery(
        &self,
        id: &DeliveryId,
        now: i64,
        lease_until: i64,
    ) -> Result<Option<Delivery>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "UPDATE deliveries SET state = 'sending', lease_until = ?2, updated_at = ?3 \
                     WHERE id = ?1 AND state = 'pending' RETURNING {COLUMNS}"
                ),
                params![id.as_str(), lease_until, now],
                from_row,
            )
            .optional()?)
    }

    /// Records how a pending or sending delivery ended. `None` if it was in
    /// neither state.
    pub fn finish_delivery(
        &self,
        id: &DeliveryId,
        outcome: DeliveryOutcome<'_>,
        now: i64,
    ) -> Result<Option<Delivery>> {
        let update = |set: &str, values: &[&dyn rusqlite::ToSql]| {
            self.conn
                .query_row(
                    &format!(
                        "UPDATE deliveries SET {set}, lease_until = NULL \
                         WHERE id = ?1 AND state IN ('pending', 'sending') RETURNING {COLUMNS}"
                    ),
                    values,
                    from_row,
                )
                .optional()
        };
        let id = id.as_str();
        let row = match outcome {
            DeliveryOutcome::Sent {
                generation,
                turn_uuid,
            } => update(
                "state = 'sent', last_error = NULL, updated_at = ?2, sent_generation = ?3, \
                 turn_uuid = ?4, read_at = NULL",
                params![id, now, to_sql_int(generation), turn_uuid],
            ),
            // Waiting is not news: updated_at stays, so lists do not churn.
            DeliveryOutcome::Defer { until } => update(
                "state = 'pending', next_attempt_at = ?2",
                params![id, until],
            ),
            DeliveryOutcome::Failed {
                error,
                retry_at: Some(at),
            } => update(
                "state = 'pending', attempts = attempts + 1, next_attempt_at = ?2, \
                 last_error = ?3, updated_at = ?4",
                params![id, at, error, now],
            ),
            DeliveryOutcome::Failed {
                error,
                retry_at: None,
            } => update(
                "state = 'dead', attempts = attempts + 1, last_error = ?2, updated_at = ?3",
                params![id, error, now],
            ),
            DeliveryOutcome::Dead { error } => update(
                "state = 'dead', last_error = ?2, updated_at = ?3",
                params![id, error, now],
            ),
        };
        Ok(row?)
    }

    /// The bot began the turn for the delivery sent with `turn_uuid`
    /// (spec 9.1). `None` if no unread delivery went with that uuid.
    pub fn mark_read(&self, turn_uuid: &str, now: i64) -> Result<Option<Delivery>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "UPDATE deliveries SET read_at = ?2, updated_at = ?2 \
                     WHERE turn_uuid = ?1 AND state = 'sent' AND read_at IS NULL \
                     RETURNING {COLUMNS}"
                ),
                params![turn_uuid, now],
                from_row,
            )
            .optional()?)
    }

    /// Deliveries written to the process of `generation` that it never
    /// began, oldest first.
    pub fn unread_deliveries(&self, bot: &BotId, generation: u64) -> Result<Vec<Delivery>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM deliveries \
             WHERE bot_id = ?1 AND sent_generation = ?2 AND state = 'sent' AND read_at IS NULL \
             ORDER BY rowid"
        ))?;
        let rows = stmt.query_map(params![bot.as_str(), to_sql_int(generation)], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Puts a sent but unread delivery back in the queue, counting an
    /// attempt; `retry_at: None` gives up. `None` if it was read meanwhile.
    pub fn reopen_unread(
        &self,
        id: &DeliveryId,
        error: &str,
        retry_at: Option<i64>,
        now: i64,
    ) -> Result<Option<Delivery>> {
        let (state, next) = match retry_at {
            Some(at) => ("pending", at),
            None => ("dead", now),
        };
        Ok(self
            .conn
            .query_row(
                &format!(
                    "UPDATE deliveries SET state = ?2, attempts = attempts + 1, \
                     next_attempt_at = ?3, last_error = ?4, updated_at = ?5, \
                     sent_generation = NULL, turn_uuid = NULL \
                     WHERE id = ?1 AND state = 'sent' AND read_at IS NULL RETURNING {COLUMNS}"
                ),
                params![id.as_str(), state, next, error, now],
                from_row,
            )
            .optional()?)
    }

    /// Returns deliveries whose sender died mid-send to `pending`.
    pub fn recover_leases(&self, now: i64) -> Result<Vec<Delivery>> {
        let mut stmt = self.conn.prepare(&format!(
            "UPDATE deliveries SET state = 'pending', lease_until = NULL, updated_at = ?1 \
             WHERE state = 'sending' AND lease_until <= ?1 RETURNING {COLUMNS}"
        ))?;
        let rows = stmt.query_map([now], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Puts a dead delivery back in the queue with its attempts reset.
    /// `None` if it is not dead.
    pub fn retry_delivery(&self, id: &DeliveryId, now: i64) -> Result<Option<Delivery>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "UPDATE deliveries SET state = 'pending', attempts = 0, next_attempt_at = ?2, \
                     last_error = NULL, updated_at = ?2 \
                     WHERE id = ?1 AND state = 'dead' RETURNING {COLUMNS}"
                ),
                params![id.as_str(), now],
                from_row,
            )
            .optional()?)
    }

    pub fn delivery_backlog(&self) -> Result<DeliveryBacklog> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(SUM(state IN ('pending', 'sending')), 0), \
                    COALESCE(SUM(state = 'dead'), 0) FROM deliveries",
            [],
            |row| {
                Ok(DeliveryBacklog {
                    pending: row.get(0)?,
                    dead: row.get(1)?,
                })
            },
        )?)
    }
}

mod courier;
#[cfg(test)]
mod tests;
