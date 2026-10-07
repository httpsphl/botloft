//! The light copy of the database (spec 14.2): what is worth carrying to the
//! cloud, which is the structure, the memory and the routines, and not the
//! conversations or what the computer did.

use std::path::Path;

use rusqlite::Connection;

use crate::{Result, Store};

/// Emptied in the light copy, children before parents. The chats, what the
/// bots were asked and answered, the runs of the routines, the usage of the
/// plan and what this computer let a bot use on the desktop.
const EMPTIED: &[&str] = &[
    "reactions",
    "questions",
    "approvals",
    "attachments",
    "deliveries",
    "tasks",
    "messages",
    "chat_items",
    "routine_runs",
    "turn_costs",
    "plan_readings",
    "desktop_grants",
];

impl Store {
    /// A copy of the database at `path` (which must not exist yet) without
    /// the conversations and the rest of [`EMPTIED`]. The sessions the bots
    /// had with Claude Code stay behind: another computer does not have them.
    pub fn snapshot_light_to(&self, path: &Path) -> Result<()> {
        self.snapshot_to(path)?;
        let copy = Connection::open(path)?;
        copy.pragma_update(None, "foreign_keys", "OFF")?;
        let tx = copy.unchecked_transaction()?;
        for table in EMPTIED {
            tx.execute(&format!("DELETE FROM {table}"), [])?;
        }
        tx.execute("UPDATE bots SET session_id = NULL, token_hash = NULL", [])?;
        tx.commit()?;
        copy.execute_batch("VACUUM")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(conn: &Connection, table: &str) -> i64 {
        conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
            row.get(0)
        })
        .expect("count")
    }

    #[test]
    fn the_light_copy_keeps_the_structure_and_drops_the_conversations() {
        let dir = tempfile::tempdir().expect("dir");
        let store = Store::open(&dir.path().join("a.db")).expect("store");
        store
            .conn
            .execute_batch(
                "INSERT INTO crews (id, name, slug, created_at) VALUES ('crw_1', 'Ops', 'ops', 1);
                 INSERT INTO bots (id, crew_id, name, handle, slug, role, instructions, color, \
                  token_hash, session_id, created_at)
                  VALUES ('bot_1', 'crw_1', 'Scout', 'scout', 'scout', 'r', 'i', 'c', 'h', 's', 1);
                 INSERT INTO settings (key, value) VALUES ('k', 'v');
                 INSERT INTO turn_costs (bot_id, at, cost) VALUES ('bot_1', 1, 0.5);",
            )
            .expect("rows");
        let light = dir.path().join("light.db");
        store.snapshot_light_to(&light).expect("light");

        let copy = Connection::open(&light).expect("open");
        assert_eq!(count(&copy, "crews"), 1);
        assert_eq!(count(&copy, "bots"), 1);
        assert_eq!(count(&copy, "settings"), 1);
        assert_eq!(count(&copy, "turn_costs"), 0);
        let (session, token): (Option<String>, Option<String>) = copy
            .query_row("SELECT session_id, token_hash FROM bots", [], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .expect("bot");
        assert_eq!((session, token), (None, None));
        // The original keeps everything.
        assert_eq!(count(&store.conn, "turn_costs"), 1);
    }
}
