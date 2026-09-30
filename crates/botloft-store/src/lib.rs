//! SQLite storage for the Botloft daemon: connection setup, numbered
//! migrations (`migrations/NNNN_name.sql`) and repositories.
//!
//! One [`Store`] wraps one connection. The daemon keeps a single store behind
//! a mutex; queries are short, so there is no connection pool.

mod approvals;
mod bots;
mod browser_sites;
mod chat;
mod crews;
mod delete;
mod deliveries;
mod messages;
mod migrate;
#[cfg(test)]
mod plans;
mod routine_runs;
mod routines;
mod tasks;
mod usage;

use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use rusqlite::types::Type;
use rusqlite::{Connection, Row, ffi};

pub use approvals::ApprovalRecord;
pub use bots::BotRecord;
pub use deliveries::DeliveryOutcome;
pub use messages::MessageFilter;
pub use migrate::LATEST_VERSION;
pub use tasks::TaskFilter;

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
        // In WAL mode NORMAL still never corrupts the database; a power cut
        // can lose the last commits, but no commit waits for the disk.
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.pragma_update(None, "temp_store", "MEMORY")?;
        conn.pragma_update(None, "cache_size", -16_000)?;
        conn.pragma_update(None, "journal_size_limit", 64 << 20)?;
        conn.busy_timeout(Duration::from_millis(5000))?;
        conn.set_prepared_statement_cache_capacity(128);
        migrate::run(&mut conn)?;
        // Statistics for the planner, bounded so opening stays fast.
        conn.execute_batch("PRAGMA optimize = 0x10002")?;
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

/// `query_row` through the statement cache, for statements that run on every
/// chat item, delivery or message.
fn cached_row<T, P, F>(conn: &Connection, sql: &str, params: P, f: F) -> rusqlite::Result<T>
where
    P: rusqlite::Params,
    F: FnOnce(&Row<'_>) -> rusqlite::Result<T>,
{
    conn.prepare_cached(sql)?.query_row(params, f)
}

/// `execute` through the statement cache; see [`cached_row`].
fn cached_execute<P: rusqlite::Params>(
    conn: &Connection,
    sql: &str,
    params: P,
) -> rusqlite::Result<usize> {
    conn.prepare_cached(sql)?.execute(params)
}

/// Generations and sizes are `u64` in the protocol; SQLite stores `i64`.
/// Neither comes near `i64::MAX`.
fn to_sql_int(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
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
pub(crate) mod tests {
    use botloft_core::ids::{BotId, CrewId, DeliveryId, MessageId};
    use botloft_core::protocol::{Crew, Delivery, DeliveryState, Message, MessageKind, SenderKind};

    use super::*;

    /// A store with one crew and two bots.
    pub(crate) struct Fixture {
        pub store: Store,
        pub crew: Crew,
        pub bots: Vec<BotRecord>,
    }

    impl Fixture {
        pub fn new() -> Self {
            let store = Store::open_in_memory().expect("store");
            let crew = Crew {
                id: CrewId::generate(),
                name: "Ops".to_owned(),
                slug: "ops".to_owned(),
                work_folder: String::new(),
                work_folder_chosen: false,
                lead_bot_id: None,
                paused: false,
                created_at: 0,
                archived_at: None,
            };
            store.insert_crew(&crew).expect("crew");
            let bots = ["scout", "writer"]
                .iter()
                .map(|handle| {
                    let bot = BotRecord {
                        id: BotId::generate(),
                        crew_id: crew.id.clone(),
                        name: (*handle).to_owned(),
                        handle: (*handle).to_owned(),
                        slug: (*handle).to_owned(),
                        role: String::new(),
                        instructions: String::new(),
                        color: "#FF7A59".to_owned(),
                        paused: false,
                        permission_mode: botloft_core::protocol::PermissionMode::Default,
                        model: botloft_core::protocol::BotModel::Default,
                        model_in_use: None,
                        effort: botloft_core::protocol::BotEffort::Default,
                        effort_default: None,
                        created_at: 0,
                        archived_at: None,
                    };
                    store.insert_bot(&bot).expect("bot");
                    bot
                })
                .collect();
            Self { store, crew, bots }
        }
    }

    /// An owner note to `bot` and its pending delivery, not yet saved.
    pub(crate) fn message_to(crew: &CrewId, bot: &BotId, body: &str) -> (Message, Delivery) {
        let message = Message {
            id: MessageId::generate(),
            crew_id: crew.clone(),
            from_kind: SenderKind::Owner,
            from_bot_id: None,
            to_bot_id: bot.clone(),
            kind: MessageKind::Note,
            body: body.to_owned(),
            task_id: None,
            routine_id: None,
            attachments: Vec::new(),
            created_at: 0,
        };
        let delivery = Delivery {
            id: DeliveryId::generate(),
            message_id: message.id.clone(),
            bot_id: bot.clone(),
            state: DeliveryState::Pending,
            attempts: 0,
            next_attempt_at: 0,
            last_error: None,
            read_at: None,
            updated_at: 0,
        };
        (message, delivery)
    }

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
        let synchronous: i64 = store
            .conn
            .pragma_query_value(None, "synchronous", |row| row.get(0))
            .expect("synchronous");
        assert_eq!(mode, "wal");
        assert_eq!(fk, 1);
        assert_eq!(synchronous, 1, "NORMAL");
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
