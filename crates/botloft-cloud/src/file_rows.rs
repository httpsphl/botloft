//! Queries for the files phones send (spec 28.14). The server never opens
//! one: what it keeps was sealed on the phone.

use rusqlite::{Connection, OptionalExtension, params};

/// How long a file waits for its computer.
pub const KEEP_MS: i64 = 2 * 3_600_000;
/// How many files one phone may have waiting.
pub const WAITING_MAX: i64 = 20;

pub struct NewFile<'a> {
    pub id: &'a str,
    pub account_id: i64,
    pub phone: &'a str,
    pub computer: &'a str,
    pub size: i64,
    pub sha256: &'a str,
    pub now: i64,
}

pub struct Waiting {
    pub account_id: i64,
    pub size: i64,
    pub sha256: String,
}

/// Where the sealed bytes of a file are kept, apart from the copies.
pub fn blob_key(account_id: i64, id: &str) -> String {
    format!("{account_id}/files/{id}")
}

/// Whether the phone may leave one more file waiting.
pub fn has_room(conn: &Connection, phone: &str, now: i64) -> rusqlite::Result<bool> {
    let waiting: i64 = conn.query_row(
        "SELECT COUNT(*) FROM phone_files WHERE phone = ?1 AND expires_at > ?2",
        params![phone, now],
        |row| row.get(0),
    )?;
    Ok(waiting < WAITING_MAX)
}

/// Gives back when the file runs out.
pub fn add(conn: &Connection, file: &NewFile<'_>) -> rusqlite::Result<i64> {
    let expires_at = file.now + KEEP_MS;
    conn.execute(
        "INSERT INTO phone_files \
         (id, account_id, phone, computer, size, sha256, created_at, expires_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            file.id,
            file.account_id,
            file.phone,
            file.computer,
            file.size,
            file.sha256,
            file.now,
            expires_at
        ],
    )?;
    Ok(expires_at)
}

/// A file sent to `computer` that has not run out.
pub fn find(
    conn: &Connection,
    computer: &str,
    id: &str,
    now: i64,
) -> rusqlite::Result<Option<Waiting>> {
    conn.query_row(
        "SELECT account_id, size, sha256 FROM phone_files \
         WHERE id = ?1 AND computer = ?2 AND expires_at > ?3",
        params![id, computer, now],
        |row| {
            Ok(Waiting {
                account_id: row.get(0)?,
                size: row.get(1)?,
                sha256: row.get(2)?,
            })
        },
    )
    .optional()
}

/// The computer took a file: its row goes, and its key comes back for the bytes.
pub fn take(conn: &Connection, computer: &str, id: &str) -> rusqlite::Result<Option<String>> {
    let account: Option<i64> = conn
        .query_row(
            "DELETE FROM phone_files WHERE id = ?1 AND computer = ?2 RETURNING account_id",
            params![id, computer],
            |row| row.get(0),
        )
        .optional()?;
    Ok(account.map(|account| blob_key(account, id)))
}

/// The files that ran out leave; their keys come back for the bytes.
pub fn take_expired(conn: &mut Connection, now: i64) -> rusqlite::Result<Vec<String>> {
    let tx = conn.transaction()?;
    let keys = {
        let mut stmt =
            tx.prepare("DELETE FROM phone_files WHERE expires_at <= ?1 RETURNING account_id, id")?;
        let rows = stmt.query_map([now], |row| {
            Ok(blob_key(row.get(0)?, &row.get::<_, String>(1)?))
        })?;
        rows.collect::<rusqlite::Result<Vec<_>>>()?
    };
    tx.commit()?;
    Ok(keys)
}

/// The keys of the files a device sent or was sent: they go with it (a
/// computer's phones go with the computer, and so do their files).
pub fn keys_of_device(
    conn: &Connection,
    account_id: i64,
    device: &str,
) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT id FROM phone_files WHERE account_id = ?1 AND (phone = ?2 OR computer = ?2)",
    )?;
    stmt.query_map(params![account_id, device], |row| {
        Ok(blob_key(account_id, &row.get::<_, String>(0)?))
    })?
    .collect()
}

/// The keys of every file of an account, for when it goes.
pub fn keys_of_account(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM phone_files WHERE account_id = ?1")?;
    stmt.query_map([account_id], |row| {
        Ok(blob_key(account_id, &row.get::<_, String>(0)?))
    })?
    .collect()
}
