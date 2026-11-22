//! `devices` table: what reaches the daemon with a token of its own (spec
//! 42.6). Only the SHA-256 of each token is kept, in hex.

use botloft_core::ids::DeviceId;
use botloft_core::protocol::Device;
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

fn from_row(row: &Row<'_>) -> rusqlite::Result<Device> {
    Ok(Device {
        id: parse_column(row, 0)?,
        name: row.get(1)?,
        paired_at: row.get(2)?,
        last_seen_at: row.get(3)?,
    })
}

impl Store {
    /// A new device, known from now on by the hash of its token.
    pub fn insert_device(&self, name: &str, token_hash: &str, now: i64) -> Result<Device> {
        let id = DeviceId::generate();
        self.conn.execute(
            "INSERT INTO devices (id, name, token_hash, paired_at) VALUES (?1, ?2, ?3, ?4)",
            params![id.as_str(), name, token_hash, now],
        )?;
        Ok(Device {
            id,
            name: name.to_owned(),
            paired_at: now,
            last_seen_at: None,
        })
    }

    /// Every device, the newest first.
    pub fn devices(&self) -> Result<Vec<Device>> {
        let mut statement = self.conn.prepare_cached(
            "SELECT id, name, paired_at, last_seen_at FROM devices \
             ORDER BY paired_at DESC, rowid DESC",
        )?;
        let devices = statement
            .query_map([], from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(devices)
    }

    /// The device whose token hashes to `token_hash`, now seen at `now`;
    /// `None` when no device has that token.
    pub fn device_seen(&self, token_hash: &str, now: i64) -> Result<Option<DeviceId>> {
        Ok(self
            .conn
            .query_row(
                "UPDATE devices SET last_seen_at = ?2 WHERE token_hash = ?1 RETURNING id",
                params![token_hash, now],
                |row| parse_column(row, 0),
            )
            .optional()?)
    }

    /// Forgets a device and its token; false when there was none.
    pub fn delete_device(&self, id: &DeviceId) -> Result<bool> {
        let deleted = self
            .conn
            .execute("DELETE FROM devices WHERE id = ?1", [id.as_str()])?;
        Ok(deleted > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("dir");
        let store = Store::open(&dir.path().join("botloft.db")).expect("store");
        (dir, store)
    }

    #[test]
    fn a_device_is_found_by_its_token_hash_and_seen() {
        let (_dir, store) = store();
        let laptop = store.insert_device("Laptop", "aa", 10).expect("insert");
        let phone = store.insert_device("Tablet", "bb", 20).expect("insert");
        assert_eq!(
            store.device_seen("aa", 30).expect("seen"),
            Some(laptop.id.clone())
        );
        assert_eq!(store.device_seen("cc", 30).expect("seen"), None);
        let devices = store.devices().expect("list");
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[0].id, phone.id, "the newest first");
        assert_eq!(devices[1].last_seen_at, Some(30));
        assert_eq!(devices[0].last_seen_at, None);
    }

    #[test]
    fn a_deleted_device_is_gone_with_its_token() {
        let (_dir, store) = store();
        let laptop = store.insert_device("Laptop", "aa", 10).expect("insert");
        assert!(store.delete_device(&laptop.id).expect("delete"));
        assert!(!store.delete_device(&laptop.id).expect("again"));
        assert_eq!(store.device_seen("aa", 30).expect("seen"), None);
        assert!(store.devices().expect("list").is_empty());
    }

    #[test]
    fn two_devices_never_share_a_token() {
        let (_dir, store) = store();
        store.insert_device("Laptop", "aa", 10).expect("insert");
        assert!(store.insert_device("Copy", "aa", 20).is_err());
    }
}
