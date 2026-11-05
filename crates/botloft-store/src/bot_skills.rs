//! Which of the owner's skills each agent may use (spec 38.3), by the name
//! of the skill's folder.

use botloft_core::ids::BotId;
use rusqlite::params;

use crate::{Result, Store, parse_column};

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
}
