//! `conversations` table: the conversations of each bot (spec 40). Every
//! bot has a main one, `ConversationId::main_of` its id, made with the bot.

use botloft_core::ids::{BotId, ConversationId};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{Result, Store, cached_execute, cached_row, parse_column};

const COLUMNS: &str = "id, bot_id, main, title, title_by, session_id, created_at, archived_at";

/// One conversation as stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationRecord {
    pub id: ConversationId,
    pub bot_id: BotId,
    pub main: bool,
    pub title: Option<String>,
    /// `owner` or `ai`; `None` while it has no title.
    pub title_by: Option<String>,
    /// The conversation the agent has on disk (spec 7.3).
    pub session_id: Option<String>,
    pub created_at: i64,
    pub archived_at: Option<i64>,
}

impl ConversationRecord {
    /// A new conversation of `bot` the owner opened, with nothing in it.
    pub fn new(bot: &BotId, created_at: i64) -> Self {
        Self {
            id: ConversationId::generate(),
            bot_id: bot.clone(),
            main: false,
            title: None,
            title_by: None,
            session_id: None,
            created_at,
            archived_at: None,
        }
    }
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<ConversationRecord> {
    Ok(ConversationRecord {
        id: parse_column(row, 0)?,
        bot_id: parse_column(row, 1)?,
        main: row.get(2)?,
        title: row.get(3)?,
        title_by: row.get(4)?,
        session_id: row.get(5)?,
        created_at: row.get(6)?,
        archived_at: row.get(7)?,
    })
}

impl Store {
    /// Makes the main conversation of `bot` when it has none.
    pub(crate) fn ensure_main_in(conn: &Connection, bot: &BotId, created_at: i64) -> Result<()> {
        cached_execute(
            conn,
            "INSERT OR IGNORE INTO conversations (id, bot_id, main, created_at) \
             VALUES (?1, ?2, 1, ?3)",
            params![
                ConversationId::main_of(bot).as_str(),
                bot.as_str(),
                created_at
            ],
        )?;
        Ok(())
    }

    pub fn insert_conversation(&self, conversation: &ConversationRecord) -> Result<()> {
        cached_execute(
            &self.conn,
            &format!(
                "INSERT INTO conversations ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"
            ),
            params![
                conversation.id.as_str(),
                conversation.bot_id.as_str(),
                conversation.main,
                conversation.title,
                conversation.title_by,
                conversation.session_id,
                conversation.created_at,
                conversation.archived_at,
            ],
        )?;
        Ok(())
    }

    pub fn conversation(&self, id: &ConversationId) -> Result<Option<ConversationRecord>> {
        Ok(cached_row(
            &self.conn,
            &format!("SELECT {COLUMNS} FROM conversations WHERE id = ?1"),
            [id.as_str()],
            from_row,
        )
        .optional()?)
    }

    /// The conversations that are not archived, of `bot` or of every bot:
    /// main ones first, then the oldest first.
    pub fn conversations(&self, bot: Option<&BotId>) -> Result<Vec<ConversationRecord>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM conversations \
             WHERE archived_at IS NULL AND (?1 IS NULL OR bot_id = ?1) \
             ORDER BY main DESC, created_at, id"
        ))?;
        let rows = statement.query_map([bot.map(BotId::as_str)], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The session of the conversation `id`; `None` starts a new one.
    pub fn conversation_session(&self, id: &ConversationId) -> Result<Option<String>> {
        Ok(cached_row(
            &self.conn,
            "SELECT session_id FROM conversations WHERE id = ?1",
            [id.as_str()],
            |row| row.get(0),
        )
        .optional()?
        .flatten())
    }

    /// Sets the session of conversation `id` of `bot`; `None` makes its
    /// next start a new conversation. A main conversation that has no row
    /// yet gets one.
    pub fn set_conversation_session(
        &self,
        bot: &BotId,
        id: &ConversationId,
        session: Option<&str>,
    ) -> Result<()> {
        if id.is_main_of(bot) {
            Self::ensure_main_in(&self.conn, bot, 0)?;
        }
        cached_execute(
            &self.conn,
            "UPDATE conversations SET session_id = ?2 WHERE id = ?1",
            params![id.as_str(), session],
        )?;
        Ok(())
    }

    /// The session of the main conversation of `bot`.
    pub fn session_id(&self, bot: &BotId) -> Result<Option<String>> {
        self.conversation_session(&ConversationId::main_of(bot))
    }

    /// Sets the session of the main conversation of `bot`.
    pub fn set_session_id(&self, bot: &BotId, session: Option<&str>) -> Result<()> {
        self.set_conversation_session(bot, &ConversationId::main_of(bot), session)
    }
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
                  VALUES ('bot_01J9Z3K8000000000000000000', 'crw_1', 'Scout', 'scout', 'scout', \
                  'r', 'i', 'c', 1);",
            )
            .expect("rows");
        (dir, store)
    }

    #[test]
    fn the_main_conversation_holds_the_session_of_the_bot() {
        let (_dir, store) = store();
        let bot: BotId = "bot_01J9Z3K8000000000000000000".parse().unwrap();
        assert_eq!(store.session_id(&bot).unwrap(), None);
        store.set_session_id(&bot, Some("session-1")).unwrap();
        assert_eq!(
            store.session_id(&bot).unwrap().as_deref(),
            Some("session-1")
        );
        let main = store
            .conversation(&ConversationId::main_of(&bot))
            .unwrap()
            .expect("main");
        assert!(main.main);
        assert_eq!(main.bot_id, bot);
    }

    #[test]
    fn other_conversations_keep_their_own_session() {
        let (_dir, store) = store();
        let bot: BotId = "bot_01J9Z3K8000000000000000000".parse().unwrap();
        store.set_session_id(&bot, Some("main-session")).unwrap();
        let other = ConversationRecord::new(&bot, 5);
        store.insert_conversation(&other).unwrap();
        store
            .set_conversation_session(&bot, &other.id, Some("other-session"))
            .unwrap();
        assert_eq!(
            store.session_id(&bot).unwrap().as_deref(),
            Some("main-session")
        );
        assert_eq!(
            store.conversation_session(&other.id).unwrap().as_deref(),
            Some("other-session")
        );
        let listed: Vec<bool> = store
            .conversations(Some(&bot))
            .unwrap()
            .iter()
            .map(|each| each.main)
            .collect();
        assert_eq!(listed, [true, false]);
    }
}
