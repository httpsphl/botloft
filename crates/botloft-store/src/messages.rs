//! `messages` and `attachments` tables. A message is saved together with
//! its attachments, its delivery, the task it creates and the recipient's
//! chat item, in one transaction (spec 9.1).

use botloft_core::ids::{
    AttachmentId, BotId, ChatItemId, CrewId, MessageId, QuestionId, RoutineId, TaskId,
};
use botloft_core::protocol::{
    Attachment, ChatBody, ChatItem, Delivery, InboundItem, Message, MessageReply, SenderKind, Task,
};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{Result, Store, cached_execute, cached_row, parse_column, to_sql_int};

const COLUMNS: &str = "id, crew_id, from_kind, from_bot_id, to_bot_id, kind, body, task_id, created_at, routine_id, \n     question_id, reply_item_id, reply_text";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Message> {
    Ok(Message {
        id: parse_column(row, 0)?,
        crew_id: parse_column(row, 1)?,
        from_kind: parse_column(row, 2)?,
        from_bot_id: optional_column(row, 3)?,
        to_bot_id: parse_column(row, 4)?,
        kind: parse_column(row, 5)?,
        body: row.get(6)?,
        task_id: optional_column(row, 7)?,
        routine_id: optional_column(row, 9)?,
        question_id: optional_column(row, 10)?,
        attachments: Vec::new(),
        reply_to: match (optional_column::<ChatItemId>(row, 11)?, row.get(12)?) {
            (Some(item_id), Some(text)) => Some(MessageReply { item_id, text }),
            _ => None,
        },
        created_at: row.get(8)?,
    })
}

fn attachment_from_row(row: &Row<'_>) -> rusqlite::Result<Attachment> {
    Ok(Attachment {
        id: parse_column(row, 0)?,
        name: row.get(1)?,
        media_type: row.get(2)?,
        size: u64::try_from(row.get::<_, i64>(3)?).unwrap_or(0),
        path: row.get(4)?,
    })
}

/// Fills in each message's attachments.
fn with_attachments(conn: &Connection, mut messages: Vec<Message>) -> Result<Vec<Message>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, name, media_type, size, path FROM attachments \
         WHERE message_id = ?1 ORDER BY rowid",
    )?;
    for message in &mut messages {
        let rows = stmt.query_map([message.id.as_str()], attachment_from_row)?;
        message.attachments = rows.collect::<rusqlite::Result<_>>()?;
    }
    Ok(messages)
}

/// Reads a nullable text column into an optional parsed type.
pub(crate) fn optional_column<T>(row: &Row<'_>, idx: usize) -> rusqlite::Result<Option<T>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    match row.get::<_, Option<String>>(idx)? {
        Some(_) => parse_column(row, idx).map(Some),
        None => Ok(None),
    }
}

/// Which messages `messages` returns, newest first.
#[derive(Debug, Clone, Copy, Default)]
pub struct MessageFilter<'a> {
    pub crew: Option<&'a CrewId>,
    /// Sent to or by this bot.
    pub bot: Option<&'a BotId>,
    /// Only messages stored before this one.
    pub before: Option<&'a MessageId>,
    pub limit: u32,
}

impl Store {
    /// Saves a message with its attachments, its delivery, the task it
    /// creates and the recipient's chat item, atomically. Returns the item.
    pub fn insert_message(
        &self,
        message: &Message,
        delivery: &Delivery,
        task: Option<&Task>,
    ) -> Result<ChatItem> {
        let tx = self.conn.unchecked_transaction()?;
        if let Some(task) = task {
            Self::insert_task_in(&tx, task)?;
        }
        let item = Self::insert_message_in(&tx, message, delivery)?;
        tx.commit()?;
        Ok(item)
    }

    /// Everything [`Store::insert_message`] saves but the task, inside the
    /// caller's transaction.
    pub(crate) fn insert_message_in(
        conn: &Connection,
        message: &Message,
        delivery: &Delivery,
    ) -> Result<ChatItem> {
        cached_execute(
            conn,
            &format!(
                "INSERT INTO messages ({COLUMNS})                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)"
            ),
            params![
                message.id.as_str(),
                message.crew_id.as_str(),
                message.from_kind.as_str(),
                message.from_bot_id.as_ref().map(BotId::as_str),
                message.to_bot_id.as_str(),
                message.kind.as_str(),
                message.body,
                message.task_id.as_ref().map(TaskId::as_str),
                message.created_at,
                message.routine_id.as_ref().map(RoutineId::as_str),
                message.question_id.as_ref().map(QuestionId::as_str),
                message
                    .reply_to
                    .as_ref()
                    .map(|reply| reply.item_id.as_str()),
                message.reply_to.as_ref().map(|reply| reply.text.as_str()),
            ],
        )?;
        for attachment in &message.attachments {
            cached_execute(
                conn,
                "INSERT INTO attachments (id, message_id, name, media_type, size, path, created_at) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    attachment.id.as_str(),
                    message.id.as_str(),
                    attachment.name,
                    attachment.media_type,
                    to_sql_int(attachment.size),
                    attachment.path,
                    message.created_at,
                ],
            )?;
        }
        Self::insert_delivery_in(conn, delivery)?;
        // Every message of the owner takes the reactions waiting for one.
        if message.from_kind == SenderKind::Owner {
            Self::send_reactions_in(conn, &message.to_bot_id, &message.id)?;
        }
        let item = ChatItem {
            id: ChatItemId::generate(),
            bot_id: message.to_bot_id.clone(),
            body: ChatBody::Inbound(InboundItem {
                message: message.clone(),
            }),
            created_at: message.created_at,
            updated_at: message.created_at,
        };
        Self::insert_chat_item_in(conn, &item)?;
        Ok(item)
    }

    /// The message that asked for `task`.
    pub fn task_request(&self, task: &TaskId) -> Result<Option<Message>> {
        Ok(cached_row(
            &self.conn,
            &format!(
                "SELECT {COLUMNS} FROM messages WHERE task_id = ?1 AND kind = 'task' \
                     ORDER BY rowid LIMIT 1"
            ),
            [task.as_str()],
            from_row,
        )
        .optional()?)
    }

    /// Whether `from` sent `to` a message at `since` or later.
    pub fn wrote_to_since(&self, from: &BotId, to: &BotId, since: i64) -> Result<bool> {
        Ok(self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM messages \
             WHERE from_bot_id = ?1 AND to_bot_id = ?2 AND created_at >= ?3)",
            params![from.as_str(), to.as_str(), since],
            |row| row.get(0),
        )?)
    }

    pub fn message(&self, id: &MessageId) -> Result<Option<Message>> {
        let message = self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM messages WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?;
        Ok(with_attachments(&self.conn, message.into_iter().collect())?.pop())
    }

    /// An attachment and the bot whose folder holds it.
    pub fn attachment(&self, id: &AttachmentId) -> Result<Option<(Attachment, BotId)>> {
        let found = self
            .conn
            .query_row(
                "SELECT a.id, a.name, a.media_type, a.size, a.path, m.to_bot_id \
                 FROM attachments a JOIN messages m ON m.id = a.message_id WHERE a.id = ?1",
                [id.as_str()],
                |row| Ok((attachment_from_row(row)?, parse_column(row, 5)?)),
            )
            .optional()?;
        Ok(found)
    }

    /// Newest first, in the order they were stored.
    pub fn messages(&self, filter: MessageFilter<'_>) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM messages \
             WHERE (?1 IS NULL OR crew_id = ?1) \
               AND (?2 IS NULL OR to_bot_id = ?2 OR from_bot_id = ?2) \
               AND (?3 IS NULL OR rowid < (SELECT rowid FROM messages WHERE id = ?3)) \
             ORDER BY rowid DESC LIMIT ?4"
        ))?;
        let rows = stmt.query_map(
            params![
                filter.crew.map(CrewId::as_str),
                filter.bot.map(BotId::as_str),
                filter.before.map(MessageId::as_str),
                filter.limit,
            ],
            from_row,
        )?;
        with_attachments(&self.conn, rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests;
