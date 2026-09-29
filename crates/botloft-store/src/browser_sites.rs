//! Sites a bot may use in its browser without asking again (spec 21.5).

use botloft_core::ids::BotId;
use rusqlite::params;

use crate::{Result, Store};

impl Store {
    /// Remembers that the owner allowed `host` for `bot`. Allowing it again
    /// keeps the first time.
    pub fn allow_browser_site(&self, bot: &BotId, host: &str, now: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO browser_sites (bot_id, host, allowed_at) VALUES (?1, ?2, ?3)
             ON CONFLICT (bot_id, host) DO NOTHING",
            params![bot.as_str(), host, now],
        )?;
        Ok(())
    }

    /// The hosts allowed for `bot`, in the order they were allowed.
    pub fn browser_sites(&self, bot: &BotId) -> Result<Vec<String>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT host FROM browser_sites WHERE bot_id = ?1 ORDER BY allowed_at, host",
        )?;
        let hosts = statement
            .query_map([bot.as_str()], |row| row.get(0))?
            .collect::<rusqlite::Result<_>>()?;
        Ok(hosts)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::Fixture;

    #[test]
    fn sites_are_kept_per_bot_once() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        fx.store
            .allow_browser_site(scout, "wikipedia.org", 10)
            .expect("allow");
        fx.store
            .allow_browser_site(scout, "example.com", 20)
            .expect("allow");
        fx.store
            .allow_browser_site(scout, "wikipedia.org", 30)
            .expect("again");
        assert_eq!(
            fx.store.browser_sites(scout).expect("sites"),
            ["wikipedia.org", "example.com"]
        );
        assert!(fx.store.browser_sites(writer).expect("sites").is_empty());
    }
}
