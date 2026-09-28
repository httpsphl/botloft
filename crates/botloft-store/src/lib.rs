//! SQLite storage for the Botloft daemon: connection setup, numbered
//! migrations (`migrations/NNNN_name.sql`) and repositories.
//!
//! One [`Store`] wraps one connection. The daemon keeps a single store behind
//! a mutex; queries are short, so there is no connection pool.

mod bots;
mod crews;
mod migrate;

use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use rusqlite::types::Type;
use rusqlite::{Connection, Row, ffi};

pub use bots::BotRecord;
pub use migrate::LATEST_VERSION;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("a row with the same {0} already exists")]
    Duplicate(&'static str),
    #[error("database schema version {found} is newer than this build supports ({supported})")]
    TooNew { found: u32, supported: u32 },
}

pub type Result<T, E = StoreError> = std::result::Result<T, E>;

pub struct Store {
    conn: Connection,
}

impl Store {
    /// Opens (or creates) the database file and brings its schema up to date.
    pub fn open(path: &Path) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    /// A private in-memory database, for tests.
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.busy_timeout(Duration::from_millis(5000))?;
        migrate::run(&mut conn)?;
        Ok(Self { conn })
    }

    /// Schema version recorded in `PRAGMA user_version`.
    pub fn schema_version(&self) -> Result<u32> {
        Ok(self
            .conn
            .pragma_query_value(None, "user_version", |row| row.get(0))?)
    }
}

/// Reads a text column into a parsed type such as an ID.
fn parse_column<T>(row: &Row<'_>, idx: usize) -> rusqlite::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let raw: String = row.get(idx)?;
    raw.parse()
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(idx, Type::Text, Box::new(err)))
}

/// Turns a UNIQUE or PRIMARY KEY failure into [`StoreError::Duplicate`];
/// other constraint failures (a missing foreign key, say) stay as they are.
fn unique_as_duplicate(err: rusqlite::Error, what: &'static str) -> StoreError {
    match &err {
        rusqlite::Error::SqliteFailure(e, _)
            if e.extended_code == ffi::SQLITE_CONSTRAINT_UNIQUE
                || e.extended_code == ffi::SQLITE_CONSTRAINT_PRIMARYKEY =>
        {
            StoreError::Duplicate(what)
        }
        _ => StoreError::Sqlite(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_database_uses_wal_and_foreign_keys() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(&dir.path().join("botloft.db")).expect("open");
        let mode: String = store
            .conn
            .pragma_query_value(None, "journal_mode", |row| row.get(0))
            .expect("journal_mode");
        let fk: i64 = store
            .conn
            .pragma_query_value(None, "foreign_keys", |row| row.get(0))
            .expect("foreign_keys");
        assert_eq!(mode, "wal");
        assert_eq!(fk, 1);
        assert_eq!(store.schema_version().expect("version"), LATEST_VERSION);
    }

    #[test]
    fn reopening_keeps_data_and_does_not_rerun_migrations() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("botloft.db");
        {
            let store = Store::open(&path).expect("open");
            store
                .conn
                .execute("INSERT INTO settings (key, value) VALUES ('k', 'v')", [])
                .expect("insert");
        }
        let store = Store::open(&path).expect("reopen");
        let value: String = store
            .conn
            .query_row("SELECT value FROM settings WHERE key = 'k'", [], |r| {
                r.get(0)
            })
            .expect("select");
        assert_eq!(value, "v");
    }
}
