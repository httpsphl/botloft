//! What every crew did since a moment (spec 31.4), counted from the tables
//! without a bot's turn: the island's "Today" card.

use botloft_core::protocol::{BusyBot, DayActivity, EditLines};
use rusqlite::{OptionalExtension, params};

use crate::{Result, Store, parse_column};

/// A count from SQLite, which keeps integers as `i64`.
fn count(value: i64) -> u32 {
    u32::try_from(value.max(0)).unwrap_or(u32::MAX)
}

impl Store {
    pub fn day_activity(&self, since: i64) -> Result<DayActivity> {
        let (done, failed): (i64, i64) = self.conn.query_row(
            "SELECT COALESCE(SUM(status = 'done'), 0), \
                    COALESCE(SUM(status IN ('failed', 'expired')), 0) \
             FROM tasks WHERE updated_at >= ?1",
            params![since],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let bot_messages: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE from_kind = 'bot' AND created_at >= ?1",
            params![since],
            |row| row.get(0),
        )?;
        let answered: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM approvals \
             WHERE status IN ('allowed', 'denied') AND answered_at >= ?1",
            params![since],
            |row| row.get(0),
        )?;
        let (files, added, removed): (i64, i64, i64) = self.conn.query_row(
            "SELECT COUNT(DISTINCT json_extract(data, '$.file')), \
                    COALESCE(SUM(json_extract(data, '$.lines.added')), 0), \
                    COALESCE(SUM(json_extract(data, '$.lines.removed')), 0) \
             FROM chat_items \
             WHERE kind = 'tool' AND created_at >= ?1 \
               AND json_extract(data, '$.status') = 'done' \
               AND json_type(data, '$.lines') = 'object'",
            params![since],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        let busiest = self
            .conn
            .query_row(
                "SELECT b.id, b.name, b.color, w.name, \
                        SUM(json_extract(c.data, '$.durationMs')) AS work \
                 FROM chat_items c JOIN bots b ON b.id = c.bot_id \
                      JOIN crews w ON w.id = b.crew_id \
                 WHERE c.kind = 'turn' AND c.created_at >= ?1 \
                 GROUP BY b.id HAVING work > 0 \
                 ORDER BY work DESC, b.name LIMIT 1",
                params![since],
                |row| {
                    Ok(BusyBot {
                        bot_id: parse_column(row, 0)?,
                        name: row.get(1)?,
                        color: row.get(2)?,
                        crew: row.get(3)?,
                        work_ms: row.get(4)?,
                    })
                },
            )
            .optional()?;
        Ok(DayActivity {
            tasks_done: count(done),
            tasks_failed: count(failed),
            bot_messages: count(bot_messages),
            requests_answered: count(answered),
            files_edited: count(files),
            lines: EditLines {
                added: count(added),
                removed: count(removed),
            },
            busiest,
        })
    }
}

#[cfg(test)]
mod tests {
    use rusqlite::params;

    use crate::tests::Fixture;

    #[test]
    fn the_day_counts_only_what_happened_since_its_start() {
        let fx = Fixture::new();
        let conn = &fx.store.conn;
        let (crew, scout, writer) = (&fx.crew.id, &fx.bots[0].id, &fx.bots[1].id);
        for (id, status, at) in [
            ("tsk_1", "done", 50),
            ("tsk_2", "failed", 60),
            ("tsk_3", "expired", 70),
            ("tsk_4", "open", 80),
            ("tsk_5", "done", 5),
        ] {
            conn.execute(
                "INSERT INTO tasks (id, crew_id, requester_bot_id, assignee_bot_id, status, \
                 deadline_at, hops, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, 0, 1, 0, ?6)",
                params![id, crew.to_string(), scout.to_string(), writer.to_string(), status, at],
            )
            .expect("task");
        }
        for (id, kind, at) in [
            ("msg_1", "bot", 20),
            ("msg_2", "owner", 30),
            ("msg_3", "bot", 1),
        ] {
            conn.execute(
                "INSERT INTO messages (id, crew_id, from_kind, to_bot_id, kind, body, created_at) \
                 VALUES (?1, ?2, ?3, ?4, 'note', 'x', ?5)",
                params![id, crew.to_string(), kind, writer.to_string(), at],
            )
            .expect("message");
        }
        let day = fx.store.day_activity(10).expect("day");
        assert_eq!((day.tasks_done, day.tasks_failed), (1, 2));
        assert_eq!(day.bot_messages, 1);
        assert_eq!((day.requests_answered, day.files_edited), (0, 0));
        assert_eq!(day.busiest, None);
    }
}
