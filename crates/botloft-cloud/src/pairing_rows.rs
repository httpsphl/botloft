//! Queries for connecting a phone (spec 28.3).

use rusqlite::{Connection, OptionalExtension, params};

/// How long a QR code stays good.
pub const PAIR_TTL_MS: i64 = 300_000;
/// How many QR codes one computer may have open at once.
pub const OPEN_MAX: i64 = 3;

/// What the computer finds when it looks at its pairing.
pub enum Seen {
    Waiting,
    Joined {
        phone_name: String,
        phone_pub: String,
        proof: String,
    },
}

/// False when the computer already has too many open, or the id is taken.
pub fn open(
    conn: &Connection,
    id: &str,
    account_id: i64,
    computer: &str,
    now: i64,
) -> rusqlite::Result<bool> {
    conn.execute("DELETE FROM pairings WHERE expires_at <= ?1", [now])?;
    let open: i64 = conn.query_row(
        "SELECT COUNT(*) FROM pairings WHERE computer = ?1",
        [computer],
        |row| row.get(0),
    )?;
    if open >= OPEN_MAX {
        return Ok(false);
    }
    let added = conn.execute(
        "INSERT OR IGNORE INTO pairings (id, account_id, computer, created_at, expires_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, account_id, computer, now, now + PAIR_TTL_MS],
    )?;
    Ok(added > 0)
}

pub struct Join<'a> {
    pub phone_pub: &'a str,
    pub phone_name: &'a str,
    pub proof: &'a str,
    pub poll_hash: &'a str,
}

/// The phone joins an open, unjoined pairing. Says which computer to tell, or
/// nothing when the code is unknown, expired or already used.
pub fn join(
    conn: &Connection,
    id: &str,
    join: &Join,
    now: i64,
) -> rusqlite::Result<Option<String>> {
    let computer: Option<String> = conn
        .query_row(
            "SELECT computer FROM pairings \
             WHERE id = ?1 AND expires_at > ?2 AND phone_pub IS NULL",
            params![id, now],
            |row| row.get(0),
        )
        .optional()?;
    let Some(computer) = computer else {
        return Ok(None);
    };
    conn.execute(
        "UPDATE pairings SET phone_pub = ?2, phone_name = ?3, proof = ?4, poll_hash = ?5 \
         WHERE id = ?1",
        params![
            id,
            join.phone_pub,
            join.phone_name,
            join.proof,
            join.poll_hash
        ],
    )?;
    Ok(Some(computer))
}

/// What the computer's own pairing looks like. `None`: unknown or expired.
pub fn seen(
    conn: &Connection,
    id: &str,
    computer: &str,
    now: i64,
) -> rusqlite::Result<Option<Seen>> {
    conn.query_row(
        "SELECT phone_name, phone_pub, proof FROM pairings \
         WHERE id = ?1 AND computer = ?2 AND expires_at > ?3",
        params![id, computer, now],
        |row| {
            Ok(match row.get::<_, Option<String>>(1)? {
                None => Seen::Waiting,
                Some(phone_pub) => Seen::Joined {
                    phone_name: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    phone_pub,
                    proof: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                },
            })
        },
    )
    .optional()
}

/// The computer accepts the phone that joined. False when there is nothing to
/// accept (no such pairing, nobody joined, or already accepted).
pub fn accept(
    conn: &Connection,
    id: &str,
    computer: &str,
    daemon_pub: &str,
    proof2: &str,
    now: i64,
) -> rusqlite::Result<bool> {
    let changed = conn.execute(
        "UPDATE pairings SET daemon_pub = ?3, proof2 = ?4 \
         WHERE id = ?1 AND computer = ?2 AND expires_at > ?5 \
           AND phone_pub IS NOT NULL AND daemon_pub IS NULL",
        params![id, computer, daemon_pub, proof2, now],
    )?;
    Ok(changed > 0)
}

/// The computer cancels, or refuses the phone that joined.
pub fn remove(conn: &Connection, id: &str, computer: &str) -> rusqlite::Result<bool> {
    let removed = conn.execute(
        "DELETE FROM pairings WHERE id = ?1 AND computer = ?2",
        params![id, computer],
    )?;
    Ok(removed > 0)
}

/// What the phone's poll finds.
pub enum Collected {
    /// Unknown, wrong poll secret, expired, cancelled or already collected.
    Gone,
    Pending,
    Ready(Ready),
}

pub struct Ready {
    pub account_id: i64,
    pub computer: String,
    pub phone_name: String,
    pub daemon_pub: String,
    pub proof2: String,
}

/// Looks at the pairing by its id and the hash of the phone's poll secret.
pub fn collect(
    conn: &Connection,
    id: &str,
    poll_hash: &str,
    now: i64,
) -> rusqlite::Result<Collected> {
    let row = conn
        .query_row(
            "SELECT account_id, computer, phone_name, daemon_pub, proof2 FROM pairings \
             WHERE id = ?1 AND poll_hash = ?2 AND expires_at > ?3",
            params![id, poll_hash, now],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                ))
            },
        )
        .optional()?;
    Ok(match row {
        None => Collected::Gone,
        Some((account_id, computer, name, Some(daemon_pub), Some(proof2))) => {
            Collected::Ready(Ready {
                account_id,
                computer,
                phone_name: name.unwrap_or_default(),
                daemon_pub,
                proof2,
            })
        }
        Some(_) => Collected::Pending,
    })
}

/// The phone collected its answer: a device with the hash of its new token,
/// and the pairing is gone, so the token is handed out once. False if someone
/// else collected it first.
pub fn finish(
    conn: &mut Connection,
    id: &str,
    ready: &Ready,
    device_id: &str,
    token_hash: &str,
    now: i64,
) -> rusqlite::Result<bool> {
    let tx = conn.transaction()?;
    let removed = tx.execute("DELETE FROM pairings WHERE id = ?1", [id])?;
    if removed == 0 {
        return Ok(false);
    }
    tx.execute(
        "INSERT INTO devices (id, account_id, name, token_hash, created_at, last_used_at, \
         kind, peer) VALUES (?1, ?2, ?3, ?4, ?5, ?5, 'phone', ?6)",
        params![
            device_id,
            ready.account_id,
            ready.phone_name,
            token_hash,
            now,
            ready.computer
        ],
    )?;
    tx.commit()?;
    Ok(true)
}
