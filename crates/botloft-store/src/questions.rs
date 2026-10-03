//! `questions` table: what bots asked the owner without waiting (spec 23).
//! Each question has a chat item in its bot's chat that mirrors it, and an
//! answer is a message to the bot, saved in the same transaction.

use botloft_core::ids::{BotId, ChatItemId, QuestionId};
use botloft_core::protocol::{
    ChatBody, ChatItem, Delivery, Message, Question, QuestionItem, QuestionStatus,
};
use rusqlite::types::Type;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

const COLUMNS: &str = "q.id, q.crew_id, q.bot_id, q.text, q.options, q.status, q.answer, q.created_at, \
     q.answered_at, q.chat_item_id";

/// A question and the chat item that shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionRecord {
    pub question: Question,
    pub chat_item_id: ChatItemId,
}

/// What changed when the owner answered: the question, its chat item and
/// the answer's own item in the bot's chat.
#[derive(Debug, Clone, PartialEq)]
pub struct Answered {
    pub question: Question,
    pub question_item: ChatItem,
    pub answer_item: ChatItem,
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<QuestionRecord> {
    let options: String = row.get(4)?;
    let options = serde_json::from_str(&options)
        .map_err(|err| rusqlite::Error::FromSqlConversionFailure(4, Type::Text, Box::new(err)))?;
    Ok(QuestionRecord {
        question: Question {
            id: parse_column(row, 0)?,
            crew_id: parse_column(row, 1)?,
            bot_id: parse_column(row, 2)?,
            text: row.get(3)?,
            options,
            status: parse_column(row, 5)?,
            answer: row.get(6)?,
            created_at: row.get(7)?,
            answered_at: row.get(8)?,
        },
        chat_item_id: parse_column(row, 9)?,
    })
}

fn find(conn: &Connection, id: &QuestionId) -> Result<Option<QuestionRecord>> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM questions q WHERE q.id = ?1"),
            [id.as_str()],
            from_row,
        )
        .optional()?)
}

/// Closes an open question; `None` if it was not open.
fn settle(
    conn: &Connection,
    id: &QuestionId,
    status: QuestionStatus,
    answer: Option<&str>,
    now: i64,
) -> Result<Option<QuestionRecord>> {
    let changed = conn.execute(
        "UPDATE questions SET status = ?2, answer = ?3, answered_at = ?4 \
         WHERE id = ?1 AND status = 'open'",
        params![id.as_str(), status.as_str(), answer, now],
    )?;
    if changed == 0 {
        return Ok(None);
    }
    find(conn, id)
}

/// Rewrites the question's chat item with how it stands now.
fn mirror(conn: &Connection, record: &QuestionRecord, now: i64) -> Result<ChatItem> {
    let body = ChatBody::Question(QuestionItem {
        question: record.question.clone(),
    });
    Store::update_chat_item_in(conn, &record.chat_item_id, &body, now)?.ok_or(
        crate::StoreError::Sqlite(rusqlite::Error::QueryReturnedNoRows),
    )
}

impl Store {
    /// Saves a new question with the chat item that shows it.
    pub fn insert_question(&self, question: &Question, item: &ChatItem) -> Result<()> {
        let options = serde_json::to_string(&question.options)
            .map_err(|err| rusqlite::Error::ToSqlConversionFailure(Box::new(err)))?;
        let tx = self.conn.unchecked_transaction()?;
        Self::insert_chat_item_in(&tx, item)?;
        tx.execute(
            "INSERT INTO questions (id, crew_id, bot_id, chat_item_id, text, options, status, \
             answer, created_at, answered_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                question.id.as_str(),
                question.crew_id.as_str(),
                question.bot_id.as_str(),
                item.id.as_str(),
                question.text,
                options,
                question.status.as_str(),
                question.answer,
                question.created_at,
                question.answered_at,
            ],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn question(&self, id: &QuestionId) -> Result<Option<QuestionRecord>> {
        find(&self.conn, id)
    }

    /// How many questions `bot` has open.
    pub fn open_questions(&self, bot: &BotId) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT count(*) FROM questions WHERE bot_id = ?1 AND status = 'open'",
            [bot.as_str()],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count).unwrap_or(0))
    }

    /// Questions of active bots in active crews with `status`, newest first.
    pub fn questions(&self, status: QuestionStatus) -> Result<Vec<Question>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM questions q \
             JOIN bots b ON b.id = q.bot_id JOIN crews c ON c.id = q.crew_id \
             WHERE q.status = ?1 AND b.archived_at IS NULL AND c.archived_at IS NULL \
             ORDER BY q.created_at DESC, q.rowid DESC"
        ))?;
        let rows = stmt.query_map([status.as_str()], |row| {
            from_row(row).map(|record| record.question)
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Answers an open question and saves `message` (the answer, to the
    /// bot) with its delivery, atomically. `None`, with nothing saved, if
    /// the question was not open.
    pub fn answer_question(
        &self,
        id: &QuestionId,
        message: &Message,
        delivery: &Delivery,
    ) -> Result<Option<Answered>> {
        let now = message.created_at;
        let tx = self.conn.unchecked_transaction()?;
        let Some(record) = settle(&tx, id, QuestionStatus::Answered, Some(&message.body), now)?
        else {
            return Ok(None);
        };
        let question_item = mirror(&tx, &record, now)?;
        let answer_item = Self::insert_message_in(&tx, message, delivery)?;
        tx.commit()?;
        Ok(Some(Answered {
            question: record.question,
            question_item,
            answer_item,
        }))
    }

    /// Closes an open question without an answer. `None` if it was not
    /// open.
    pub fn dismiss_question(
        &self,
        id: &QuestionId,
        now: i64,
    ) -> Result<Option<(Question, ChatItem)>> {
        let tx = self.conn.unchecked_transaction()?;
        let Some(record) = settle(&tx, id, QuestionStatus::Dismissed, None, now)? else {
            return Ok(None);
        };
        let item = mirror(&tx, &record, now)?;
        tx.commit()?;
        Ok(Some((record.question, item)))
    }
}

#[cfg(test)]
pub(crate) mod tests;
