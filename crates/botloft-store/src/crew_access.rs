//! `crew_access` table: what the owner let a bot reach in another crew for
//! good (spec 10.4).

use botloft_core::ids::{BotId, CrewAccessId, CrewId};
use rusqlite::{Row, params};

use crate::{Result, Store, parse_column};

/// A bot's lasting access to another crew, or to one bot of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrewAccessRecord {
    pub id: CrewAccessId,
    /// The bot that asked.
    pub bot_id: BotId,
    pub crew_id: CrewId,
    /// One bot of the crew; `None` for the whole crew.
    pub target_bot_id: Option<BotId>,
    /// Talking with its bots: roster, messages and tasks.
    pub talk: bool,
    pub created_at: i64,
}

const COLUMNS: &str = "id, bot_id, crew_id, target_bot_id, talk, created_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<CrewAccessRecord> {
    let target: Option<String> = row.get(3)?;
    Ok(CrewAccessRecord {
        id: parse_column(row, 0)?,
        bot_id: parse_column(row, 1)?,
        crew_id: parse_column(row, 2)?,
        target_bot_id: match target {
            Some(_) => Some(parse_column(row, 3)?),
            None => None,
        },
        talk: row.get(4)?,
        created_at: row.get(5)?,
    })
}

impl Store {
    /// Lets `bot` reach `crew`, or only `target` in it. Access already
    /// there keeps what it had and gains what is new.
    pub fn grant_crew_access(
        &self,
        bot: &BotId,
        crew: &CrewId,
        target: Option<&BotId>,
        talk: bool,
        now: i64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO crew_access (id, bot_id, crew_id, target_bot_id, talk, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
             ON CONFLICT (bot_id, crew_id, coalesce(target_bot_id, '')) DO UPDATE SET \
             talk = talk OR excluded.talk",
            params![
                CrewAccessId::generate().as_str(),
                bot.as_str(),
                crew.as_str(),
                target.map(BotId::as_str),
                talk,
                now,
            ],
        )?;
        Ok(())
    }

    /// What `bot` may reach in other crews, oldest first.
    pub fn crew_access(&self, bot: &BotId) -> Result<Vec<CrewAccessRecord>> {
        let mut statement = self.conn.prepare_cached(&format!(
            "SELECT {COLUMNS} FROM crew_access WHERE bot_id = ?1 ORDER BY created_at, id"
        ))?;
        let rows = statement
            .query_map([bot.as_str()], from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(rows)
    }

    /// Takes the access back; `false` if there was none with that id.
    pub fn revoke_crew_access(&self, id: &CrewAccessId) -> Result<bool> {
        Ok(self
            .conn
            .execute("DELETE FROM crew_access WHERE id = ?1", [id.as_str()])?
            > 0)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::ids::CrewId;

    use crate::tests::Fixture;

    #[test]
    fn access_is_kept_per_bot_and_reach_and_only_grows() {
        let fx = Fixture::new();
        let (scout, writer) = (&fx.bots[0].id, &fx.bots[1].id);
        let blog = CrewId::generate();
        fx.store
            .insert_crew(&botloft_core::protocol::Crew {
                id: blog.clone(),
                slug: "blog".into(),
                ..fx.crew.clone()
            })
            .expect("blog");
        fx.store
            .grant_crew_access(scout, &blog, Some(writer), true, 10)
            .expect("bot");
        fx.store
            .grant_crew_access(scout, &blog, None, false, 20)
            .expect("crew");
        fx.store
            .grant_crew_access(scout, &blog, None, true, 30)
            .expect("crew again");
        let access = fx.store.crew_access(scout).expect("list");
        assert_eq!(access.len(), 2);
        assert_eq!(access[0].target_bot_id.as_ref(), Some(writer));
        assert!(access[1].target_bot_id.is_none());
        assert!(access[1].talk);
        assert_eq!(access[1].created_at, 20);
        assert!(fx.store.crew_access(writer).expect("none").is_empty());

        assert!(fx.store.revoke_crew_access(&access[0].id).expect("revoke"));
        assert!(!fx.store.revoke_crew_access(&access[0].id).expect("again"));
        assert_eq!(fx.store.crew_access(scout).expect("list").len(), 1);
    }
}
