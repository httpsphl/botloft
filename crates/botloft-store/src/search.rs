//! Searching the chats (spec 8.8) through `chat_search`, the FTS5 index the
//! triggers of migration 0016 keep over `chat_items`.

use botloft_core::ids::{BotId, ChatItemId, CrewId};
use botloft_core::protocol::{ChatItem, SNIPPET_CLOSE, SNIPPET_OPEN};
use rusqlite::params;

use crate::{Result, Store, chat, parse_column};

/// Words of context a snippet keeps around the match.
const SNIPPET_TOKENS: u32 = 18;

/// Which chats a search looks in, and where its page starts.
#[derive(Debug, Clone, Copy, Default)]
pub struct SearchFilter<'a> {
    pub bot: Option<&'a BotId>,
    pub crew: Option<&'a CrewId>,
    /// Only items stored before this one.
    pub before: Option<&'a ChatItemId>,
    pub limit: u32,
}

/// A chat item that matched, with its crew and a snippet.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    pub item: ChatItem,
    pub crew: CrewId,
    pub snippet: String,
}

/// The words of `text` as an FTS5 query: every word must be there, each
/// taken literally, and the last one may be the start of a word. `None`
/// when no word is left.
pub fn fts_query(text: &str) -> Option<String> {
    let words: Vec<String> = text
        .split_whitespace()
        .map(|word| word.replace('"', ""))
        .filter(|word| word.chars().any(char::is_alphanumeric))
        .collect();
    let (last, rest) = words.split_last()?;
    let mut query: Vec<String> = rest.iter().map(|word| format!("\"{word}\"")).collect();
    query.push(format!("\"{last}\"*"));
    Some(query.join(" "))
}

impl Store {
    /// Chat items of active bots in active crews that match `query` (an
    /// FTS5 query, see [`fts_query`]), newest first.
    pub fn search_chat(&self, query: &str, filter: SearchFilter<'_>) -> Result<Vec<Found>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT c.id, c.bot_id, c.data, c.created_at, c.updated_at, b.crew_id, \
                    snippet(chat_search, 0, ?6, ?7, '…', ?8) \
             FROM chat_search \
             JOIN chat_items c ON c.rowid = chat_search.rowid \
             JOIN bots b ON b.id = c.bot_id \
             JOIN crews k ON k.id = b.crew_id \
             WHERE chat_search MATCH ?1 \
               AND b.archived_at IS NULL AND k.archived_at IS NULL \
               AND (?2 IS NULL OR c.bot_id = ?2) \
               AND (?3 IS NULL OR b.crew_id = ?3) \
               AND (?4 IS NULL OR c.rowid < (SELECT rowid FROM chat_items WHERE id = ?4)) \
             ORDER BY c.rowid DESC LIMIT ?5",
        )?;
        let rows = stmt.query_map(
            params![
                query,
                filter.bot.map(BotId::as_str),
                filter.crew.map(CrewId::as_str),
                filter.before.map(ChatItemId::as_str),
                filter.limit,
                SNIPPET_OPEN.to_string(),
                SNIPPET_CLOSE.to_string(),
                SNIPPET_TOKENS,
            ],
            |row| {
                Ok(Found {
                    item: chat::from_row(row)?,
                    crew: parse_column(row, 5)?,
                    snippet: row.get(6)?,
                })
            },
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests;
