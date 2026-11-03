//! `imports` table: the summaries of the owner's history made for the Maker
//! (spec 36.7). Only what a summary covered and the counts of what was
//! swapped out; never its text.

use std::collections::BTreeMap;

use botloft_core::ids::{ImportId, MessageId};
use botloft_core::protocol::{Import, ImportSource};
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

const COLUMNS: &str = "id, source, created_at, sent_at, sent_message_id, scope, chars, redactions";

fn source_of(raw: &str) -> ImportSource {
    match raw {
        "claude_ai" => ImportSource::ClaudeAi,
        _ => ImportSource::ClaudeCode,
    }
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<Import> {
    let source: String = row.get(1)?;
    let scope: String = row.get(5)?;
    let redactions: String = row.get(7)?;
    Ok(Import {
        id: parse_column(row, 0)?,
        source: source_of(&source),
        created_at: row.get(2)?,
        sent_at: row.get(3)?,
        sent_message_id: row
            .get::<_, Option<String>>(4)?
            .and_then(|id| id.parse::<MessageId>().ok()),
        scope: serde_json::from_str(&scope).unwrap_or_default(),
        chars: row.get(6)?,
        redactions: serde_json::from_str::<BTreeMap<String, u32>>(&redactions).unwrap_or_default(),
    })
}

impl Store {
    pub fn insert_import(&self, import: &Import) -> Result<()> {
        self.conn.execute(
            &format!("INSERT INTO imports ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"),
            params![
                import.id.as_str(),
                import.source.as_str(),
                import.created_at,
                import.sent_at,
                import.sent_message_id.as_ref().map(MessageId::as_str),
                serde_json::Value::from(import.scope.clone()).to_string(),
                import.chars,
                serde_json::to_string(&import.redactions).unwrap_or_else(|_| "{}".to_owned()),
            ],
        )?;
        Ok(())
    }

    pub fn import(&self, id: &ImportId) -> Result<Option<Import>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM imports WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Every summary, newest first, at most `limit`.
    pub fn imports(&self, limit: u32) -> Result<Vec<Import>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM imports ORDER BY created_at DESC, id DESC LIMIT ?1"
        ))?;
        let rows = stmt.query_map([limit], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Marks the summary sent with `message`; false when it already was.
    pub fn mark_import_sent(&self, id: &ImportId, at: i64, message: &MessageId) -> Result<bool> {
        let changed = self.conn.execute(
            "UPDATE imports SET sent_at = ?2, sent_message_id = ?3 \
             WHERE id = ?1 AND sent_at IS NULL",
            params![id.as_str(), at, message.as_str()],
        )?;
        Ok(changed == 1)
    }

    /// Forgets a summary that was never sent; false when there is none.
    pub fn delete_unsent_import(&self, id: &ImportId) -> Result<bool> {
        let changed = self.conn.execute(
            "DELETE FROM imports WHERE id = ?1 AND sent_at IS NULL",
            [id.as_str()],
        )?;
        Ok(changed == 1)
    }

    /// The summaries made before `before` and never sent.
    pub fn stale_imports(&self, before: i64) -> Result<Vec<ImportId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id FROM imports WHERE sent_at IS NULL AND created_at < ?1")?;
        let rows = stmt.query_map([before], |row| parse_column(row, 0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn import(created_at: i64) -> Import {
        Import {
            id: ImportId::generate(),
            source: ImportSource::ClaudeCode,
            created_at,
            sent_at: None,
            sent_message_id: None,
            scope: vec!["C:/work/blog".to_owned()],
            chars: 1200,
            redactions: BTreeMap::from([("key".to_owned(), 2), ("email".to_owned(), 1)]),
        }
    }

    #[test]
    fn keeps_what_a_summary_covered_and_sends_it_once() {
        let store = Store::open_in_memory().expect("store");
        let first = import(10);
        store.insert_import(&first).expect("insert");
        assert_eq!(store.import(&first.id).expect("get"), Some(first.clone()));

        let message = MessageId::generate();
        assert!(
            store
                .mark_import_sent(&first.id, 20, &message)
                .expect("send")
        );
        assert!(
            !store
                .mark_import_sent(&first.id, 30, &message)
                .expect("again")
        );
        let sent = store.import(&first.id).expect("get").expect("import");
        assert_eq!(
            (sent.sent_at, sent.sent_message_id),
            (Some(20), Some(message))
        );
        // A sent summary is the Maker's conversation now: it is not forgotten here.
        assert!(!store.delete_unsent_import(&first.id).expect("forget"));
    }

    #[test]
    fn finds_old_unsent_summaries_and_forgets_them() {
        let store = Store::open_in_memory().expect("store");
        let old = import(10);
        let new = import(500);
        store.insert_import(&old).expect("insert");
        store.insert_import(&new).expect("insert");
        assert_eq!(
            store.stale_imports(100).expect("stale"),
            vec![old.id.clone()]
        );
        assert!(store.delete_unsent_import(&old.id).expect("forget"));
        assert_eq!(store.imports(10).expect("list"), vec![new]);
    }
}
