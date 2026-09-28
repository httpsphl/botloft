//! `messages` table. A message is saved together with its delivery, and
//! with the task it creates, in one transaction (spec 9.1).

use botloft_core::ids::{BotId, CrewId, MessageId, TaskId};
use botloft_core::protocol::{Delivery, Message, Task};
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

const COLUMNS: &str =
    "id, crew_id, from_kind, from_bot_id, to_bot_id, kind, body, task_id, created_at";

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
        created_at: row.get(8)?,
    })
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
    /// Saves a message, its delivery and the task it creates, atomically.
    pub fn insert_message(
        &self,
        message: &Message,
        delivery: &Delivery,
        task: Option<&Task>,
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        if let Some(task) = task {
            Self::insert_task_in(&tx, task)?;
        }
        Self::insert_message_in(&tx, message, delivery)?;
        tx.commit()?;
        Ok(())
    }

    /// The message and its delivery, inside the caller's transaction.
    pub(crate) fn insert_message_in(
        conn: &Connection,
        message: &Message,
        delivery: &Delivery,
    ) -> Result<()> {
        conn.execute(
            &format!(
                "INSERT INTO messages ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
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
            ],
        )?;
        Self::insert_delivery_in(conn, delivery)
    }

    /// The message that asked for `task`.
    pub fn task_request(&self, task: &TaskId) -> Result<Option<Message>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "SELECT {COLUMNS} FROM messages WHERE task_id = ?1 AND kind = 'task' \
                     ORDER BY rowid LIMIT 1"
                ),
                [task.as_str()],
                from_row,
            )
            .optional()?)
    }

    pub fn message(&self, id: &MessageId) -> Result<Option<Message>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM messages WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Newest first, in the order they were stored.
    pub fn messages(&self, filter: MessageFilter<'_>) -> Result<Vec<Message>> {
        let mut stmt = self.conn.prepare(&format!(
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
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::{MessageKind, SenderKind};

    use super::*;
    use crate::tests::{Fixture, message_to};

    #[test]
    fn a_message_is_saved_with_its_delivery_and_read_back() {
        let fx = Fixture::new();
        let (message, delivery) = message_to(&fx.crew.id, &fx.bots[0].id, "hello");
        fx.store
            .insert_message(&message, &delivery, None)
            .expect("insert");
        assert_eq!(
            fx.store.message(&message.id).expect("read"),
            Some(message.clone())
        );
        assert_eq!(
            fx.store.delivery(&delivery.id).expect("read"),
            Some(delivery)
        );
        assert_eq!(message.from_kind, SenderKind::Owner);
        assert_eq!(message.kind, MessageKind::Note);
    }

    #[test]
    fn a_failed_insert_leaves_nothing_behind() {
        let fx = Fixture::new();
        let (message, mut delivery) = message_to(&fx.crew.id, &fx.bots[0].id, "hello");
        delivery.bot_id = BotId::generate(); // no such bot: the FK fails
        assert!(fx.store.insert_message(&message, &delivery, None).is_err());
        assert_eq!(fx.store.message(&message.id).expect("read"), None);
    }

    #[test]
    fn listing_filters_by_bot_and_pages_back_newest_first() {
        let fx = Fixture::new();
        let mut ids = Vec::new();
        for (i, bot) in [&fx.bots[0], &fx.bots[1], &fx.bots[0]].iter().enumerate() {
            let (message, delivery) = message_to(&fx.crew.id, &bot.id, &format!("m{i}"));
            fx.store
                .insert_message(&message, &delivery, None)
                .expect("insert");
            ids.push(message.id);
        }
        let all = fx
            .store
            .messages(MessageFilter {
                limit: 10,
                ..MessageFilter::default()
            })
            .expect("list");
        let bodies: Vec<_> = all.iter().map(|m| m.body.as_str()).collect();
        assert_eq!(bodies, ["m2", "m1", "m0"]);

        let first_bot = fx
            .store
            .messages(MessageFilter {
                bot: Some(&fx.bots[0].id),
                limit: 10,
                ..MessageFilter::default()
            })
            .expect("list");
        assert_eq!(first_bot.len(), 2);

        let older = fx
            .store
            .messages(MessageFilter {
                crew: Some(&fx.crew.id),
                before: Some(&ids[2]),
                limit: 1,
                ..MessageFilter::default()
            })
            .expect("page");
        assert_eq!(older.len(), 1);
        assert_eq!(older[0].body, "m1");
    }
}
