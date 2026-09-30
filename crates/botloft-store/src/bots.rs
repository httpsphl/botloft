//! `bots` table.

use botloft_core::ids::{BotId, CrewId};
use botloft_core::protocol::{BotEffort, BotModel, ModelEffort, PermissionMode};
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, cached_execute, cached_row, parse_column, unique_as_duplicate};

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
    pub effort: BotEffort,
    /// What Claude Code last reported; written by
    /// [`Store::set_effort_default`] and left alone by [`Store::update_bot`].
    pub effort_default: Option<ModelEffort>,
    pub created_at: i64,
    pub archived_at: Option<i64>,
}

const COLUMNS: &str = "id, crew_id, name, handle, slug, role, instructions, color, paused, created_at, archived_at, \
     permission_mode, model, model_in_use, effort, effort_default";

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
        effort: parse_column(row, 14)?,
        // A level this build does not know reads as not known.
        effort_default: row
            .get::<_, Option<String>>(15)?
            .and_then(|level| level.parse().ok()),
    })
}

impl Store {
    pub fn insert_bot(&self, bot: &BotRecord) -> Result<()> {
        self.conn
            .execute(
                &format!(
                    "INSERT INTO bots ({COLUMNS}) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"
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
                    bot.model_in_use,
                    bot.effort.as_str(),
                    bot.effort_default.map(ModelEffort::as_str)
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
        let mut stmt = self.conn.prepare_cached(&format!(
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
                 color = ?6, paused = ?7, archived_at = ?8, permission_mode = ?9, model = ?10, \
                 effort = ?11 WHERE id = ?1",
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
                    bot.model.as_str(),
                    bot.effort.as_str()
                ],
            )
            .map_err(|err| unique_as_duplicate(err, "bot handle"))?;
        Ok(())
    }

    /// The model Claude Code reported (spec 7.4).
    pub fn set_model_in_use(&self, id: &BotId, model: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE bots SET model_in_use = ?2 WHERE id = ?1",
            params![id.as_str(), model],
        )?;
        Ok(())
    }

    /// The effort the bot's model uses by itself, as Claude Code reported it
    /// (spec 7.4); `None` when it is not known.
    pub fn set_effort_default(&self, id: &BotId, effort: Option<ModelEffort>) -> Result<()> {
        self.conn.execute(
            "UPDATE bots SET effort_default = ?2 WHERE id = ?1",
            params![id.as_str(), effort.map(ModelEffort::as_str)],
        )?;
        Ok(())
    }

    /// The Claude Code conversation the bot resumes (spec 7.3).
    pub fn session_id(&self, id: &BotId) -> Result<Option<String>> {
        Ok(cached_row(
            &self.conn,
            "SELECT session_id FROM bots WHERE id = ?1",
            [id.as_str()],
            |row| row.get(0),
        )
        .optional()?
        .flatten())
    }

    /// `None` makes the next start a new conversation.
    pub fn set_session_id(&self, id: &BotId, session: Option<&str>) -> Result<()> {
        cached_execute(
            &self.conn,
            "UPDATE bots SET session_id = ?2 WHERE id = ?1",
            params![id.as_str(), session],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests;
