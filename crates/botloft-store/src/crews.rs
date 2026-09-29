//! `crews` table.

use botloft_core::ids::CrewId;
use botloft_core::protocol::Crew;
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column, unique_as_duplicate};

const COLUMNS: &str = "id, name, slug, paused, created_at, archived_at, work_dir";

/// A crew without a chosen folder comes back with an empty `work_folder`;
/// the daemon fills in the `shared` folder, which depends on its paths.
fn from_row(row: &Row<'_>) -> rusqlite::Result<Crew> {
    let work_dir: Option<String> = row.get(6)?;
    Ok(Crew {
        id: parse_column(row, 0)?,
        name: row.get(1)?,
        slug: row.get(2)?,
        work_folder_chosen: work_dir.is_some(),
        work_folder: work_dir.unwrap_or_default(),
        paused: row.get(3)?,
        created_at: row.get(4)?,
        archived_at: row.get(5)?,
    })
}

fn work_dir(crew: &Crew) -> Option<&str> {
    crew.work_folder_chosen.then_some(crew.work_folder.as_str())
}

impl Store {
    pub fn insert_crew(&self, crew: &Crew) -> Result<()> {
        self.conn
            .execute(
                &format!("INSERT INTO crews ({COLUMNS}) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"),
                params![
                    crew.id.as_str(),
                    crew.name,
                    crew.slug,
                    crew.paused,
                    crew.created_at,
                    crew.archived_at,
                    work_dir(crew)
                ],
            )
            .map_err(|err| unique_as_duplicate(err, "crew slug"))?;
        Ok(())
    }

    pub fn crew(&self, id: &CrewId) -> Result<Option<Crew>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM crews WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Crews in creation order.
    pub fn crews(&self, include_archived: bool) -> Result<Vec<Crew>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COLUMNS} FROM crews WHERE ?1 OR archived_at IS NULL ORDER BY created_at, id"
        ))?;
        let rows = stmt.query_map([include_archived], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Whether any crew, archived or not, already uses `slug`.
    pub fn crew_slug_exists(&self, slug: &str) -> Result<bool> {
        Ok(self.conn.query_row(
            "SELECT EXISTS (SELECT 1 FROM crews WHERE slug = ?1)",
            [slug],
            |row| row.get(0),
        )?)
    }

    /// Saves the mutable fields: name, paused, archived_at and the chosen
    /// work folder.
    pub fn update_crew(&self, crew: &Crew) -> Result<()> {
        self.conn.execute(
            "UPDATE crews SET name = ?2, paused = ?3, archived_at = ?4, work_dir = ?5 WHERE id = ?1",
            params![
                crew.id.as_str(),
                crew.name,
                crew.paused,
                crew.archived_at,
                work_dir(crew)
            ],
        )?;
        Ok(())
    }

    /// Archives the crew and every active bot in it, atomically.
    pub fn archive_crew(&self, id: &CrewId, at: i64) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;
        tx.execute(
            "UPDATE crews SET archived_at = ?2 WHERE id = ?1 AND archived_at IS NULL",
            params![id.as_str(), at],
        )?;
        tx.execute(
            "UPDATE bots SET archived_at = ?2 WHERE crew_id = ?1 AND archived_at IS NULL",
            params![id.as_str(), at],
        )?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::StoreError;

    pub(crate) fn crew(slug: &str, created_at: i64) -> Crew {
        Crew {
            id: CrewId::generate(),
            name: slug.to_uppercase(),
            slug: slug.to_owned(),
            work_folder: String::new(),
            work_folder_chosen: false,
            paused: false,
            created_at,
            archived_at: None,
        }
    }

    #[test]
    fn insert_get_and_list_in_creation_order() {
        let store = Store::open_in_memory().expect("store");
        let second = crew("second", 20);
        let first = crew("first", 10);
        store.insert_crew(&second).expect("insert");
        store.insert_crew(&first).expect("insert");

        assert_eq!(store.crew(&first.id).expect("get"), Some(first.clone()));
        assert_eq!(store.crew(&CrewId::generate()).expect("get"), None);
        assert_eq!(store.crews(false).expect("list"), vec![first, second]);
    }

    #[test]
    fn slugs_are_unique_even_after_archiving() {
        let store = Store::open_in_memory().expect("store");
        let docs = crew("docs", 1);
        store.insert_crew(&docs).expect("insert");
        store.archive_crew(&docs.id, 2).expect("archive");

        assert!(store.crew_slug_exists("docs").expect("exists"));
        assert!(!store.crew_slug_exists("other").expect("exists"));
        let clash = store.insert_crew(&crew("docs", 3));
        assert!(matches!(clash, Err(StoreError::Duplicate(_))));
    }

    #[test]
    fn update_and_archive_filter_the_listing() {
        let store = Store::open_in_memory().expect("store");
        let mut docs = crew("docs", 1);
        store.insert_crew(&docs).expect("insert");
        docs.name = "Docs team".to_owned();
        docs.paused = true;
        store.update_crew(&docs).expect("update");
        assert_eq!(store.crew(&docs.id).expect("get"), Some(docs.clone()));

        store.archive_crew(&docs.id, 5).expect("archive");
        assert!(store.crews(false).expect("list").is_empty());
        let all = store.crews(true).expect("list all");
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].archived_at, Some(5));
    }
}
