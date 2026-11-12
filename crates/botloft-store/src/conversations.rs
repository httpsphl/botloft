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

    /// Names conversation `id`; `None` takes its name away.
    pub fn set_conversation_title(
        &self,
        id: &ConversationId,
        title: Option<&str>,
        by: Option<&str>,
    ) -> Result<()> {
        cached_execute(
            &self.conn,
            "UPDATE conversations SET title = ?2, title_by = ?3 WHERE id = ?1",
            params![id.as_str(), title, by],
        )?;
        Ok(())
    }

    /// Archives conversation `id` at `at`, or brings it back with `None`.
    pub fn set_conversation_archived(&self, id: &ConversationId, at: Option<i64>) -> Result<()> {
        cached_execute(
            &self.conn,
            "UPDATE conversations SET archived_at = ?2 WHERE id = ?1",
            params![id.as_str(), at],
        )?;
        Ok(())
    }

    /// The archived conversations of `bot`, the last archived first.
    pub fn archived_conversations(&self, bot: &BotId) -> Result<Vec<ConversationRecord>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM conversations \
             WHERE archived_at IS NOT NULL AND bot_id = ?1 \
             ORDER BY archived_at DESC, id"
        ))?;
        let rows = statement.query_map([bot.as_str()], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// When the newest chat item of conversation `id` was made; `None`
    /// while it has none.
    pub fn conversation_activity(&self, id: &ConversationId) -> Result<Option<i64>> {
        Ok(cached_row(
            &self.conn,
            "SELECT MAX(created_at) FROM chat_items WHERE conversation_id = ?1",
            [id.as_str()],
            |row| row.get(0),
        )?)
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
mod tests;
