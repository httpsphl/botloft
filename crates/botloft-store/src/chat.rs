//! `chat_items` table: each bot's chat (spec 8). The body is stored as the
//! protocol's JSON, so new item kinds need no migration.

use botloft_core::chat::activity;
use botloft_core::ids::{BotId, ChatItemId};
use botloft_core::protocol::{Activity, ChatBody, ChatItem};
use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{Result, Store, cached_execute, cached_row, parse_column};

const COLUMNS: &str = "id, bot_id, data, created_at, updated_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<ChatItem> {
    let data: String = row.get(2)?;
    let body = serde_json::from_str(&data)
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(2, Type::Text, Box::new(err)))?;
    Ok(ChatItem {
        id: parse_column(row, 0)?,
        bot_id: parse_column(row, 1)?,
        body,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

/// The `kind` column: the body's tag.
fn kind(body: &ChatBody) -> &'static str {
    match body {
        ChatBody::Inbound(_) => "inbound",
        ChatBody::Reply(_) => "reply",
        ChatBody::Tool(_) => "tool",
        ChatBody::Approval(_) => "approval",
        ChatBody::Turn(_) => "turn",
        ChatBody::Notice(_) => "notice",
    }
}

fn json(body: &ChatBody) -> Result<String> {
    Ok(serde_json::to_string(body)
        .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?)
}

impl Store {
    pub fn insert_chat_item(&self, item: &ChatItem) -> Result<()> {
        Self::insert_chat_item_in(&self.conn, item)
    }

    pub(crate) fn insert_chat_item_in(conn: &Connection, item: &ChatItem) -> Result<()> {
        cached_execute(
            conn,
            "INSERT INTO chat_items (id, bot_id, kind, data, created_at, updated_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                item.id.as_str(),
                item.bot_id.as_str(),
                kind(&item.body),
                json(&item.body)?,
                item.created_at,
                item.updated_at,
            ],
        )?;
        Ok(())
    }

    /// Replaces an item's body. `None` if there is no such item.
    pub fn update_chat_item(
        &self,
        id: &ChatItemId,
        body: &ChatBody,
        now: i64,
    ) -> Result<Option<ChatItem>> {
        Ok(cached_row(
            &self.conn,
            &format!(
                "UPDATE chat_items SET kind = ?2, data = ?3, updated_at = ?4 \
                     WHERE id = ?1 RETURNING {COLUMNS}"
            ),
            params![id.as_str(), kind(body), json(body)?, now],
            from_row,
        )
        .optional()?)
    }

    pub fn chat_item(&self, id: &ChatItemId) -> Result<Option<ChatItem>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM chat_items WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Newest first, in the order items were stored; `before` pages back.
    pub fn chat_history(
        &self,
        bot: &BotId,
        before: Option<&ChatItemId>,
        limit: u32,
    ) -> Result<Vec<ChatItem>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM chat_items \
             WHERE bot_id = ?1 \
               AND (?2 IS NULL OR rowid < (SELECT rowid FROM chat_items WHERE id = ?2)) \
             ORDER BY rowid DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(
            params![bot.as_str(), before.map(ChatItemId::as_str), limit],
            from_row,
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The bot's newest tool item for `tool_use_id`, to update it when the
    /// tool returns.
    pub fn tool_item(&self, bot: &BotId, tool_use_id: &str) -> Result<Option<ChatItem>> {
        Ok(cached_row(
            &self.conn,
            &format!(
                "SELECT {COLUMNS} FROM chat_items \
                     WHERE bot_id = ?1 AND kind = 'tool' \
                       AND json_extract(data, '$.toolUseId') = ?2 \
                     ORDER BY rowid DESC LIMIT 1"
            ),
            params![bot.as_str(), tool_use_id],
            from_row,
        )
        .optional()?)
    }

    /// Paths of the files the bot's `Write`/`Edit` calls changed, newest
    /// first and without repeats; failed calls do not count.
    pub fn written_files(&self, bot: &BotId, limit: u32) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT json_extract(data, '$.file') FROM chat_items \
             WHERE bot_id = ?1 AND kind = 'tool' \
               AND json_extract(data, '$.file') IS NOT NULL \
               AND json_extract(data, '$.status') != 'failed' \
             GROUP BY json_extract(data, '$.file') \
             ORDER BY MAX(rowid) DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(params![bot.as_str(), limit], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The conversation-list line: the newest item that has one.
    pub fn last_activity(&self, bot: &BotId) -> Result<Option<Activity>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM chat_items WHERE bot_id = ?1 AND kind != 'turn' \
             ORDER BY rowid DESC LIMIT 1"
        ))?;
        let item = stmt.query_row([bot.as_str()], from_row).optional()?;
        Ok(item.and_then(|item| activity(&item.body, item.updated_at)))
    }

    /// When the bot last finished a reply, for the unread mark in the
    /// conversation list (spec 15.1); `None` before its first.
    pub fn last_reply_at(&self, bot: &BotId) -> Result<Option<i64>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT MAX(updated_at) FROM chat_items WHERE bot_id = ?1 AND kind = 'reply'",
        )?;
        Ok(stmt.query_row([bot.as_str()], |row| row.get(0))?)
    }
}

#[cfg(test)]
mod tests;
