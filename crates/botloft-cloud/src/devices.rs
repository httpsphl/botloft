//! Devices behind the tokens (spec 27.3, 28.2): who a token is, the list of an
//! account's devices, and removing one.

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

/// What a device is (spec 28.2): a phone only reaches the relay.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Computer,
    Phone,
}

impl Kind {
    pub fn tag(self) -> &'static str {
        match self {
            Self::Computer => "computer",
            Self::Phone => "phone",
        }
    }

    fn parse(tag: &str) -> Self {
        if tag == "phone" {
            Self::Phone
        } else {
            Self::Computer
        }
    }
}

/// Who a token belongs to.
pub struct Who {
    pub account_id: i64,
    pub device_id: String,
    pub email: String,
    pub kind: Kind,
    /// The computer a phone was connected with.
    pub peer: Option<String>,
}

/// Looks the token up and notes that the device was used, at most once a minute.
pub fn who_is(conn: &Connection, token_hash: &str, now: i64) -> rusqlite::Result<Option<Who>> {
    let who = conn
        .query_row(
            "SELECT devices.account_id, devices.id, accounts.email, devices.kind, devices.peer \
             FROM devices JOIN accounts ON accounts.id = devices.account_id \
             WHERE devices.token_hash = ?1",
            [token_hash],
            |row| {
                Ok(Who {
                    account_id: row.get(0)?,
                    device_id: row.get(1)?,
                    email: row.get(2)?,
                    kind: Kind::parse(&row.get::<_, String>(3)?),
                    peer: row.get(4)?,
                })
            },
        )
        .optional()?;
    if let Some(who) = &who {
        conn.execute(
            "UPDATE devices SET last_used_at = ?1 WHERE id = ?2 AND last_used_at < ?1 - 60000",
            params![now, who.device_id],
        )?;
    }
    Ok(who)
}

#[derive(Serialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub peer: Option<String>,
    pub created_at: i64,
    pub last_used_at: i64,
}

pub fn devices(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<Device>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, kind, peer, created_at, last_used_at FROM devices \
         WHERE account_id = ?1 ORDER BY created_at",
    )?;
    stmt.query_map([account_id], |row| {
        Ok(Device {
            id: row.get(0)?,
            name: row.get(1)?,
            kind: row.get(2)?,
            peer: row.get(3)?,
            created_at: row.get(4)?,
            last_used_at: row.get(5)?,
        })
    })?
    .collect()
}

/// A device that left, and the computer that must hear of it if it was a phone.
pub struct Removed {
    pub id: String,
    pub peer: Option<String>,
}

/// Removes a device and, with a computer, the phones connected to it (the
/// foreign key does it). Says who left; empty when the account has no such
/// device.
pub fn delete_device(
    conn: &mut Connection,
    account_id: i64,
    id: &str,
) -> rusqlite::Result<Vec<Removed>> {
    let tx = conn.transaction()?;
    let mut gone = Vec::new();
    {
        let mut stmt = tx.prepare(
            "SELECT id, peer FROM devices WHERE account_id = ?2 AND (id = ?1 OR peer = ?1)",
        )?;
        let rows = stmt.query_map(params![id, account_id], |row| {
            Ok(Removed {
                id: row.get(0)?,
                peer: row.get(1)?,
            })
        })?;
        for row in rows {
            gone.push(row?);
        }
    }
    if !gone.iter().any(|device| device.id == id) {
        return Ok(Vec::new());
    }
    tx.execute(
        "DELETE FROM devices WHERE id = ?1 AND account_id = ?2",
        params![id, account_id],
    )?;
    tx.commit()?;
    Ok(gone)
}

/// Every device of an account, for closing their sockets before it goes.
pub fn device_ids(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM devices WHERE account_id = ?1")?;
    stmt.query_map([account_id], |row| row.get(0))?.collect()
}
