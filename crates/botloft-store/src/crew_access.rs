//! `crew_access` table: what the owner let a bot reach in another crew for
//! good (spec 10.4).

use botloft_core::ids::{BotId, CrewAccessId, CrewId};
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

/// What a bot may do in another crew. Editing files includes reading them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AccessKinds {
    /// Talking with its bots: roster, messages and tasks.
    pub talk: bool,
    /// Listing and reading their files.
    pub read: bool,
    /// Changing and adding files.
    pub edit: bool,
}

impl AccessKinds {
    /// Everything either one allows.
    pub fn with(self, other: Self) -> Self {
        Self {
            talk: self.talk || other.talk,
            read: self.read || other.read,
            edit: self.edit || other.edit,
        }
    }
}

/// A bot's lasting access to another crew, or to one bot of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrewAccessRecord {
    pub id: CrewAccessId,
    /// The bot that asked.
    pub bot_id: BotId,
    pub crew_id: CrewId,
    /// One bot of the crew; `None` for the whole crew.
    pub target_bot_id: Option<BotId>,
    pub kinds: AccessKinds,
    pub created_at: i64,
}

const COLUMNS: &str = "id, bot_id, crew_id, target_bot_id, talk, read, edit, created_at";

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
        kinds: AccessKinds {
            talk: row.get(4)?,
            read: row.get(5)?,
            edit: row.get(6)?,
        },
        created_at: row.get(7)?,
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
        kinds: AccessKinds,
        now: i64,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO crew_access \
             (id, bot_id, crew_id, target_bot_id, talk, read, edit, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8) \
             ON CONFLICT (bot_id, crew_id, coalesce(target_bot_id, '')) DO UPDATE SET \
             talk = talk OR excluded.talk, read = read OR excluded.read, \
             edit = edit OR excluded.edit",
            params![
                CrewAccessId::generate().as_str(),
                bot.as_str(),
                crew.as_str(),
                target.map(BotId::as_str),
                kinds.talk,
                kinds.read,
                kinds.edit,
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

    /// Takes the access back. The bot that had it, or `None` if there was
    /// none with that id.
    pub fn revoke_crew_access(&self, id: &CrewAccessId) -> Result<Option<BotId>> {
        let bot: Option<String> = self
            .conn
            .query_row(
                "DELETE FROM crew_access WHERE id = ?1 RETURNING bot_id",
                [id.as_str()],
                |row| row.get(0),
            )
            .optional()?;
        Ok(bot.and_then(|bot| bot.parse().ok()))
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::ids::CrewId;

    use super::AccessKinds;
    use crate::tests::Fixture;

    const TALK: AccessKinds = AccessKinds {
        talk: true,
        read: false,
        edit: false,
    };

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
            .grant_crew_access(scout, &blog, Some(writer), TALK, 10)
            .expect("bot");
        fx.store
            .grant_crew_access(scout, &blog, None, AccessKinds::default(), 20)
            .expect("crew");
        fx.store
            .grant_crew_access(scout, &blog, None, TALK, 30)
            .expect("crew again");
        let edit = AccessKinds {
            edit: true,
            ..AccessKinds::default()
        };
        fx.store
            .grant_crew_access(scout, &blog, None, edit, 40)
            .expect("and files");
        let access = fx.store.crew_access(scout).expect("list");
        assert_eq!(access.len(), 2);
        assert_eq!(access[0].target_bot_id.as_ref(), Some(writer));
        assert!(access[1].target_bot_id.is_none());
        assert!(access[1].kinds.talk && access[1].kinds.edit && !access[1].kinds.read);
        assert_eq!(access[1].created_at, 20);
        assert!(fx.store.crew_access(writer).expect("none").is_empty());

        let revoked = fx.store.revoke_crew_access(&access[0].id).expect("revoke");
        assert_eq!(revoked.as_ref(), Some(scout));
        assert!(
            fx.store
                .revoke_crew_access(&access[0].id)
                .expect("again")
                .is_none()
        );
        assert_eq!(fx.store.crew_access(scout).expect("list").len(), 1);
    }
}
