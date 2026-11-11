//! `bot_versions` table (spec 8.10): the history of each bot's memory and
//! instructions. Versions go by `rowid`, the order they were saved in, since
//! two ids made in the same millisecond do not always sort in that order.

use botloft_core::ids::{BotId, VersionId};
use botloft_core::protocol::{TextVersion, VersionKind};
use rusqlite::{OptionalExtension, Row, params};

use crate::messages::optional_column;
use crate::{Result, Store, parse_column};

/// The most versions kept of one bot's memory or instructions; older ones go.
pub const VERSIONS_KEPT: usize = 100;

const COLUMNS: &str = "id, bot_id, kind, author, text, restored_from, created_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<TextVersion> {
    Ok(TextVersion {
        id: parse_column(row, 0)?,
        bot_id: parse_column(row, 1)?,
        kind: parse_column(row, 2)?,
        author: parse_column(row, 3)?,
        text: row.get(4)?,
        restored_from: optional_column(row, 5)?,
        created_at: row.get(6)?,
    })
}

impl Store {
    /// Saves a version and drops the oldest beyond [`VERSIONS_KEPT`].
    pub fn insert_version(&self, version: &TextVersion) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            &format!("INSERT INTO bot_versions ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"),
            params![
                version.id.as_str(),
                version.bot_id.as_str(),
                version.kind.as_str(),
                version.author.as_str(),
                version.text,
                version.restored_from.as_ref().map(VersionId::as_str),
                version.created_at,
            ],
        )?;
        tx.execute(
            "DELETE FROM bot_versions WHERE bot_id = ?1 AND kind = ?2 AND rowid NOT IN \
             (SELECT rowid FROM bot_versions WHERE bot_id = ?1 AND kind = ?2 \
              ORDER BY rowid DESC LIMIT ?3)",
            params![
                version.bot_id.as_str(),
                version.kind.as_str(),
                i64::try_from(VERSIONS_KEPT).unwrap_or(i64::MAX),
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    /// The bot's versions of `kind`, newest first.
    pub fn versions(
        &self,
        bot: &BotId,
        kind: VersionKind,
        limit: usize,
    ) -> Result<Vec<TextVersion>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM bot_versions WHERE bot_id = ?1 AND kind = ?2 \
             ORDER BY rowid DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(
            params![
                bot.as_str(),
                kind.as_str(),
                i64::try_from(limit).unwrap_or(i64::MAX)
            ],
            from_row,
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The bot's newest version of `kind`, if there is one.
    pub fn latest_version(&self, bot: &BotId, kind: VersionKind) -> Result<Option<TextVersion>> {
        Ok(self.versions(bot, kind, 1)?.pop())
    }

    pub fn version(&self, id: &VersionId) -> Result<Option<TextVersion>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM bot_versions WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::VersionAuthor;

    use super::*;
    use crate::tests::Fixture;

    fn version(bot: &BotId, kind: VersionKind, text: &str, at: i64) -> TextVersion {
        TextVersion {
            id: VersionId::generate(),
            bot_id: bot.clone(),
            kind,
            author: VersionAuthor::Agent,
            text: text.into(),
            restored_from: None,
            created_at: at,
        }
    }

    #[test]
    fn versions_come_back_newest_first_and_apart_by_kind() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        for n in 0..3 {
            let saved = version(bot, VersionKind::Memory, &format!("memory {n}"), n);
            fx.store.insert_version(&saved).expect("insert");
        }
        let rules = version(bot, VersionKind::Instructions, "Be brief.", 9);
        fx.store.insert_version(&rules).expect("insert");

        let memory = fx
            .store
            .versions(bot, VersionKind::Memory, 10)
            .expect("list");
        let texts: Vec<_> = memory.iter().map(|v| v.text.as_str()).collect();
        assert_eq!(texts, ["memory 2", "memory 1", "memory 0"]);
        assert_eq!(
            fx.store
                .latest_version(bot, VersionKind::Instructions)
                .expect("latest")
                .map(|v| v.text),
            Some("Be brief.".to_owned())
        );
        assert_eq!(
            fx.store.version(&rules.id).expect("read").map(|v| v.kind),
            Some(VersionKind::Instructions)
        );
    }

    #[test]
    fn only_the_newest_hundred_stay() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        for n in 0..VERSIONS_KEPT + 5 {
            let at = i64::try_from(n).expect("small");
            let saved = version(bot, VersionKind::Memory, &format!("v{n}"), at);
            fx.store.insert_version(&saved).expect("insert");
        }
        let kept = fx
            .store
            .versions(bot, VersionKind::Memory, 500)
            .expect("list");
        assert_eq!(kept.len(), VERSIONS_KEPT);
        assert_eq!(kept[0].text, format!("v{}", VERSIONS_KEPT + 4));
        assert_eq!(kept[kept.len() - 1].text, "v5");
    }
}
