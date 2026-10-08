//! Queries for logins, accounts and devices (spec 27.3).

use rusqlite::{Connection, OptionalExtension, params};

/// How long a sign-in link and its request live.
pub const LOGIN_TTL_MS: i64 = 600_000;

pub struct NewLogin<'a> {
    pub request_hash: &'a str,
    pub code_hash: &'a str,
    pub email: &'a str,
    pub device_name: &'a str,
    pub locale: &'a str,
}

pub fn insert_login(conn: &Connection, login: &NewLogin, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO logins (request_hash, code_hash, email, device_name, locale, created_at, \
         expires_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            login.request_hash,
            login.code_hash,
            login.email,
            login.device_name,
            login.locale,
            now,
            now + LOGIN_TTL_MS
        ],
    )?;
    Ok(())
}

pub fn drop_login(conn: &Connection, request_hash: &str) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM logins WHERE request_hash = ?1", [request_hash])?;
    Ok(())
}

/// What the page behind a link shows before the owner confirms.
pub struct Waiting {
    pub device_name: String,
    pub locale: String,
}

/// A link that is still good: not used, not expired.
pub fn peek_login(
    conn: &Connection,
    code_hash: &str,
    now: i64,
) -> rusqlite::Result<Option<Waiting>> {
    conn.query_row(
        "SELECT device_name, locale FROM logins \
         WHERE code_hash = ?1 AND expires_at > ?2 AND account_id IS NULL",
        params![code_hash, now],
        |row| {
            Ok(Waiting {
                device_name: row.get(0)?,
                locale: row.get(1)?,
            })
        },
    )
    .optional()
}

/// The link was confirmed: the account exists now (born here, if new) and the
/// request waits for the app to collect its device. False if the link is bad.
pub fn confirm_login(conn: &mut Connection, code_hash: &str, now: i64) -> rusqlite::Result<bool> {
    let tx = conn.transaction()?;
    let email: Option<String> = tx
        .query_row(
            "SELECT email FROM logins \
             WHERE code_hash = ?1 AND expires_at > ?2 AND account_id IS NULL",
            params![code_hash, now],
            |row| row.get(0),
        )
        .optional()?;
    let Some(email) = email else {
        return Ok(false);
    };
    tx.execute(
        "INSERT INTO accounts (email, created_at) VALUES (?1, ?2) ON CONFLICT(email) DO NOTHING",
        params![email, now],
    )?;
    let account: i64 = tx.query_row(
        "SELECT id FROM accounts WHERE email = ?1",
        [&email],
        |row| row.get(0),
    )?;
    tx.execute(
        "UPDATE logins SET account_id = ?1 WHERE code_hash = ?2",
        params![account, code_hash],
    )?;
    tx.commit()?;
    Ok(true)
}

/// What a poll of a request finds.
pub enum Request {
    /// No such request, or it ran out.
    Gone,
    Pending,
    Approved(Approved),
}

pub struct Approved {
    pub account_id: i64,
    pub email: String,
    pub device_name: String,
}

pub fn read_request(conn: &Connection, request_hash: &str, now: i64) -> rusqlite::Result<Request> {
    let row = conn
        .query_row(
            "SELECT logins.expires_at, logins.account_id, logins.email, logins.device_name \
             FROM logins WHERE request_hash = ?1",
            [request_hash],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                ))
            },
        )
        .optional()?;
    Ok(match row {
        Some((expires, _, _, _)) if expires <= now => Request::Gone,
        Some((_, None, _, _)) => Request::Pending,
        Some((_, Some(account_id), email, device_name)) => Request::Approved(Approved {
            account_id,
            email,
            device_name,
        }),
        None => Request::Gone,
    })
}

/// The app collected its sign-in: a device with the hash of its new token, and
/// the request is gone, so the token is handed out once.
pub fn finish_login(
    conn: &mut Connection,
    request_hash: &str,
    approved: &Approved,
    device_id: &str,
    token_hash: &str,
    now: i64,
) -> rusqlite::Result<bool> {
    let tx = conn.transaction()?;
    let removed = tx.execute(
        "DELETE FROM logins WHERE request_hash = ?1 AND account_id IS NOT NULL",
        [request_hash],
    )?;
    if removed == 0 {
        return Ok(false);
    }
    tx.execute(
        "INSERT INTO devices (id, account_id, name, token_hash, created_at, last_used_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
        params![
            device_id,
            approved.account_id,
            approved.device_name,
            token_hash,
            now
        ],
    )?;
    tx.commit()?;
    Ok(true)
}

/// Drops what ran out: links, phones being connected, messages nobody took,
/// and the old counts of the rate limits.
pub fn purge(conn: &Connection, now: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM logins WHERE expires_at <= ?1", [now])?;
    conn.execute("DELETE FROM attempts WHERE at <= ?1", [now - 3_600_000])?;
    conn.execute("DELETE FROM deletions WHERE expires_at <= ?1", [now])?;
    conn.execute("DELETE FROM pairings WHERE expires_at <= ?1", [now])?;
    conn.execute("DELETE FROM relay_queue WHERE expires_at <= ?1", [now])?;
    Ok(())
}
