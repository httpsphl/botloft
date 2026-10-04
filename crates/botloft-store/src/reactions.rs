//! `reactions` table (spec 8.9): the owner's emoji on a bot's reply, which
//! the owner's next message takes to the bot.

use botloft_core::ids::{BotId, ChatItemId, MessageId};
use botloft_core::protocol::Reaction;
use rusqlite::{Connection, Row, params};

use crate::messages::optional_column;
use crate::{Result, Store, cached_execute, parse_column};

const COLUMNS: &str = "bot_id, item_id, emoji, created_at, sent_in";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Reaction> {
    Ok(Reaction {
        bot_id: parse_column(row, 0)?,
        item_id: parse_column(row, 1)?,
        emoji: row.get(2)?,
        created_at: row.get(3)?,
        sent_in: optional_column(row, 4)?,
    })
}

impl Store {
    /// Puts or changes the owner's reaction on a reply. A changed one waits
    /// for the next message again.
    pub fn set_reaction(&self, reaction: &Reaction, quote: &str) -> Result<()> {
        cached_execute(
            &self.conn,
            "INSERT INTO reactions (bot_id, item_id, emoji, quote, created_at, sent_in) \
             VALUES (?1, ?2, ?3, ?4, ?5, NULL) \
             ON CONFLICT (item_id) DO UPDATE SET \
             emoji = excluded.emoji, quote = excluded.quote, \
             created_at = excluded.created_at, sent_in = NULL",
            params![
                reaction.bot_id.as_str(),
                reaction.item_id.as_str(),
                reaction.emoji,
                quote,
                reaction.created_at,
            ],
        )?;
        Ok(())
    }

    /// Takes the reaction off; whether there was one.
    pub fn remove_reaction(&self, item: &ChatItemId) -> Result<bool> {
        let removed = cached_execute(
            &self.conn,
            "DELETE FROM reactions WHERE item_id = ?1",
            [item.as_str()],
        )?;
        Ok(removed > 0)
    }

    /// The bot's reactions, oldest first.
    pub fn reactions(&self, bot: &BotId) -> Result<Vec<Reaction>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM reactions WHERE bot_id = ?1 ORDER BY created_at, item_id"
        ))?;
        let rows = stmt.query_map([bot.as_str()], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The reactions `message` took to its bot, with what each quotes,
    /// oldest first.
    pub fn reactions_sent_in(&self, message: &MessageId) -> Result<Vec<(Reaction, String)>> {
        let mut stmt = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS}, quote FROM reactions WHERE sent_in = ?1 ORDER BY created_at, item_id"
        ))?;
        let rows = stmt.query_map([message.as_str()], |row| Ok((from_row(row)?, row.get(5)?)))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Gives the bot's waiting reactions to the owner's `message`, inside
    /// the caller's transaction (spec 8.9).
    pub(crate) fn send_reactions_in(
        conn: &Connection,
        bot: &BotId,
        message: &MessageId,
    ) -> Result<()> {
        cached_execute(
            conn,
            "UPDATE reactions SET sent_in = ?2 WHERE bot_id = ?1 AND sent_in IS NULL",
            params![bot.as_str(), message.as_str()],
        )?;
        Ok(())
    }
}
