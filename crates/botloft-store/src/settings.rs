//! What the owner keeps in the database rather than in `config.toml`: free
//! text that can be long, and that a cloud copy carries (spec 12, 14.2).

use rusqlite::{OptionalExtension, params};

use crate::{Result, Store};

/// The instructions the owner wrote for every agent (spec 7.10).
const OWNER_INSTRUCTIONS: &str = "owner_instructions";

impl Store {
    /// What the owner wrote for every agent; empty when nothing.
    pub fn owner_instructions(&self) -> Result<String> {
        Ok(self.setting(OWNER_INSTRUCTIONS)?.unwrap_or_default())
    }

    /// Saves them; empty text takes them out.
    pub fn set_owner_instructions(&self, text: &str) -> Result<()> {
        if text.is_empty() {
            self.conn.execute(
                "DELETE FROM settings WHERE key = ?1",
                params![OWNER_INSTRUCTIONS],
            )?;
            return Ok(());
        }
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            params![OWNER_INSTRUCTIONS, text],
        )?;
        Ok(())
    }

    pub(crate) fn setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT value FROM settings WHERE key = ?1")?;
        Ok(stmt.query_row(params![key], |row| row.get(0)).optional()?)
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::Fixture;

    #[test]
    fn the_owners_instructions_are_saved_replaced_and_taken_out() {
        let fx = Fixture::new();
        assert_eq!(fx.store.owner_instructions().expect("none"), "");
        fx.store
            .set_owner_instructions("Answer in Portuguese.\nNever push to main.")
            .expect("save");
        fx.store
            .set_owner_instructions("Answer in Portuguese.")
            .expect("replace");
        assert_eq!(
            fx.store.owner_instructions().expect("read"),
            "Answer in Portuguese."
        );
        fx.store.set_owner_instructions("").expect("take out");
        assert_eq!(fx.store.owner_instructions().expect("gone"), "");
    }
}
