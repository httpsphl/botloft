//! What the light copy holds, as one hash (spec 27.10): the automatic backup
//! sends a copy only when this changed since the last one it sent.

use std::path::Path;

use rusqlite::Connection;
use rusqlite::types::ValueRef;
use sha2::{Digest, Sha256};

use crate::{Result, Store};

/// What changes by itself while nothing the owner made does: the model and
/// effort Claude Code reported, and when a routine runs next. Left in, the
/// copy would look new every few minutes. A column added later that also
/// moves by itself only costs a copy sent too often, never one missed.
const BY_THEMSELVES: &[&str] = &[
    "UPDATE bots SET model_in_use = NULL, effort_default = NULL",
    "UPDATE routines SET next_run_at = NULL",
];

/// The search index mirrors the chats, which the light copy leaves out.
fn skipped(table: &str) -> bool {
    table.starts_with("sqlite_") || table.starts_with("chat_search")
}

impl Store {
    /// A hash of the light copy of the database (14.2), hex. `scratch` is a
    /// file name that may not exist: the copy is made there and removed.
    pub fn light_fingerprint(&self, scratch: &Path) -> Result<String> {
        let _ = std::fs::remove_file(scratch);
        self.snapshot_light_to(scratch)?;
        let hash = hash_tables(scratch);
        let _ = std::fs::remove_file(scratch);
        hash
    }
}

fn hash_tables(path: &Path) -> Result<String> {
    let copy = Connection::open(path)?;
    for update in BY_THEMSELVES {
        copy.execute(update, [])?;
    }
    let tables: Vec<String> = copy
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?
        .query_map([], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    let mut hash = Sha256::new();
    for table in tables.iter().filter(|table| !skipped(table)) {
        hash.update(table.as_bytes());
        let columns = copy
            .prepare(&format!("SELECT * FROM \"{table}\" LIMIT 0"))?
            .column_count();
        let order = (1..=columns)
            .map(|column| column.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let mut rows = copy.prepare(&format!("SELECT * FROM \"{table}\" ORDER BY {order}"))?;
        let mut rows = rows.query([])?;
        while let Some(row) = rows.next()? {
            hash.update([0xFF]);
            for column in 0..columns {
                match row.get_ref(column)? {
                    ValueRef::Null => hash.update([0]),
                    ValueRef::Integer(value) => {
                        hash.update([1]);
                        hash.update(value.to_le_bytes());
                    }
                    ValueRef::Real(value) => {
                        hash.update([2]);
                        hash.update(value.to_le_bytes());
                    }
                    ValueRef::Text(bytes) | ValueRef::Blob(bytes) => {
                        hash.update([3]);
                        hash.update((bytes.len() as u64).to_le_bytes());
                        hash.update(bytes);
                    }
                }
            }
        }
    }
    Ok(hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("dir");
        let store = Store::open(&dir.path().join("a.db")).expect("store");
        store
            .conn
            .execute_batch(
                "INSERT INTO crews (id, name, slug, created_at) VALUES ('crw_1', 'Ops', 'ops', 1);
                 INSERT INTO bots (id, crew_id, name, handle, slug, role, instructions, color, \
                  created_at)
                  VALUES ('bot_1', 'crw_1', 'Scout', 'scout', 'scout', 'r', 'i', 'c', 1);
                 INSERT INTO routines (id, bot_id, name, prompt, schedule, timezone, overlap, \
                  missed, enabled, next_run_at, created_at, updated_at)
                  VALUES ('rtn_1', 'bot_1', 'Morning', 'p', '{}', 'UTC', 'skip', 'skip', 1, 10, 1, 1);",
            )
            .expect("rows");
        (dir, store)
    }

    fn print(dir: &tempfile::TempDir, store: &Store) -> String {
        store
            .light_fingerprint(&dir.path().join("scratch.db"))
            .expect("fingerprint")
    }

    #[test]
    fn what_changes_by_itself_does_not_change_the_fingerprint() {
        let (dir, store) = store();
        let first = print(&dir, &store);
        assert_eq!(first.len(), 64);
        assert_eq!(print(&dir, &store), first);
        store
            .conn
            .execute_batch(
                "UPDATE bots SET model_in_use = 'claude-opus-5-5', effort_default = 'medium';
                 UPDATE routines SET next_run_at = 99;
                 INSERT INTO turn_costs (bot_id, at, cost) VALUES ('bot_1', 1, 0.5);",
            )
            .expect("by themselves");
        assert_eq!(print(&dir, &store), first);
        // The scratch copy does not stay behind.
        assert!(!dir.path().join("scratch.db").exists());
    }

    #[test]
    fn what_the_owner_made_changes_it() {
        let (dir, store) = store();
        let first = print(&dir, &store);
        store
            .conn
            .execute(
                "UPDATE bots SET instructions = 'new' WHERE id = 'bot_1'",
                [],
            )
            .expect("edit");
        let edited = print(&dir, &store);
        assert_ne!(edited, first);
        store
            .conn
            .execute("UPDATE routines SET enabled = 0", [])
            .expect("routine");
        assert_ne!(print(&dir, &store), edited);
    }
}
