//! Sites a bot may use in its browser without asking again (spec 21.5).

use botloft_core::ids::BotId;
use botloft_core::protocol::BrowserSite;
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
        Ok(self
            .allowed_sites(bot)?
            .into_iter()
            .map(|site| site.site)
            .collect())
    }

    /// The sites allowed for `bot` with when, in the order they were
    /// allowed: what the owner sees in Settings (spec 15.1).
    pub fn allowed_sites(&self, bot: &BotId) -> Result<Vec<BrowserSite>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT host, allowed_at FROM browser_sites WHERE bot_id = ?1 \
             ORDER BY allowed_at, host",
        )?;
        let sites = statement
            .query_map([bot.as_str()], |row| {
                Ok(BrowserSite {
                    bot_id: bot.clone(),
                    site: row.get(0)?,
                    allowed_at: row.get(1)?,
                })
            })?
            .collect::<rusqlite::Result<_>>()?;
        Ok(sites)
    }

    /// Takes `host` away from `bot`: it asks the next time. Whether it was
    /// there.
    pub fn revoke_browser_site(&self, bot: &BotId, host: &str) -> Result<bool> {
        let removed = self.conn.execute(
            "DELETE FROM browser_sites WHERE bot_id = ?1 AND host = ?2",
            params![bot.as_str(), host],
        )?;
        Ok(removed > 0)
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
        let sites = fx.store.allowed_sites(scout).expect("sites");
        assert_eq!(sites[0].allowed_at, 10);
        assert_eq!(&sites[0].bot_id, scout);
        assert!(fx.store.browser_sites(writer).expect("sites").is_empty());
    }

    #[test]
    fn a_site_taken_away_is_asked_for_again() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        for bot in [scout, writer] {
            fx.store
                .allow_browser_site(bot, "wikipedia.org", 10)
                .expect("allow");
        }
        assert!(
            fx.store
                .revoke_browser_site(scout, "wikipedia.org")
                .expect("revoke")
        );
        assert!(fx.store.browser_sites(scout).expect("sites").is_empty());
        // The other bot keeps its own; a site not there is not an error.
        assert_eq!(
            fx.store.browser_sites(writer).expect("sites"),
            ["wikipedia.org"]
        );
        assert!(
            !fx.store
                .revoke_browser_site(scout, "wikipedia.org")
                .expect("again")
        );
    }
}
