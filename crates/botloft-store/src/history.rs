//! Forgetting the history older than the owner keeps (spec 12.1). Off by
//! default: everything stays until the owner picks a number of days. What
//! still waits for someone never goes: a request or a question with no
//! answer, a message not yet delivered or begun, an open task's messages, a
//! routine run in progress. The files in the bots' folders are the owner's
//! and stay.

use std::ops::AddAssign;

use rusqlite::{Connection, params};

use crate::{Result, Store};

/// What one pass removed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Pruned {
    pub chat_items: usize,
    pub messages: usize,
    pub routine_runs: usize,
}

impl Pruned {
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

impl AddAssign for Pruned {
    fn add_assign(&mut self, other: Self) {
        self.chat_items += other.chat_items;
        self.messages += other.messages;
        self.routine_runs += other.routine_runs;
    }
}

/// The chat items a pass takes, oldest first: older than `?1`, `?2` at most.
const OLD_ITEMS: &str = "INSERT INTO temp.pruned (id) SELECT c.id FROM chat_items c \
     WHERE c.created_at < ?1 \
       AND NOT EXISTS (SELECT 1 FROM approvals a WHERE a.chat_item_id = c.id AND a.status = 'pending') \
       AND NOT EXISTS (SELECT 1 FROM questions q WHERE q.chat_item_id = c.id AND q.status = 'open') \
     ORDER BY c.rowid LIMIT ?2";

/// What points at those items goes first; an answer keeps its message.
const ITEMS: &[&str] = &[
    "UPDATE messages SET question_id = NULL WHERE question_id IN \
     (SELECT id FROM questions WHERE chat_item_id IN (SELECT id FROM temp.pruned))",
    "DELETE FROM questions WHERE chat_item_id IN (SELECT id FROM temp.pruned)",
    "DELETE FROM reactions WHERE item_id IN (SELECT id FROM temp.pruned)",
    "DELETE FROM approvals WHERE chat_item_id IN (SELECT id FROM temp.pruned)",
];

/// The messages a pass takes: every delivery of theirs read or given up,
/// no open task or waiting run behind them, and no reaction that went with
/// them (it goes with its chat item, and the message in a later pass).
const OLD_MESSAGES: &str = "INSERT INTO temp.pruned (id) SELECT m.id FROM messages m \
     WHERE m.created_at < ?1 \
       AND NOT EXISTS (SELECT 1 FROM deliveries d WHERE d.message_id = m.id \
             AND NOT (d.state = 'dead' OR (d.state = 'sent' AND d.read_at IS NOT NULL))) \
       AND NOT EXISTS (SELECT 1 FROM tasks t WHERE t.id = m.task_id AND t.status = 'open') \
       AND NOT EXISTS (SELECT 1 FROM routine_runs r WHERE r.message_id = m.id AND r.status = 'queued') \
       AND NOT EXISTS (SELECT 1 FROM reactions x WHERE x.sent_in = m.id) \
     ORDER BY m.rowid LIMIT ?2";

const MESSAGES: &[&str] = &[
    "UPDATE routine_runs SET message_id = NULL WHERE message_id IN (SELECT id FROM temp.pruned)",
    // The rows only: the files stay in the bot's folder.
    "DELETE FROM attachments WHERE message_id IN (SELECT id FROM temp.pruned)",
    "DELETE FROM deliveries WHERE message_id IN (SELECT id FROM temp.pruned)",
];

const OLD_RUNS: &str = "DELETE FROM routine_runs WHERE id IN (SELECT id FROM routine_runs \
     WHERE status <> 'queued' AND created_at < ?1 ORDER BY rowid LIMIT ?2)";

impl Store {
    /// Removes what is older than `before` (Unix ms): up to `batch` chat
    /// items, `batch` messages with their deliveries and `batch` finished
    /// routine runs, in one transaction. Call it again, letting others use
    /// the store in between, until it removes nothing.
    pub fn prune_history(&self, before: i64, batch: usize) -> Result<Pruned> {
        let batch = i64::try_from(batch).unwrap_or(i64::MAX);
        let tx = self.conn.unchecked_transaction()?;
        tx.execute_batch("CREATE TEMP TABLE IF NOT EXISTS pruned (id TEXT PRIMARY KEY)")?;
        let chat_items = take(&tx, OLD_ITEMS, ITEMS, "chat_items", before, batch)?;
        let messages = take(&tx, OLD_MESSAGES, MESSAGES, "messages", before, batch)?;
        let routine_runs = tx.execute(OLD_RUNS, params![before, batch])?;
        tx.execute_batch("DELETE FROM temp.pruned")?;
        tx.commit()?;
        Ok(Pruned {
            chat_items,
            messages,
            routine_runs,
        })
    }
}

/// Picks rows with `pick`, lets go of them with `before_delete`, then
/// deletes them from `table`.
fn take(
    conn: &Connection,
    pick: &str,
    before_delete: &[&str],
    table: &str,
    before: i64,
    batch: i64,
) -> Result<usize> {
    conn.execute_batch("DELETE FROM temp.pruned")?;
    conn.execute(pick, params![before, batch])?;
    for statement in before_delete {
        conn.execute(statement, [])?;
    }
    Ok(conn.execute(
        &format!("DELETE FROM {table} WHERE id IN (SELECT id FROM temp.pruned)"),
        [],
    )?)
}

#[cfg(test)]
mod tests;
