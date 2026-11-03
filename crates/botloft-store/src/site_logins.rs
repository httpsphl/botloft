//! The sites the owner signed in to once for the agents, and which agents
//! have each login (spec 35). The cookies stay in the browser profiles.

use botloft_core::ids::BotId;
use rusqlite::params;

use crate::{Result, Store, parse_column};

/// A saved login: the site and when the owner last signed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteLoginRecord {
    pub site: String,
    pub signed_at: i64,
}

impl Store {
    /// Every saved login, by site.
    pub fn site_logins(&self) -> Result<Vec<SiteLoginRecord>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT site, signed_at FROM site_logins ORDER BY site")?;
        let rows = stmt.query_map([], |row| {
            Ok(SiteLoginRecord {
                site: row.get(0)?,
                signed_at: row.get(1)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Saves the login of `site`, or the new time the owner signed in.
    pub fn save_site_login(&self, site: &str, at: i64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO site_logins (site, signed_at) VALUES (?1, ?2)
             ON CONFLICT (site) DO UPDATE SET signed_at = excluded.signed_at",
            params![site, at],
        )?;
        Ok(())
    }

    /// Forgets the login of `site` and who had it.
    pub fn delete_site_login(&self, site: &str) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM bot_site_logins WHERE site = ?1", params![site])?;
        tx.execute("DELETE FROM site_logins WHERE site = ?1", params![site])?;
        tx.commit()?;
        Ok(())
    }

    /// The agents that have the login of `site`.
    pub fn site_login_bots(&self, site: &str) -> Result<Vec<BotId>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT bot_id FROM bot_site_logins WHERE site = ?1 ORDER BY given_at, bot_id",
        )?;
        let rows = stmt.query_map(params![site], |row| parse_column(row, 0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The sites whose login the agent has.
    pub fn bot_site_logins(&self, bot: &BotId) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT site FROM bot_site_logins WHERE bot_id = ?1 ORDER BY site")?;
        let rows = stmt.query_map(params![bot.to_string()], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Gives the login of `site` to `bot`.
    pub fn give_site_login(&self, bot: &BotId, site: &str, at: i64) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO bot_site_logins (bot_id, site, given_at) VALUES (?1, ?2, ?3)",
            params![bot.to_string(), site, at],
        )?;
        Ok(())
    }

    /// Takes every login the agent had; how many there were.
    pub fn take_site_logins_of(&self, bot: &BotId) -> Result<usize> {
        Ok(self.conn.execute(
            "DELETE FROM bot_site_logins WHERE bot_id = ?1",
            params![bot.to_string()],
        )?)
    }

    /// Takes the login of `site` from `bot`.
    pub fn take_site_login(&self, bot: &BotId, site: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM bot_site_logins WHERE bot_id = ?1 AND site = ?2",
            params![bot.to_string(), site],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::Fixture;

    #[test]
    fn a_login_is_saved_given_taken_and_forgotten() {
        let fx = Fixture::new();
        let (scout, cody) = (&fx.bots[0].id, &fx.bots[1].id);
        fx.store.save_site_login("reddit.com", 10).expect("save");
        fx.store.save_site_login("reddit.com", 20).expect("again");
        let logins = fx.store.site_logins().expect("list");
        assert_eq!(logins.len(), 1);
        assert_eq!(logins[0].signed_at, 20);

        fx.store
            .give_site_login(scout, "reddit.com", 30)
            .expect("give");
        fx.store
            .give_site_login(cody, "reddit.com", 31)
            .expect("give");
        fx.store
            .give_site_login(scout, "reddit.com", 32)
            .expect("twice");
        assert_eq!(
            fx.store.site_login_bots("reddit.com").expect("bots"),
            [scout.clone(), cody.clone()]
        );
        assert_eq!(
            fx.store.bot_site_logins(scout).expect("sites"),
            ["reddit.com"]
        );

        fx.store.take_site_login(scout, "reddit.com").expect("take");
        assert_eq!(
            fx.store.site_login_bots("reddit.com").expect("bots"),
            std::slice::from_ref(cody)
        );

        fx.store.delete_site_login("reddit.com").expect("delete");
        assert!(fx.store.site_logins().expect("none").is_empty());
        assert!(fx.store.bot_site_logins(cody).expect("none").is_empty());
    }
}
