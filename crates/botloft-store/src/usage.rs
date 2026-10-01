//! Tokens per bot (spec 8.7), summed from the `turn` items of the chat.

use botloft_core::protocol::{BotTokens, TokenUsage};
use rusqlite::{Row, params};

use crate::{Result, Store, parse_column};

/// A sum of token counts; SQLite keeps integers as `i64`.
fn count(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    Ok(u64::try_from(row.get::<_, i64>(index)?).unwrap_or_default())
}

impl Store {
    /// Each bot's tokens in turns that ended at or after `since`, the bot
    /// that used the most first. Bots with no such turn are left out.
    pub fn turn_tokens(&self, since: i64) -> Result<Vec<BotTokens>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT b.id, b.name, b.color, b.archived_at IS NOT NULL, COUNT(*), \
                    SUM(json_extract(c.data, '$.tokens.input')), \
                    SUM(json_extract(c.data, '$.tokens.cacheWrite')), \
                    SUM(json_extract(c.data, '$.tokens.cacheRead')), \
                    SUM(json_extract(c.data, '$.tokens.output')) AS output, \
                    COALESCE(SUM(json_extract(c.data, '$.tokens.reloaded')), 0) AS reloaded, \
                    w.name \
             FROM chat_items c JOIN bots b ON b.id = c.bot_id JOIN crews w ON w.id = b.crew_id \
             WHERE c.kind = 'turn' AND c.created_at >= ?1 \
               AND json_type(c.data, '$.tokens') = 'object' \
             GROUP BY b.id \
             ORDER BY SUM(json_extract(c.data, '$.tokens.input')) \
                    + SUM(json_extract(c.data, '$.tokens.cacheWrite')) - reloaded + output DESC, \
                    b.name, w.name",
        )?;
        let rows = stmt.query_map(params![since], |row| {
            Ok(BotTokens {
                bot_id: parse_column(row, 0)?,
                name: row.get(1)?,
                color: row.get(2)?,
                crew: row.get(10)?,
                archived: row.get(3)?,
                turns: row.get(4)?,
                tokens: TokenUsage {
                    input: count(row, 5)?,
                    cache_write: count(row, 6)?,
                    reloaded: count(row, 9)?,
                    cache_read: count(row, 7)?,
                    output: count(row, 8)?,
                },
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::ids::{BotId, ChatItemId};
    use botloft_core::protocol::{ChatBody, ChatItem, ReplyItem, TurnItem};

    use super::*;
    use crate::tests::Fixture;

    fn add(fx: &Fixture, bot: &BotId, body: ChatBody, at: i64) {
        let item = ChatItem {
            id: ChatItemId::generate(),
            bot_id: bot.clone(),
            body,
            created_at: at,
            updated_at: at,
        };
        fx.store.insert_chat_item(&item).expect("insert");
    }

    fn turn(tokens: Option<TokenUsage>) -> ChatBody {
        ChatBody::Turn(TurnItem {
            duration_ms: 1000,
            tokens,
            error: None,
        })
    }

    fn tokens(input: u64, output: u64) -> TokenUsage {
        TokenUsage {
            input,
            cache_write: 100,
            reloaded: 40,
            cache_read: 5000,
            output,
        }
    }

    #[test]
    fn turns_are_summed_per_bot_from_the_time_asked() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        add(&fx, scout, turn(Some(tokens(10, 20))), 5);
        add(&fx, scout, turn(Some(tokens(1, 2))), 20);
        add(&fx, scout, turn(Some(tokens(3, 4))), 30);
        // Older turns and other items do not count.
        add(&fx, scout, turn(None), 25);
        let reply = ChatBody::Reply(ReplyItem { text: "hi".into() });
        add(&fx, scout, reply, 26);
        add(&fx, writer, turn(Some(tokens(500, 0))), 40);

        let all = fx.store.turn_tokens(10).expect("tokens");
        let order: Vec<&BotId> = all.iter().map(|b| &b.bot_id).collect();
        assert_eq!(order, [writer, scout], "the bot that used the most first");
        let scout_tokens = &all[1];
        assert_eq!(scout_tokens.name, "scout");
        assert_eq!(scout_tokens.crew, "Ops");
        assert!(!scout_tokens.archived);
        assert_eq!(scout_tokens.turns, 2);
        assert_eq!(
            scout_tokens.tokens,
            TokenUsage {
                input: 4,
                cache_write: 200,
                reloaded: 80,
                cache_read: 10_000,
                output: 6,
            }
        );
        assert!(fx.store.turn_tokens(41).expect("tokens").is_empty());
    }
}
