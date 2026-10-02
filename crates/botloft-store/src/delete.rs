//! Deleting a bot or a crew for good (spec 7.6 and 12). Archiving keeps the
//! rows; this removes them with everything that points at them, in one
//! transaction.

use botloft_core::ids::{BotId, CrewId};
use rusqlite::Connection;

use crate::{Result, Store};

/// Every statement takes the bot's id. The order follows the foreign keys:
/// what points at a row goes, or lets go of it, before the row.
const DELETE_BOT: &[&str] = &[
    "DELETE FROM approvals WHERE bot_id = ?1",
    "DELETE FROM chat_items WHERE bot_id = ?1",
    "DELETE FROM browser_sites WHERE bot_id = ?1",
    "DELETE FROM allow_rules WHERE bot_id = ?1",
    "DELETE FROM routine_runs WHERE routine_id IN (SELECT id FROM routines WHERE bot_id = ?1)",
    // What the bot received goes with it.
    "UPDATE routine_runs SET message_id = NULL \
     WHERE message_id IN (SELECT id FROM messages WHERE to_bot_id = ?1)",
    "DELETE FROM attachments WHERE message_id IN (SELECT id FROM messages WHERE to_bot_id = ?1)",
    "DELETE FROM deliveries WHERE bot_id = ?1 \
     OR message_id IN (SELECT id FROM messages WHERE to_bot_id = ?1)",
    "DELETE FROM messages WHERE to_bot_id = ?1",
    "UPDATE messages SET routine_id = NULL \
     WHERE routine_id IN (SELECT id FROM routines WHERE bot_id = ?1)",
    "DELETE FROM routines WHERE bot_id = ?1",
    // What it sent stays with who received it, without a sender.
    "UPDATE messages SET from_bot_id = NULL WHERE from_bot_id = ?1",
    "UPDATE messages SET task_id = NULL WHERE task_id IN \
     (SELECT id FROM tasks WHERE requester_bot_id = ?1 OR assignee_bot_id = ?1)",
    "UPDATE tasks SET origin_task_id = NULL WHERE origin_task_id IN \
     (SELECT id FROM tasks WHERE requester_bot_id = ?1 OR assignee_bot_id = ?1)",
    "DELETE FROM tasks WHERE requester_bot_id = ?1 OR assignee_bot_id = ?1",
    "UPDATE crews SET lead_bot_id = NULL WHERE lead_bot_id = ?1",
];

fn delete_bot_in(conn: &Connection, bot: &str) -> Result<bool> {
    for statement in DELETE_BOT {
        conn.execute(statement, [bot])?;
    }
    Ok(conn.execute("DELETE FROM bots WHERE id = ?1", [bot])? > 0)
}

impl Store {
    /// Deletes the bot with its chat, its approvals, the messages it
    /// received, its routines, its browser sites and the tasks it asked for
    /// or was given. Messages it sent to other bots stay, without a sender.
    /// `false`, with nothing changed, if there is no such bot.
    pub fn delete_bot(&self, id: &BotId) -> Result<bool> {
        let tx = self.conn.unchecked_transaction()?;
        let deleted = delete_bot_in(&tx, id.as_str())?;
        tx.commit()?;
        Ok(deleted)
    }

    /// Deletes the crew and every bot in it, archived ones too, as
    /// [`Store::delete_bot`] does. `false` if there is no such crew.
    pub fn delete_crew(&self, id: &CrewId) -> Result<bool> {
        let tx = self.conn.unchecked_transaction()?;
        let bots: Vec<String> = {
            let mut stmt = tx.prepare_cached("SELECT id FROM bots WHERE crew_id = ?1")?;
            let rows = stmt.query_map([id.as_str()], |row| row.get(0))?;
            rows.collect::<rusqlite::Result<_>>()?
        };
        for bot in &bots {
            delete_bot_in(&tx, bot)?;
        }
        // Every message and task of a crew is to or for one of its bots, so
        // nothing of the crew is left to hold it.
        let deleted = tx.execute("DELETE FROM crews WHERE id = ?1", [id.as_str()])? > 0;
        tx.commit()?;
        Ok(deleted)
    }
}

#[cfg(test)]
mod tests;
