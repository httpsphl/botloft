//! Rate limits of sign-in (spec 27.3): so many tries per e-mail and per
//! address in an hour, and a gap between two e-mails to the same person.

use rusqlite::{Connection, params};

pub struct Rule {
    pub kind: &'static str,
    pub limit: u32,
    pub window_ms: i64,
}

pub const PER_EMAIL: Rule = Rule {
    kind: "email",
    limit: 5,
    window_ms: 3_600_000,
};
pub const PER_ADDRESS: Rule = Rule {
    kind: "address",
    limit: 20,
    window_ms: 3_600_000,
};
/// Phones joining a pairing (spec 28.3), per address.
pub const PAIR_PER_ADDRESS: Rule = Rule {
    kind: "pair",
    limit: 30,
    window_ms: 3_600_000,
};
pub const GAP: Rule = Rule {
    kind: "gap",
    limit: 1,
    window_ms: 60_000,
};

/// Counts the tries of every `(rule, key)` and, if none is over, records one
/// more for each. When one is over, nothing is recorded and the answer is how
/// many seconds until it frees up. `key` is hashed by the caller.
pub fn take(
    conn: &mut Connection,
    now: i64,
    checks: &[(&Rule, &str)],
) -> rusqlite::Result<Result<(), u64>> {
    let tx = conn.transaction()?;
    let mut wait: Option<i64> = None;
    for (rule, key) in checks {
        let since = now - rule.window_ms;
        let (count, oldest): (u32, Option<i64>) = tx.query_row(
            "SELECT COUNT(*), MIN(at) FROM attempts WHERE kind = ?1 AND key = ?2 AND at > ?3",
            params![rule.kind, key, since],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        if count >= rule.limit {
            let free = oldest.map_or(rule.window_ms, |at| at + rule.window_ms - now);
            wait = Some(wait.map_or(free, |other| other.max(free)));
        }
    }
    if let Some(ms) = wait {
        return Ok(Err(((ms + 999) / 1000).max(1) as u64));
    }
    for (rule, key) in checks {
        tx.execute(
            "INSERT INTO attempts (kind, key, at) VALUES (?1, ?2, ?3)",
            params![rule.kind, key, now],
        )?;
    }
    tx.commit()?;
    Ok(Ok(()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Db;

    #[test]
    fn the_sixth_try_in_an_hour_waits_for_the_first_to_age() {
        let db = Db::memory().expect("db");
        for second in 0..5 {
            let now = second * 1000;
            let taken = db
                .run(|conn| take(conn, now, &[(&PER_EMAIL, "k")]))
                .expect("take");
            assert_eq!(taken, Ok(()));
        }
        let taken = db
            .run(|conn| take(conn, 10_000, &[(&PER_EMAIL, "k")]))
            .expect("take");
        assert_eq!(taken, Err(3590));
        // Another key is not held back, and an hour later the first is free.
        let other = db
            .run(|conn| take(conn, 10_000, &[(&PER_EMAIL, "z")]))
            .expect("take");
        assert_eq!(other, Ok(()));
        let later = db
            .run(|conn| take(conn, 3_601_000, &[(&PER_EMAIL, "k")]))
            .expect("take");
        assert_eq!(later, Ok(()));
    }

    #[test]
    fn a_refused_try_records_nothing() {
        let db = Db::memory().expect("db");
        let first = db.run(|conn| take(conn, 0, &[(&GAP, "k")])).expect("take");
        assert_eq!(first, Ok(()));
        // The gap holds, so the address count must not move either.
        let refused = db
            .run(|conn| take(conn, 1000, &[(&PER_ADDRESS, "a"), (&GAP, "k")]))
            .expect("take");
        assert_eq!(refused, Err(59));
        let count: u32 = db
            .run(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM attempts WHERE kind = 'address'",
                    [],
                    |row| row.get(0),
                )
            })
            .expect("count");
        assert_eq!(count, 0);
    }
}
