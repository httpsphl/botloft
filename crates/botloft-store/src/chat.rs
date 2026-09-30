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
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::{ActivityKind, ReplyItem, ToolItem, ToolStatus, TurnItem};

    use super::*;
    use crate::tests::Fixture;

    fn item(bot: &BotId, body: ChatBody, at: i64) -> ChatItem {
        ChatItem {
            id: ChatItemId::generate(),
            bot_id: bot.clone(),
            body,
            created_at: at,
            updated_at: at,
        }
    }

    fn reply(text: &str) -> ChatBody {
        ChatBody::Reply(ReplyItem { text: text.into() })
    }

    #[test]
    fn items_page_back_newest_first() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        let other = &fx.bots[1].id;
        let ids: Vec<_> = (0..3)
            .map(|n| {
                let entry = item(bot, reply(&format!("r{n}")), 10 + n);
                fx.store.insert_chat_item(&entry).expect("insert");
                entry.id
            })
            .collect();
        fx.store
            .insert_chat_item(&item(other, reply("elsewhere"), 20))
            .expect("insert");
        let newest = fx.store.chat_history(bot, None, 2).expect("history");
        assert_eq!(
            newest.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
            [ids[2].clone(), ids[1].clone()]
        );
        let older = fx
            .store
            .chat_history(bot, Some(&ids[1]), 10)
            .expect("older");
        assert_eq!(older.len(), 1);
        assert_eq!(older[0].body, reply("r0"));
    }

    #[test]
    fn a_tool_item_is_found_and_updated() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        let running = ToolItem {
            tool_use_id: "toolu_7".into(),
            name: "Bash".into(),
            summary: "npm test".into(),
            explanation: None,
            input: "{}".into(),
            status: ToolStatus::Running,
            output: None,
            file: None,
        };
        let entry = item(bot, ChatBody::Tool(running.clone()), 5);
        fx.store.insert_chat_item(&entry).expect("insert");
        let found = fx
            .store
            .tool_item(bot, "toolu_7")
            .expect("find")
            .expect("some");
        assert_eq!(found.id, entry.id);
        assert!(fx.store.tool_item(bot, "toolu_8").expect("find").is_none());

        let done = ChatBody::Tool(ToolItem {
            status: ToolStatus::Done,
            output: Some("ok".into()),
            ..running
        });
        let updated = fx
            .store
            .update_chat_item(&entry.id, &done, 9)
            .expect("update")
            .expect("some");
        assert_eq!(
            (updated.body, updated.created_at, updated.updated_at),
            (done, 5, 9)
        );
    }

    #[test]
    fn written_files_come_newest_first_without_repeats_or_failures() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        let write = |file: Option<&str>, status: ToolStatus| {
            ChatBody::Tool(ToolItem {
                tool_use_id: "t".into(),
                name: "Write".into(),
                summary: String::new(),
                explanation: None,
                input: "{}".into(),
                status,
                output: None,
                file: file.map(str::to_owned),
            })
        };
        for (n, (file, status)) in [
            (Some(r"C:\work\a.md"), ToolStatus::Done),
            (Some(r"C:\work\b.pdf"), ToolStatus::Done),
            (Some(r"C:\work\bad.md"), ToolStatus::Failed),
            (None, ToolStatus::Done),
            (Some(r"C:\work\a.md"), ToolStatus::Done),
        ]
        .into_iter()
        .enumerate()
        {
            fx.store
                .insert_chat_item(&item(bot, write(file, status), n as i64))
                .expect("insert");
        }
        let files = fx.store.written_files(bot, 10).expect("files");
        assert_eq!(files, [r"C:\work\a.md", r"C:\work\b.pdf"]);
        assert_eq!(
            fx.store.written_files(bot, 1).expect("one"),
            [r"C:\work\a.md"]
        );
    }

    #[test]
    fn the_last_activity_skips_turn_ends() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        assert_eq!(fx.store.last_activity(bot).expect("none"), None);
        fx.store
            .insert_chat_item(&item(bot, reply("All done.\nDetails below"), 7))
            .expect("insert");
        let turn = ChatBody::Turn(TurnItem {
            duration_ms: 1000,
            tokens: None,
            error: None,
        });
        fx.store
            .insert_chat_item(&item(bot, turn, 8))
            .expect("insert");
        assert_eq!(
            fx.store.last_activity(bot).expect("activity"),
            Some(Activity {
                kind: ActivityKind::Reply,
                text: "All done. Details below".into(),
                tool: None,
                at: 7
            })
        );
    }
}
