//! `builds` table: what the Maker built in one go, so the owner can undo it
//! (spec 36.4). The crews it made are kept as a JSON list of ids.

use botloft_core::ids::{BotId, BuildId, CrewId};
use botloft_core::protocol::{BUILD_UNDO_DAYS, Build};
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

const COLUMNS: &str = "id, bot_id, approval_id, title, crew_ids, applied_at, undone_at";
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

fn from_row(row: &Row<'_>) -> rusqlite::Result<Build> {
    let crews: String = row.get(4)?;
    let applied_at: i64 = row.get(5)?;
    Ok(Build {
        id: parse_column(row, 0)?,
        bot_id: parse_column(row, 1)?,
        approval_id: row
            .get::<_, Option<String>>(2)?
            .and_then(|id| id.parse().ok()),
        title: row.get(3)?,
        crew_ids: serde_json::from_str::<Vec<String>>(&crews)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|id| id.parse::<CrewId>().ok())
            .collect(),
        applied_at,
        undo_until: applied_at + BUILD_UNDO_DAYS * DAY_MS,
        undone_at: row.get(6)?,
    })
}

impl Store {
    pub fn insert_build(&self, build: &Build) -> Result<()> {
        let crews: Vec<&str> = build.crew_ids.iter().map(CrewId::as_str).collect();
        self.conn.execute(
            &format!("INSERT INTO builds ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"),
            params![
                build.id.as_str(),
                build.bot_id.as_str(),
                build.approval_id.as_ref().map(|id| id.as_str()),
                build.title,
                serde_json::Value::from(crews).to_string(),
                build.applied_at,
                build.undone_at,
            ],
        )?;
        Ok(())
    }

    pub fn build(&self, id: &BuildId) -> Result<Option<Build>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM builds WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// The builds of `bot`, newest first, at most `limit`.
    pub fn builds(&self, bot: &BotId, limit: u32) -> Result<Vec<Build>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM builds WHERE bot_id = ?1 \
             ORDER BY applied_at DESC, id DESC LIMIT ?2"
        ))?;
        let rows = stmt.query_map(params![bot.as_str(), limit], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Marks the build undone; false when it already was.
    pub fn undo_build(&self, id: &BuildId, at: i64) -> Result<bool> {
        let changed = self.conn.execute(
            "UPDATE builds SET undone_at = ?2 WHERE id = ?1 AND undone_at IS NULL",
            params![id.as_str(), at],
        )?;
        Ok(changed == 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saves_lists_and_undoes_once() {
        let store = Store::open_in_memory().expect("store");
        let bot = BotId::generate();
        let build = Build {
            id: BuildId::generate(),
            bot_id: bot.clone(),
            approval_id: None,
            title: "Blog".to_owned(),
            crew_ids: vec![CrewId::generate(), CrewId::generate()],
            applied_at: 10,
            undo_until: 10 + BUILD_UNDO_DAYS * DAY_MS,
            undone_at: None,
        };
        store.insert_build(&build).expect("insert");
        assert_eq!(store.build(&build.id).expect("get"), Some(build.clone()));
        assert_eq!(
            store.builds(&bot, 5).expect("list"),
            std::slice::from_ref(&build)
        );
        assert!(
            store
                .builds(&BotId::generate(), 5)
                .expect("list")
                .is_empty()
        );

        assert!(store.undo_build(&build.id, 20).expect("undo"));
        assert!(!store.undo_build(&build.id, 30).expect("again"));
        let undone = store.build(&build.id).expect("get").expect("build");
        assert_eq!(undone.undone_at, Some(20));
    }
}
