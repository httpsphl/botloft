//! Which of the owner's Claude Code plugins each agent may use (spec 41.3),
//! by the plugin's name: chosen one by one, or for every agent.

use botloft_core::ids::BotId;
use rusqlite::params;

use crate::{Result, Store, parse_column};

/// The setting that gives every plugin to every agent on Claude Code.
const ALL_PLUGINS_EVERYONE: &str = "all_plugins_everyone";

impl Store {
    /// The agent's plugins, by name.
    pub fn bot_plugins(&self, bot: &BotId) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT name FROM bot_plugins WHERE bot_id = ?1 ORDER BY name")?;
        let rows = stmt.query_map(params![bot.to_string()], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The agents that have the plugin `name`.
    pub fn plugin_bots(&self, name: &str) -> Result<Vec<BotId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT bot_id FROM bot_plugins WHERE name = ?1 ORDER BY bot_id")?;
        let rows = stmt.query_map(params![name], |row| parse_column(row, 0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Every plugin some agent has.
    pub fn given_plugins(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT DISTINCT name FROM bot_plugins ORDER BY name")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Puts `bots` in place of the agents that had the plugin `name`.
    pub fn set_plugin_bots(&self, name: &str, bots: &[BotId]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM bot_plugins WHERE name = ?1", params![name])?;
        for bot in bots {
            tx.execute(
                "INSERT OR IGNORE INTO bot_plugins (bot_id, name) VALUES (?1, ?2)",
                params![bot.to_string(), name],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// The plugins every agent on Claude Code uses, by name.
    pub fn everyone_plugins(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT name FROM plugins_everyone ORDER BY name")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Gives the plugin `name` to every agent on Claude Code, or takes that
    /// back: then the agents chosen one by one keep it.
    pub fn set_plugin_everyone(&self, name: &str, on: bool) -> Result<()> {
        let sql = if on {
            "INSERT OR IGNORE INTO plugins_everyone (name) VALUES (?1)"
        } else {
            "DELETE FROM plugins_everyone WHERE name = ?1"
        };
        self.conn.execute(sql, params![name])?;
        Ok(())
    }

    /// Whether every agent on Claude Code uses every plugin.
    pub fn all_plugins_everyone(&self) -> Result<bool> {
        Ok(self.setting(ALL_PLUGINS_EVERYONE)?.is_some())
    }

    /// Gives every plugin to every agent on Claude Code, or takes that back.
    pub fn set_all_plugins_everyone(&self, on: bool) -> Result<()> {
        let sql = if on {
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, '1')"
        } else {
            "DELETE FROM settings WHERE key = ?1"
        };
        self.conn.execute(sql, params![ALL_PLUGINS_EVERYONE])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::Fixture;

    #[test]
    fn a_plugin_goes_to_the_agents_chosen_in_place_of_those_before() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        fx.store
            .set_plugin_bots("vercel", &[scout.clone(), writer.clone(), scout.clone()])
            .expect("set");
        fx.store
            .set_plugin_bots("data", std::slice::from_ref(writer))
            .expect("set");
        assert_eq!(
            fx.store.bot_plugins(writer).expect("list"),
            ["data", "vercel"]
        );
        assert_eq!(fx.store.plugin_bots("vercel").expect("bots").len(), 2);
        fx.store.set_plugin_bots("vercel", &[]).expect("none");
        assert!(fx.store.bot_plugins(scout).expect("none").is_empty());
        assert_eq!(fx.store.given_plugins().expect("given"), ["data"]);
    }

    #[test]
    fn a_plugin_or_all_of_them_go_to_everyone_and_come_back() {
        let fx = Fixture::new();
        assert!(fx.store.everyone_plugins().expect("none").is_empty());
        fx.store.set_plugin_everyone("data", true).expect("on");
        fx.store.set_plugin_everyone("vercel", true).expect("on");
        fx.store.set_plugin_everyone("vercel", true).expect("twice");
        assert_eq!(
            fx.store.everyone_plugins().expect("list"),
            ["data", "vercel"]
        );
        fx.store.set_plugin_everyone("data", false).expect("off");
        assert_eq!(fx.store.everyone_plugins().expect("list"), ["vercel"]);

        assert!(!fx.store.all_plugins_everyone().expect("off"));
        fx.store.set_all_plugins_everyone(true).expect("on");
        assert!(fx.store.all_plugins_everyone().expect("on"));
        // Skills and plugins are set apart.
        assert!(!fx.store.all_skills_everyone().expect("skills"));
        fx.store.set_all_plugins_everyone(false).expect("off");
        assert!(!fx.store.all_plugins_everyone().expect("off"));
    }
}
