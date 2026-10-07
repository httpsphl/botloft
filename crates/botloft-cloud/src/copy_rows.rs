//! Queries for the copies (spec 27.4).

use rusqlite::{Connection, OptionalExtension, params};

pub struct Copy {
    pub id: String,
    pub size: i64,
    pub sha256: String,
    pub created_at: i64,
}

/// Where the bytes of a copy are kept.
pub fn blob_key(account_id: i64, id: &str) -> String {
    format!("{account_id}/{id}")
}

pub fn used(conn: &Connection, account_id: i64) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(size), 0) FROM copies WHERE account_id = ?1",
        [account_id],
        |row| row.get(0),
    )
}

/// Newest first.
pub fn list(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<Copy>> {
    let mut stmt = conn.prepare(
        "SELECT id, size, sha256, created_at FROM copies WHERE account_id = ?1 \
         ORDER BY created_at DESC, rowid DESC",
    )?;
    stmt.query_map([account_id], row)?.collect()
}

fn row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Copy> {
    Ok(Copy {
        id: row.get(0)?,
        size: row.get(1)?,
        sha256: row.get(2)?,
        created_at: row.get(3)?,
    })
}

pub fn find(conn: &Connection, account_id: i64, id: &str) -> rusqlite::Result<Option<Copy>> {
    conn.query_row(
        "SELECT id, size, sha256, created_at FROM copies WHERE id = ?1 AND account_id = ?2",
        params![id, account_id],
        row,
    )
    .optional()
}

/// False when the account has no such copy.
pub fn delete(conn: &Connection, account_id: i64, id: &str) -> rusqlite::Result<bool> {
    let removed = conn.execute(
        "DELETE FROM copies WHERE id = ?1 AND account_id = ?2",
        params![id, account_id],
    )?;
    Ok(removed > 0)
}

/// Whether a copy of `size` bytes fits: what the account holds, less the
/// oldest copies that the new one pushes past `keep`, plus the new one.
pub fn fits(
    conn: &Connection,
    account_id: i64,
    keep: u32,
    size: u64,
    quota: u64,
) -> rusqlite::Result<bool> {
    let held = used(conn, account_id)?;
    let freed: i64 = list(conn, account_id)?
        .iter()
        .skip(keep.saturating_sub(1) as usize)
        .map(|copy| copy.size)
        .sum();
    Ok(held - freed + size as i64 <= quota as i64)
}

/// Records the new copy and drops the oldest past `keep`; the keys of those
/// are what the caller removes from the store.
pub fn add_and_prune(
    conn: &mut Connection,
    account_id: i64,
    copy: &Copy,
    keep: u32,
) -> rusqlite::Result<Vec<String>> {
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO copies (id, account_id, size, sha256, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![copy.id, account_id, copy.size, copy.sha256, copy.created_at],
    )?;
    let old: Vec<String> = list(&tx, account_id)?
        .into_iter()
        .skip(keep.max(1) as usize)
        .map(|copy| copy.id)
        .collect();
    for id in &old {
        tx.execute("DELETE FROM copies WHERE id = ?1", [id])?;
    }
    tx.commit()?;
    Ok(old.iter().map(|id| blob_key(account_id, id)).collect())
}

/// Every key the account's copies use.
pub fn keys(conn: &Connection, account_id: i64) -> rusqlite::Result<Vec<String>> {
    Ok(list(conn, account_id)?
        .iter()
        .map(|copy| blob_key(account_id, &copy.id))
        .collect())
}
