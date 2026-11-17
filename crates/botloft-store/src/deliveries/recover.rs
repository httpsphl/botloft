//! What a daemon that stopped left halfway in the queue (spec 9.1).

use botloft_core::protocol::Delivery;

use super::{COLUMNS, from_row};
use crate::{Result, Store};

impl Store {
    /// Returns deliveries whose sender died mid-send to `pending`.
    pub fn recover_leases(&self, now: i64) -> Result<Vec<Delivery>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "UPDATE deliveries SET state = 'pending', lease_until = NULL, updated_at = ?1 \
             WHERE state = 'sending' AND lease_until <= ?1 RETURNING {COLUMNS}"
        ))?;
        let rows = stmt.query_map([now], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// At start, before any bot runs: deliveries written to a process of the
    /// last run that never began them go back to `pending`, due now. The
    /// daemon stopped, not the message, so no attempt is counted; the queue
    /// keeps its order, which is the rowid's.
    pub fn reopen_left_behind(&self, error: &str, now: i64) -> Result<Vec<Delivery>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "UPDATE deliveries SET state = 'pending', next_attempt_at = ?2, last_error = ?1, \
             updated_at = ?2, sent_generation = NULL, turn_uuid = NULL \
             WHERE state = 'sent' AND read_at IS NULL RETURNING {COLUMNS}"
        ))?;
        let rows = stmt.query_map(rusqlite::params![error, now], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
