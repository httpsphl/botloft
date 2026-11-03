//! Which of the owner's claude.ai connectors each agent may use (spec 34.4),
//! by the connector's plain name.

use botloft_core::ids::BotId;
use rusqlite::params;

use crate::{Result, Store};

impl Store {
    /// The agent's chosen connectors, by name.
    pub fn bot_connectors(&self, bot: &BotId) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT name FROM bot_connectors WHERE bot_id = ?1 ORDER BY name")?;
        let rows = stmt.query_map(params![bot.to_string()], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Puts `names` in place of the agent's choice.
    pub fn set_bot_connectors(&self, bot: &BotId, names: &[String]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "DELETE FROM bot_connectors WHERE bot_id = ?1",
            params![bot.to_string()],
        )?;
        for name in names {
            tx.execute(
                "INSERT OR IGNORE INTO bot_connectors (bot_id, name) VALUES (?1, ?2)",
                params![bot.to_string(), name],
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::Fixture;

    #[test]
    fn a_choice_replaces_the_one_before_per_agent() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        assert!(fx.store.bot_connectors(bot).expect("none").is_empty());
        fx.store
            .set_bot_connectors(
                bot,
                &["Gmail".into(), "Google Calendar".into(), "Gmail".into()],
            )
            .expect("set");
        assert_eq!(
            fx.store.bot_connectors(bot).expect("list"),
            ["Gmail", "Google Calendar"]
        );
        fx.store
            .set_bot_connectors(bot, &["Notion".into()])
            .expect("again");
        assert_eq!(fx.store.bot_connectors(bot).expect("list"), ["Notion"]);
        assert!(
            fx.store
                .bot_connectors(&fx.bots[1].id)
                .expect("other")
                .is_empty()
        );
    }
}
