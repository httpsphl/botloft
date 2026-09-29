//! `bots` table.

use botloft_core::ids::{BotId, CrewId};
use botloft_core::protocol::{BotModel, PermissionMode};
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column, unique_as_duplicate};

/// A bot as stored. The protocol `Bot` adds runtime state and the workspace
/// path, which the daemon derives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BotRecord {
    pub id: BotId,
    pub crew_id: CrewId,
    pub name: String,
    pub handle: String,
    pub slug: String,
    pub role: String,
    pub instructions: String,
    pub color: String,
    pub paused: bool,
    pub permission_mode: PermissionMode,
    pub model: BotModel,
    /// What Claude Code last reported; written by [`Store::set_model_in_use`]
    /// and left alone by [`Store::update_bot`].
    pub model_in_use: Option<String>,
    pub created_at: i64,
    pub archived_at: Option<i64>,
}

const COLUMNS: &str = "id, crew_id, name, handle, slug, role, instructions, color, paused, created_at, archived_at, \
     permission_mode, model, model_in_use";

fn from_row(row: &Row<'_>) -> rusqlite::Result<BotRecord> {
    Ok(BotRecord {
        id: parse_column(row, 0)?,
        crew_id: parse_column(row, 1)?,
        name: row.get(2)?,
        handle: row.get(3)?,
        slug: row.get(4)?,
        role: row.get(5)?,
        instructions: row.get(6)?,
        color: row.get(7)?,
        paused: row.get(8)?,
        created_at: row.get(9)?,
        archived_at: row.get(10)?,
        permission_mode: parse_column(row, 11)?,
        model: parse_column(row, 12)?,
        model_in_use: row.get(13)?,
    })
}

impl Store {
    pub fn insert_bot(&self, bot: &BotRecord) -> Result<()> {
        self.conn
            .execute(
                &format!(
                    "INSERT INTO bots ({COLUMNS}) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
                ),
                params![
                    bot.id.as_str(),
                    bot.crew_id.as_str(),
                    bot.name,
                    bot.handle,
                    bot.slug,
                    bot.role,
                    bot.instructions,
                    bot.color,
                    bot.paused,
                    bot.created_at,
                    bot.archived_at,
                    bot.permission_mode.as_str(),
                    bot.model.as_str(),
                    bot.model_in_use
                ],
            )
            .map_err(|err| unique_as_duplicate(err, "bot handle or slug"))?;
        Ok(())
    }

    pub fn bot(&self, id: &BotId) -> Result<Option<BotRecord>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM bots WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Bots in creation order, optionally limited to one crew.
    pub fn bots(&self, crew: Option<&CrewId>, include_archived: bool) -> Result<Vec<BotRecord>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM bots \
             WHERE (?1 IS NULL OR crew_id = ?1) AND (?2 OR archived_at IS NULL) \
             ORDER BY created_at, id"
        ))?;
        let rows = stmt.query_map(
            params![crew.map(CrewId::as_str), include_archived],
            from_row,
        )?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Whether any bot of the crew, archived or not, already uses `slug`.
    pub fn bot_slug_exists(&self, crew: &CrewId, slug: &str) -> Result<bool> {
        Ok(self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM bots WHERE crew_id = ?1 AND slug = ?2)",
            params![crew.as_str(), slug],
            |row| row.get(0),
        )?)
    }

    /// The active bot of the crew that answers to `handle`, if any.
    pub fn active_bot_by_handle(&self, crew: &CrewId, handle: &str) -> Result<Option<BotId>> {
        Ok(self
            .conn
            .query_row(
                "SELECT id FROM bots WHERE crew_id = ?1 AND handle = ?2 AND archived_at IS NULL",
                params![crew.as_str(), handle],
                |row| parse_column(row, 0),
            )
            .optional()?)
    }

    /// Bots ever created in the crew, archived ones included.
    pub fn count_bots(&self, crew: &CrewId) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM bots WHERE crew_id = ?1",
            [crew.as_str()],
            |row| row.get(0),
        )?;
        Ok(usize::try_from(count).unwrap_or(0))
    }

    /// Saves every field except the id, crew, slug and creation time.
    pub fn update_bot(&self, bot: &BotRecord) -> Result<()> {
        self.conn
            .execute(
                "UPDATE bots SET name = ?2, handle = ?3, role = ?4, instructions = ?5, \
                 color = ?6, paused = ?7, archived_at = ?8, permission_mode = ?9, model = ?10                  WHERE id = ?1",
                params![
                    bot.id.as_str(),
                    bot.name,
                    bot.handle,
                    bot.role,
                    bot.instructions,
                    bot.color,
                    bot.paused,
                    bot.archived_at,
                    bot.permission_mode.as_str(),
                    bot.model.as_str()
                ],
            )
            .map_err(|err| unique_as_duplicate(err, "bot handle"))?;
        Ok(())
    }

    /// The model Claude Code reported at the start of a turn (spec 7.4).
    pub fn set_model_in_use(&self, id: &BotId, model: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE bots SET model_in_use = ?2 WHERE id = ?1",
            params![id.as_str(), model],
        )?;
        Ok(())
    }

    /// The Claude Code conversation the bot resumes (spec 7.3).
    pub fn session_id(&self, id: &BotId) -> Result<Option<String>> {
        Ok(self
            .conn
            .query_row(
                "SELECT session_id FROM bots WHERE id = ?1",
                [id.as_str()],
                |row| row.get(0),
            )
            .optional()?
            .flatten())
    }

    /// `None` makes the next start a new conversation.
    pub fn set_session_id(&self, id: &BotId, session: Option<&str>) -> Result<()> {
        self.conn.execute(
            "UPDATE bots SET session_id = ?2 WHERE id = ?1",
            params![id.as_str(), session],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::Crew;

    use super::*;
    use crate::StoreError;

    fn setup() -> (Store, CrewId) {
        let store = Store::open_in_memory().expect("store");
        let crew = Crew {
            id: CrewId::generate(),
            name: "Docs".to_owned(),
            slug: "docs".to_owned(),
            work_folder: String::new(),
            work_folder_chosen: false,
            paused: false,
            created_at: 1,
            archived_at: None,
        };
        store.insert_crew(&crew).expect("crew");
        (store, crew.id)
    }

    fn bot(crew: &CrewId, handle: &str, created_at: i64) -> BotRecord {
        BotRecord {
            id: BotId::generate(),
            crew_id: crew.clone(),
            name: handle.to_owned(),
            handle: handle.to_owned(),
            slug: handle.to_owned(),
            role: "writes docs".to_owned(),
            instructions: "Be brief.\nCite sources.".to_owned(),
            color: "#FF7A59".to_owned(),
            paused: false,
            permission_mode: PermissionMode::Default,
            model: BotModel::Default,
            model_in_use: None,
            created_at,
            archived_at: None,
        }
    }

    #[test]
    fn insert_get_and_list_by_crew() {
        let (store, crew) = setup();
        let writer = bot(&crew, "writer", 10);
        let editor = bot(&crew, "editor", 20);
        store.insert_bot(&writer).expect("insert");
        store.insert_bot(&editor).expect("insert");

        assert_eq!(store.bot(&writer.id).expect("get"), Some(writer.clone()));
        let expected = vec![writer, editor];
        assert_eq!(store.bots(Some(&crew), false).expect("list"), expected);
        assert_eq!(store.bots(None, false).expect("list"), expected);
        assert!(
            store
                .bots(Some(&CrewId::generate()), false)
                .expect("list")
                .is_empty()
        );
    }

    #[test]
    fn bot_needs_an_existing_crew() {
        let (store, _) = setup();
        let orphan = store.insert_bot(&bot(&CrewId::generate(), "orphan", 1));
        assert!(matches!(orphan, Err(StoreError::Sqlite(_))), "{orphan:?}");
    }

    #[test]
    fn handles_are_unique_among_active_bots_only() {
        let (store, crew) = setup();
        let mut first = bot(&crew, "writer", 1);
        store.insert_bot(&first).expect("insert");
        assert_eq!(
            store.active_bot_by_handle(&crew, "writer").expect("lookup"),
            Some(first.id.clone())
        );

        let mut clash = bot(&crew, "writer", 2);
        clash.slug = "writer-2".to_owned();
        assert!(matches!(
            store.insert_bot(&clash),
            Err(StoreError::Duplicate(_))
        ));

        first.archived_at = Some(3);
        store.update_bot(&first).expect("archive");
        assert_eq!(
            store.active_bot_by_handle(&crew, "writer").expect("lookup"),
            None
        );
        store.insert_bot(&clash).expect("handle is free again");
        assert!(store.bot_slug_exists(&crew, "writer").expect("slug"));
        assert_eq!(store.count_bots(&crew).expect("count"), 2);
    }

    #[test]
    fn update_saves_mutable_fields() {
        let (store, crew) = setup();
        let mut writer = bot(&crew, "writer", 1);
        store.insert_bot(&writer).expect("insert");
        writer.name = "Lead Writer".to_owned();
        writer.handle = "lead-writer".to_owned();
        writer.color = "#5EC8FF".to_owned();
        writer.paused = true;
        store.update_bot(&writer).expect("update");
        assert_eq!(store.bot(&writer.id).expect("get"), Some(writer));
    }

    #[test]
    fn archiving_the_crew_archives_its_bots() {
        let (store, crew) = setup();
        store.insert_bot(&bot(&crew, "writer", 1)).expect("insert");
        store.archive_crew(&crew, 9).expect("archive");
        assert!(store.bots(Some(&crew), false).expect("list").is_empty());
        let all = store.bots(Some(&crew), true).expect("list all");
        assert_eq!(all[0].archived_at, Some(9));
    }
}
