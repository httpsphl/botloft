//! The server's own SQLite database: one connection behind a mutex, queries
//! short, migrations numbered in `migrations/`.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rusqlite::Connection;

use crate::error::ApiError;

/// `(version, sql)`; versions are consecutive and match the file prefix.
const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("../migrations/0001_accounts.sql")),
    (2, include_str!("../migrations/0002_copies.sql")),
    (3, include_str!("../migrations/0003_mobile.sql")),
    (4, include_str!("../migrations/0004_push.sql")),
];

#[derive(Clone)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    pub fn open(path: &Path) -> rusqlite::Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.busy_timeout(Duration::from_secs(5))?;
        Self::ready(conn)
    }

    pub fn memory() -> rusqlite::Result<Self> {
        Self::ready(Connection::open_in_memory()?)
    }

    fn ready(mut conn: Connection) -> rusqlite::Result<Self> {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        migrate(&mut conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// A consistent copy of the database at `path`, which must not exist yet
    /// (`botloft-cloud backup`): safe while the server runs.
    pub fn snapshot_to(&self, path: &Path) -> rusqlite::Result<()> {
        let conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        conn.execute("VACUUM INTO ?1", [path.to_string_lossy().as_ref()])?;
        Ok(())
    }

    /// Runs `work` with the connection. Keep it short: the lock is shared.
    pub(crate) fn run<T>(
        &self,
        work: impl FnOnce(&mut Connection) -> rusqlite::Result<T>,
    ) -> Result<T, ApiError> {
        let mut conn = self
            .conn
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        work(&mut conn).map_err(ApiError::from)
    }
}

fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (version, sql) in MIGRATIONS.iter().filter(|(version, _)| *version > current) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_snapshot_is_a_database_of_its_own() {
        let dir = tempfile::tempdir().expect("dir");
        let db = Db::memory().expect("db");
        db.run(|conn| {
            conn.execute(
                "INSERT INTO accounts (email, created_at) VALUES ('a@b.c', 1)",
                [],
            )
        })
        .expect("row");
        let copy = dir.path().join("copy.db");
        db.snapshot_to(&copy).expect("snapshot");
        let back = Db::open(&copy).expect("open the copy");
        let email: String = back
            .run(|conn| conn.query_row("SELECT email FROM accounts", [], |row| row.get(0)))
            .expect("email");
        assert_eq!(email, "a@b.c");
        // It will not overwrite a copy that is there.
        assert!(db.snapshot_to(&copy).is_err());
    }

    #[test]
    fn a_new_database_has_every_table() {
        let db = Db::memory().expect("db");
        let tables: Vec<String> = db
            .run(|conn| {
                let mut stmt = conn
                    .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")?;
                stmt.query_map([], |row| row.get(0))?.collect()
            })
            .expect("tables");
        for name in [
            "accounts",
            "attempts",
            "copies",
            "deletions",
            "devices",
            "logins",
            "pairings",
            "push_subscriptions",
            "relay_queue",
        ] {
            assert!(tables.iter().any(|table| table == name), "{name}");
        }
    }
}
