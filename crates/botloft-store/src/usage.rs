//! Tokens and cost per bot (spec 8.7): the tokens summed from the `turn`
//! items of the chat, the cost from `turn_costs`.

use botloft_core::protocol::{BotTokens, TokenUsage};
use rusqlite::{Row, params};

use crate::{Result, Store, parse_column};

/// A sum of token counts; SQLite keeps integers as `i64`.
fn count(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    Ok(u64::try_from(row.get::<_, i64>(index)?).unwrap_or_default())
}

impl Store {
    /// Each bot's tokens and cost from `since` on: the dearest bot first,
    /// then the bots with no price by their tokens. A bot on Claude always
    /// has a price (0 for turns from before Botloft kept it); the other
    /// engines give none. Bots with neither a turn nor a cost in the period
    /// are left out.
    pub fn bot_usage(&self, since: i64) -> Result<Vec<BotTokens>> {
        let mut stmt = self.conn.prepare_cached(
            "WITH t AS ( \
                SELECT bot_id, COUNT(*) AS turns, \
                       SUM(json_extract(data, '$.tokens.input')) AS input, \
                       SUM(json_extract(data, '$.tokens.cacheWrite')) AS cache_write, \
                       SUM(json_extract(data, '$.tokens.cacheRead')) AS cache_read, \
                       SUM(json_extract(data, '$.tokens.output')) AS output, \
                       COALESCE(SUM(json_extract(data, '$.tokens.reloaded')), 0) AS reloaded \
                FROM chat_items \
                WHERE kind = 'turn' AND created_at >= ?1 \
                  AND json_type(data, '$.tokens') = 'object' \
                GROUP BY bot_id), \
             k AS (SELECT bot_id, SUM(cost) AS cost FROM turn_costs WHERE at >= ?1 GROUP BY bot_id) \
             SELECT b.id, b.name, b.color, b.archived_at IS NOT NULL, COALESCE(t.turns, 0), \
                    COALESCE(t.input, 0), COALESCE(t.cache_write, 0), COALESCE(t.cache_read, 0), \
                    COALESCE(t.output, 0), COALESCE(t.reloaded, 0), w.name, \
                    CASE WHEN b.agent = 'claude' THEN COALESCE(k.cost, 0) ELSE k.cost END AS cost \
             FROM bots b JOIN crews w ON w.id = b.crew_id \
             LEFT JOIN t ON t.bot_id = b.id LEFT JOIN k ON k.bot_id = b.id \
             WHERE t.bot_id IS NOT NULL OR k.bot_id IS NOT NULL \
             ORDER BY cost IS NULL, cost DESC, \
                      COALESCE(t.input + t.cache_write + t.output, 0) DESC, b.name, w.name",
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
                cost: row.get(11)?,
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
            conversation_id: botloft_core::ids::ConversationId::main_of(bot),
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
            cost_micros: None,
            error: None,
            stopped: false,
            quiet: false,
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

        let all = fx.store.bot_usage(10).expect("tokens");
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
        assert!(fx.store.bot_usage(41).expect("tokens").is_empty());
    }

    #[test]
    fn the_dearest_bot_comes_first_and_a_cost_alone_counts() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        // Scout used many tokens, on a cheap model.
        add(&fx, scout, turn(Some(tokens(5_000, 900))), 10);
        let cost = |bot, cost| {
            fx.store
                .add_turn_cost(&crate::TurnCost {
                    bot,
                    at: 10,
                    cost,
                    model: None,
                    run: None,
                })
                .expect("cost");
        };
        cost(scout, 0.05);
        // A compaction costs but leaves no turn.
        cost(writer, 0.40);
        let all = fx.store.bot_usage(0).expect("usage");
        let order: Vec<&BotId> = all.iter().map(|b| &b.bot_id).collect();
        assert_eq!(order, [writer, scout]);
        assert_eq!(all[0].turns, 0);
        assert_eq!(all[0].tokens, TokenUsage::default());
        assert_eq!(all[0].cost, Some(0.40));
        assert_eq!(all[1].cost, Some(0.05));
    }

    #[test]
    fn only_a_bot_on_another_engine_has_no_price() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        fx.store
            .conn
            .execute(
                "UPDATE bots SET agent = 'codex' WHERE id = ?1",
                [writer.as_str()],
            )
            .expect("codex");
        // Turns from before Botloft kept the cost.
        add(&fx, scout, turn(Some(tokens(1, 2))), 10);
        add(&fx, writer, turn(Some(tokens(900, 900))), 10);
        let all = fx.store.bot_usage(0).expect("usage");
        let order: Vec<&BotId> = all.iter().map(|b| &b.bot_id).collect();
        assert_eq!(order, [scout, writer], "the bots with no price go last");
        assert_eq!(all[0].cost, Some(0.0));
        assert_eq!(all[1].cost, None);
    }
}
