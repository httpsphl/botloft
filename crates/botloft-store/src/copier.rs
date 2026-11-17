//! Copies of the database for the backups (spec 14.2, 27.10). A copy reads
//! the whole database: made on the daemon's store, it held every other
//! caller (the bots' output, the tools, the app) for as long as it took. A
//! [`Copier`] reads on a connection of its own instead; in WAL it sees one
//! fixed moment while the store goes on writing.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OpenFlags};

use crate::{Result, Store};

/// Reads the database at a path for a copy, without the daemon's store.
pub struct Copier {
    conn: Connection,
}

impl Copier {
    pub fn open(db: &Path) -> Result<Self> {
        let conn = Connection::open_with_flags(
            db,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        conn.busy_timeout(Duration::from_millis(5000))?;
        Ok(Self { conn })
    }

    /// [`Store::snapshot_to`], on this connection.
    pub fn snapshot_to(&self, path: &Path) -> Result<()> {
        vacuum_into(&self.conn, path)
    }

    /// [`Store::snapshot_light_to`], on this connection.
    pub fn snapshot_light_to(&self, path: &Path) -> Result<()> {
        crate::light::light_copy(&self.conn, path)
    }

    /// [`Store::light_fingerprint`], on this connection.
    pub fn light_fingerprint(&self, scratch: &Path) -> Result<String> {
        crate::fingerprint::fingerprint_of(&self.conn, scratch)
    }
}

impl Store {
    /// The file the database lives in, for a [`Copier`]; `None` in memory.
    pub fn file(&self) -> Option<PathBuf> {
        self.conn
            .path()
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
    }
}

/// A consistent copy of everything `conn` reads at `path`, which must not
/// exist yet.
pub(crate) fn vacuum_into(conn: &Connection, path: &Path) -> Result<()> {
    conn.execute("VACUUM INTO ?1", [path.to_string_lossy().as_ref()])?;
    Ok(())
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
    fn a_copy_reads_on_its_own_while_the_store_writes() {
        let dir = tempfile::tempdir().expect("dir");
        let db = dir.path().join("botloft.db");
        let store = Store::open(&db).expect("store");
        // SQLite gives the real path: on macOS the temporary folder under
        // `/var` is `/private/var`.
        let real = |path: &Path| path.canonicalize().expect("real path");
        assert_eq!(store.file().as_deref().map(real), Some(real(&db)));
        assert_eq!(Store::open_in_memory().expect("memory").file(), None);
        store
            .conn
            .execute_batch(
                "INSERT INTO crews (id, name, slug, created_at) VALUES ('crw_1', 'Ops', 'ops', 1)",
            )
            .expect("crew");

        // The store is in the middle of a write that is not committed yet.
        store
            .conn
            .execute_batch(
                "BEGIN IMMEDIATE;
                 INSERT INTO crews (id, name, slug, created_at) VALUES ('crw_2', 'Dev', 'dev', 2);",
            )
            .expect("writing");
        let copier = Copier::open(&db).expect("copier");
        let whole = dir.path().join("whole.db");
        copier
            .snapshot_to(&whole)
            .expect("copy while the store writes");
        let light = dir.path().join("light.db");
        copier.snapshot_light_to(&light).expect("light copy");
        let print = copier
            .light_fingerprint(&dir.path().join("scratch.db"))
            .expect("fingerprint");
        store.conn.execute_batch("COMMIT").expect("commit");

        for copy in [&whole, &light] {
            let copy = Connection::open(copy).expect("open");
            assert_eq!(count(&copy, "crews"), 1, "the copy is the committed moment");
        }
        assert_eq!(count(&store.conn, "crews"), 2);
        let now = store
            .light_fingerprint(&dir.path().join("scratch2.db"))
            .expect("fingerprint now");
        assert_ne!(print, now, "the fingerprint was of the committed moment");
    }
}
