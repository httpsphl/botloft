//! The agents the owner gave their own Chrome to (spec 37.2).

use botloft_core::ids::BotId;
use botloft_core::protocol::ChromeGrant;
use rusqlite::{OptionalExtension, params};

use crate::{Result, Store, parse_column};

impl Store {
    /// When the owner gave the agent their Chrome; `None` when they did not.
    pub fn bot_chrome(&self, bot: &BotId) -> Result<Option<i64>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT given_at FROM bot_chrome WHERE bot_id = ?1")?;
        Ok(stmt
            .query_row(params![bot.to_string()], |row| row.get(0))
            .optional()?)
    }

    /// Gives the agent the owner's Chrome; giving it again keeps the first
    /// time and the new acceptance of the risks.
    pub fn give_bot_chrome(&self, bot: &BotId, now: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO bot_chrome (bot_id, given_at, accepted_risks_at) VALUES (?1, ?2, ?2)
             ON CONFLICT (bot_id) DO UPDATE SET accepted_risks_at = excluded.accepted_risks_at",
            params![bot.to_string(), now],
        )?;
        Ok(())
    }

    /// Takes the owner's Chrome back; `false` when the agent did not have it.
    pub fn take_bot_chrome(&self, bot: &BotId) -> Result<bool> {
        let removed = self.conn.execute(
            "DELETE FROM bot_chrome WHERE bot_id = ?1",
            params![bot.to_string()],
        )?;
        Ok(removed > 0)
    }

    /// Every agent that has the owner's Chrome, the oldest first.
    pub fn chrome_grants(&self) -> Result<Vec<ChromeGrant>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT bot_id, given_at FROM bot_chrome ORDER BY given_at, bot_id")?;
        let rows = stmt.query_map([], |row| {
            Ok(ChromeGrant {
                bot_id: parse_column(row, 0)?,
                given_at: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::Fixture;

    #[test]
    fn the_chrome_is_given_once_and_taken_back_per_agent() {
        let fx = Fixture::new();
        let bot = &fx.bots[0].id;
        assert_eq!(fx.store.bot_chrome(bot).expect("none"), None);
        fx.store.give_bot_chrome(bot, 10).expect("give");
        fx.store.give_bot_chrome(bot, 20).expect("again");
        assert_eq!(fx.store.bot_chrome(bot).expect("given"), Some(10));
        assert_eq!(fx.store.bot_chrome(&fx.bots[1].id).expect("other"), None);
        let grants = fx.store.chrome_grants().expect("grants");
        assert_eq!(grants.len(), 1);
        assert_eq!(&grants[0].bot_id, bot);
        assert!(fx.store.take_bot_chrome(bot).expect("take"));
        assert!(!fx.store.take_bot_chrome(bot).expect("twice"));
        assert!(fx.store.chrome_grants().expect("empty").is_empty());
    }
}
