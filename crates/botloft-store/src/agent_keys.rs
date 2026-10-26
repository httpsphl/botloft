//! The names of an agent's keys (spec 32.2): what the app lists. The values
//! are in `secrets\keys`, never here.

use botloft_core::ids::BotId;
use botloft_core::protocol::AgentKey;
use rusqlite::params;

use crate::{Result, Store};

impl Store {
    /// The agent's keys by name, every one marked as present: the daemon
    /// knows which values are missing.
    pub fn agent_keys(&self, bot: &BotId) -> Result<Vec<AgentKey>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT name, description, set_at FROM agent_keys WHERE bot_id = ?1 ORDER BY name",
        )?;
        let rows = stmt.query_map(params![bot.to_string()], |row| {
            Ok(AgentKey {
                name: row.get(0)?,
                description: row.get(1)?,
                set_at: row.get(2)?,
                missing: false,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Adds a key or marks it changed; a `None` description keeps the old one.
    pub fn set_agent_key(
        &self,
        bot: &BotId,
        name: &str,
        description: Option<&str>,
        at: i64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO agent_keys (bot_id, name, description, set_at) \
             VALUES (?1, ?2, COALESCE(?3, ''), ?4) \
             ON CONFLICT (bot_id, name) DO UPDATE SET \
               description = COALESCE(?3, description), set_at = ?4",
            params![bot.to_string(), name, description, at],
        )?;
        Ok(())
    }

    /// How many of the bot's requests for a key the owner has not answered
    /// (spec 32.5).
    pub fn open_key_requests(&self, bot: &BotId) -> Result<u32> {
        let open: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM chat_items \
             WHERE bot_id = ?1 AND kind = 'key_request' \
               AND json_extract(data, '$.status') = 'open'",
            params![bot.to_string()],
            |row| row.get(0),
        )?;
        Ok(u32::try_from(open).unwrap_or(u32::MAX))
    }

    /// Whether there was such a key.
    pub fn delete_agent_key(&self, bot: &BotId, name: &str) -> Result<bool> {
        let gone = self.conn.execute(
            "DELETE FROM agent_keys WHERE bot_id = ?1 AND name = ?2",
            params![bot.to_string(), name],
        )?;
        Ok(gone > 0)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::Fixture;

    #[test]
    fn names_are_kept_and_a_new_value_keeps_the_description() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        fx.store
            .set_agent_key(bot, "WA_TOKEN", Some("WhatsApp"), 10)
            .expect("set");
        fx.store.set_agent_key(bot, "A_KEY", None, 11).expect("set");
        fx.store
            .set_agent_key(bot, "WA_TOKEN", None, 20)
            .expect("again");
        let keys = fx.store.agent_keys(bot).expect("list");
        let seen: Vec<_> = keys
            .iter()
            .map(|k| (k.name.as_str(), k.description.as_str(), k.set_at))
            .collect();
        assert_eq!(seen, [("A_KEY", "", 11), ("WA_TOKEN", "WhatsApp", 20)]);
        assert!(
            fx.store
                .agent_keys(&fx.bots[1].id)
                .expect("other")
                .is_empty()
        );
        assert!(fx.store.delete_agent_key(bot, "A_KEY").expect("delete"));
        assert!(!fx.store.delete_agent_key(bot, "A_KEY").expect("again"));
    }
}
