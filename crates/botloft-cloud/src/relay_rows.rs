//! The queue of the relay (spec 28.4): sealed messages a phone sent that its
//! computer has not acknowledged. The server never reads a `body`.

use rusqlite::{Connection, params};

/// How long a message waits for a computer that is offline.
pub const QUEUE_TTL_MS: i64 = 3_600_000;
/// How many unacknowledged messages one phone may have waiting.
pub const QUEUE_MAX: i64 = 50;

pub enum Pushed {
    Queued,
    /// The same `seq` was already queued: nothing new, and not an error.
    Again,
    Full,
}

pub fn push(
    conn: &Connection,
    from: &str,
    to: &str,
    seq: i64,
    body: &str,
    now: i64,
) -> rusqlite::Result<Pushed> {
    conn.execute(
        "DELETE FROM relay_queue WHERE from_device = ?1 AND expires_at <= ?2",
        params![from, now],
    )?;
    let waiting: i64 = conn.query_row(
        "SELECT COUNT(*) FROM relay_queue WHERE from_device = ?1",
        [from],
        |row| row.get(0),
    )?;
    if waiting >= QUEUE_MAX {
        return Ok(Pushed::Full);
    }
    let added = conn.execute(
        "INSERT OR IGNORE INTO relay_queue (from_device, to_device, seq, body, expires_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![from, to, seq, body, now + QUEUE_TTL_MS],
    )?;
    Ok(if added == 0 {
        Pushed::Again
    } else {
        Pushed::Queued
    })
}

/// What the computer has not acknowledged, oldest first per phone.
pub fn waiting(
    conn: &Connection,
    to: &str,
    now: i64,
) -> rusqlite::Result<Vec<(String, i64, String)>> {
    let mut stmt = conn.prepare(
        "SELECT from_device, seq, body FROM relay_queue \
         WHERE to_device = ?1 AND expires_at > ?2 ORDER BY from_device, seq",
    )?;
    stmt.query_map(params![to, now], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?
    .collect()
}

/// The computer took everything from `from` up to and including `upto`.
pub fn ack(conn: &Connection, to: &str, from: &str, upto: i64) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM relay_queue WHERE to_device = ?1 AND from_device = ?2 AND seq <= ?3",
        params![to, from, upto],
    )?;
    Ok(())
}

/// The phones connected to a computer, for presence and for where it may send.
pub fn phones_of(conn: &Connection, computer: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM devices WHERE peer = ?1 ORDER BY created_at")?;
    stmt.query_map([computer], |row| row.get(0))?.collect()
}

/// True when `phone` is a phone connected with `computer`.
pub fn is_phone_of(conn: &Connection, phone: &str, computer: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT COUNT(*) FROM devices WHERE id = ?1 AND peer = ?2 AND kind = 'phone'",
        params![phone, computer],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count > 0)
}
