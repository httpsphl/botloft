//! Schema migrations. Each file in `migrations/` runs once, in order, inside
//! a transaction that also bumps `PRAGMA user_version`.

use rusqlite::Connection;

use crate::{Result, StoreError};

/// `(version, sql)`; versions are consecutive and match the file prefix.
const MIGRATIONS: &[(u32, &str)] = &[
    (1, include_str!("../migrations/0001_init.sql")),
    (2, include_str!("../migrations/0002_messages.sql")),
    (3, include_str!("../migrations/0003_chat.sql")),
    (4, include_str!("../migrations/0004_permission_mode.sql")),
    (5, include_str!("../migrations/0005_bot_model.sql")),
    (6, include_str!("../migrations/0006_crew_work_folder.sql")),
    (7, include_str!("../migrations/0007_crew_lead.sql")),
    (8, include_str!("../migrations/0008_routines.sql")),
    (9, include_str!("../migrations/0009_browser_sites.sql")),
    (10, include_str!("../migrations/0010_bot_effort.sql")),
    (11, include_str!("../migrations/0011_turn_tokens.sql")),
    (12, include_str!("../migrations/0012_perf_indexes.sql")),
    (13, include_str!("../migrations/0013_reply_index.sql")),
    (14, include_str!("../migrations/0014_allow_rules.sql")),
    (15, include_str!("../migrations/0015_questions.sql")),
];

/// Schema version after every migration has run.
pub const LATEST_VERSION: u32 = MIGRATIONS.len() as u32;

pub(crate) fn run(conn: &mut Connection) -> Result<()> {
    let current: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if current > LATEST_VERSION {
        return Err(StoreError::TooNew {
            found: current,
            supported: LATEST_VERSION,
        });
    }
    for &(version, sql) in MIGRATIONS.iter().filter(|(v, _)| *v > current) {
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
    fn versions_are_consecutive_from_one() {
        for (i, (version, _)) in MIGRATIONS.iter().enumerate() {
            assert_eq!(*version as usize, i + 1);
        }
    }

    #[test]
    fn migrating_twice_is_a_no_op() {
        let mut conn = Connection::open_in_memory().expect("open");
        run(&mut conn).expect("first run");
        run(&mut conn).expect("second run");
        let version: u32 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .expect("version");
        assert_eq!(version, LATEST_VERSION);
    }

    #[test]
    fn refuses_a_database_from_a_newer_build() {
        let mut conn = Connection::open_in_memory().expect("open");
        conn.pragma_update(None, "user_version", LATEST_VERSION + 1)
            .expect("set version");
        assert!(matches!(run(&mut conn), Err(StoreError::TooNew { .. })));
    }
}
