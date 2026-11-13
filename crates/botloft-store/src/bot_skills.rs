//! Which of the owner's skills each agent may use (spec 38.3), by the name
//! of the skill's folder: chosen one by one, or for every agent.

use botloft_core::ids::BotId;
use rusqlite::params;

use crate::{Result, Store, parse_column};

/// The setting that gives every skill to every agent on Claude Code.
const ALL_SKILLS_EVERYONE: &str = "all_skills_everyone";

impl Store {
    /// The agent's skills, by name.
    pub fn bot_skills(&self, bot: &BotId) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT name FROM bot_skills WHERE bot_id = ?1 ORDER BY name")?;
        let rows = stmt.query_map(params![bot.to_string()], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// The agents that have the skill `name`.
    pub fn skill_bots(&self, name: &str) -> Result<Vec<BotId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT bot_id FROM bot_skills WHERE name = ?1 ORDER BY bot_id")?;
        let rows = stmt.query_map(params![name], |row| parse_column(row, 0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Every skill some agent has.
    pub fn given_skills(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT DISTINCT name FROM bot_skills ORDER BY name")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Puts `bots` in place of the agents that had the skill `name`.
    pub fn set_skill_bots(&self, name: &str, bots: &[BotId]) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute("DELETE FROM bot_skills WHERE name = ?1", params![name])?;
        for bot in bots {
            tx.execute(
                "INSERT OR IGNORE INTO bot_skills (bot_id, name) VALUES (?1, ?2)",
                params![bot.to_string(), name],
            )?;
        }
        tx.commit()?;
        Ok(())
    }

    /// The skills every agent on Claude Code uses, by name.
    pub fn everyone_skills(&self) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT name FROM skills_everyone ORDER BY name")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Gives the skill `name` to every agent on Claude Code, or takes that
    /// back: then the agents chosen one by one keep it.
    pub fn set_skill_everyone(&self, name: &str, on: bool) -> Result<()> {
        let sql = if on {
            "INSERT OR IGNORE INTO skills_everyone (name) VALUES (?1)"
        } else {
            "DELETE FROM skills_everyone WHERE name = ?1"
        };
        self.conn.execute(sql, params![name])?;
        Ok(())
    }

    /// Whether every agent on Claude Code uses every skill.
    pub fn all_skills_everyone(&self) -> Result<bool> {
        Ok(self.setting(ALL_SKILLS_EVERYONE)?.is_some())
    }

    /// Gives every skill to every agent on Claude Code, or takes that back.
    pub fn set_all_skills_everyone(&self, on: bool) -> Result<()> {
        let sql = if on {
            "INSERT OR IGNORE INTO settings (key, value) VALUES (?1, '1')"
        } else {
            "DELETE FROM settings WHERE key = ?1"
        };
        self.conn.execute(sql, params![ALL_SKILLS_EVERYONE])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::Fixture;

    #[test]
    fn a_skill_goes_to_the_agents_chosen_in_place_of_those_before() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        fx.store
            .set_skill_bots("deploy", &[scout.clone(), writer.clone(), scout.clone()])
            .expect("set");
        fx.store
            .set_skill_bots("review", std::slice::from_ref(writer))
            .expect("set");
        assert_eq!(
            fx.store.bot_skills(writer).expect("list"),
            ["deploy", "review"]
        );
        assert_eq!(fx.store.skill_bots("deploy").expect("bots").len(), 2);
        fx.store.set_skill_bots("deploy", &[]).expect("none");
        assert!(fx.store.bot_skills(scout).expect("none").is_empty());
        assert_eq!(fx.store.given_skills().expect("given"), ["review"]);
    }

    #[test]
    fn a_skill_or_all_of_them_go_to_everyone_and_come_back() {
        let fx = Fixture::new();
        assert!(fx.store.everyone_skills().expect("none").is_empty());
        fx.store.set_skill_everyone("review", true).expect("on");
        fx.store.set_skill_everyone("deploy", true).expect("on");
        fx.store.set_skill_everyone("deploy", true).expect("twice");
        assert_eq!(
            fx.store.everyone_skills().expect("list"),
            ["deploy", "review"]
        );
        fx.store.set_skill_everyone("review", false).expect("off");
        assert_eq!(fx.store.everyone_skills().expect("list"), ["deploy"]);

        assert!(!fx.store.all_skills_everyone().expect("off"));
        fx.store.set_all_skills_everyone(true).expect("on");
        fx.store.set_all_skills_everyone(true).expect("twice");
        assert!(fx.store.all_skills_everyone().expect("on"));
        fx.store.set_all_skills_everyone(false).expect("off");
        assert!(!fx.store.all_skills_everyone().expect("off"));
    }
}
