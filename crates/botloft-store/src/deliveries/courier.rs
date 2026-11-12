//! What the courier asks to plan its sleep and to catch up with a bot that
//! became ready (spec 9.1).

use botloft_core::ids::{BotId, ConversationId, DeliveryId};
use rusqlite::params;

use crate::{Result, Store};

impl Store {
    /// When the courier has something to do next without being woken: the
    /// first delivery of a bot's queue comes due, a lease runs out or an
    /// open task passes its deadline. `None` when nothing waits for a time.
    pub fn courier_next_at(&self) -> Result<Option<i64>> {
        Ok(self
            .conn
            .prepare_cached(
                "SELECT MIN(at) FROM ( \
               SELECT MIN(d.next_attempt_at) AS at FROM deliveries AS d \
               WHERE d.state = 'pending' \
                 AND d.rowid = (SELECT MIN(rowid) FROM deliveries \
                                WHERE bot_id = d.bot_id AND state = 'pending') \
               UNION ALL \
               SELECT MIN(lease_until) FROM deliveries WHERE state = 'sending' \
               UNION ALL \
               SELECT MIN(deadline_at) FROM tasks WHERE status = 'open')",
            )?
            .query_row([], |row| row.get(0))?)
    }

    /// A bot became ready: what waited only for that (never tried and
    /// failed) goes now instead of at its next check (spec 9.1). Deliveries
    /// that failed keep their backoff, so a message that makes the bot crash
    /// does not come straight back.
    pub fn hasten_deliveries(&self, bot: &BotId, now: i64) -> Result<usize> {
        Ok(self
            .conn
            .prepare_cached(
                "UPDATE deliveries SET next_attempt_at = ?2 \
             WHERE bot_id = ?1 AND state = 'pending' AND attempts = 0 AND next_attempt_at > ?2",
            )?
            .execute(params![bot.as_str(), now])?)
    }

    /// A pending delivery goes to another conversation of its bot: the
    /// main one, when the one it waited for was archived (spec 40.2).
    pub fn move_delivery(&self, id: &DeliveryId, conversation: &ConversationId) -> Result<()> {
        self.conn
            .prepare_cached(
                "UPDATE deliveries SET conversation_id = ?2 WHERE id = ?1 AND state = 'pending'",
            )?
            .execute(params![id.as_str(), conversation.as_str()])?;
        Ok(())
    }

    /// Every pending delivery of conversation `from` goes to `to`, as one
    /// the owner archives gives what waited for it to the main one.
    pub fn move_deliveries(&self, from: &ConversationId, to: &ConversationId) -> Result<usize> {
        Ok(self
            .conn
            .prepare_cached(
                "UPDATE deliveries SET conversation_id = ?2 \
                 WHERE conversation_id = ?1 AND state = 'pending'",
            )?
            .execute(params![from.as_str(), to.as_str()])?)
    }

    /// The bots a message waits for, to go out or going out now: they come
    /// first when only so many agents may run (spec 7.8).
    pub fn bots_with_work(&self) -> Result<std::collections::HashSet<BotId>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT DISTINCT bot_id FROM deliveries WHERE state IN ('pending', 'sending')",
        )?;
        let rows = stmt.query_map([], |row| crate::parse_column(row, 0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
