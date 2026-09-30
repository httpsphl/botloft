//! The hot queries use an index instead of reading whole tables, which
//! never shrink. Each statement repeats the WHERE and ORDER BY of the query
//! it stands for; keep them in step.

use rusqlite::Connection;

use crate::Store;

/// SQLite's plan for `sql`, one step per line.
fn plan(conn: &Connection, sql: &str) -> String {
    let mut stmt = conn
        .prepare(&format!("EXPLAIN QUERY PLAN {sql}"))
        .expect("explain");
    // Parameters stay unbound: the plan does not depend on their values.
    let mut rows = stmt.raw_query();
    let mut steps = Vec::new();
    while let Some(row) = rows.next().expect("plan") {
        steps.push(row.get::<_, String>(3).expect("step"));
    }
    steps.join("\n")
}

fn assert_indexed(conn: &Connection, sql: &str) {
    let plan = plan(conn, sql);
    let scans = plan
        .lines()
        .filter(|step| {
            step.starts_with("SCAN") && !step.contains("USING") && step != &"SCAN CONSTANT ROW"
        })
        .count();
    assert_eq!(scans, 0, "full scan in:\n{sql}\n{plan}");
    assert!(!plan.contains("TEMP B-TREE"), "sort in:\n{sql}\n{plan}");
}

#[test]
fn hot_queries_use_an_index() {
    let store = Store::open_in_memory().expect("store");
    let conn = &store.conn;
    // chat.rs: tool_item, written_files.
    assert_indexed(
        conn,
        "SELECT id FROM chat_items WHERE bot_id = ?1 AND kind = 'tool' \
         AND json_extract(data, '$.toolUseId') = ?2 ORDER BY rowid DESC LIMIT 1",
    );
    let written = plan(
        conn,
        "SELECT json_extract(data, '$.file') FROM chat_items \
         WHERE bot_id = ?1 AND kind = 'tool' \
           AND json_extract(data, '$.file') IS NOT NULL \
           AND json_extract(data, '$.status') != 'failed' \
         GROUP BY json_extract(data, '$.file') ORDER BY MAX(rowid) DESC LIMIT ?2",
    );
    assert!(written.contains("chat_items_tool_file"), "{written}");
    // messages.rs: task_request.
    assert_indexed(
        conn,
        "SELECT id FROM messages WHERE task_id = ?1 AND kind = 'task' ORDER BY rowid LIMIT 1",
    );
    // deliveries.rs: delivery_backlog.
    assert_indexed(
        conn,
        "SELECT (SELECT COUNT(*) FROM deliveries WHERE state IN ('pending', 'sending')), \
                (SELECT COUNT(*) FROM deliveries WHERE state = 'dead')",
    );
    // routines.rs: due_routines; routine_runs.rs: runs, runs_with_dead_delivery.
    // Only the ties among routines already due get sorted.
    let due = plan(
        conn,
        "SELECT id FROM routines WHERE enabled AND archived_at IS NULL AND next_run_at <= ?1 \
         ORDER BY next_run_at, id",
    );
    assert!(due.contains("USING INDEX routines_due"), "{due}");
    assert_indexed(
        conn,
        "SELECT id FROM routine_runs WHERE routine_id = ?1 ORDER BY rowid DESC LIMIT ?2",
    );
    assert_indexed(
        conn,
        "SELECT r.id FROM routine_runs r WHERE r.status = 'queued' AND EXISTS \
         (SELECT 1 FROM deliveries d WHERE d.message_id = r.message_id AND d.state = 'dead')",
    );
}

#[test]
fn deleting_a_parent_finds_its_children_by_index() {
    let store = Store::open_in_memory().expect("store");
    let conn = &store.conn;
    for sql in [
        "SELECT 1 FROM messages WHERE task_id = ?1",
        "SELECT 1 FROM messages WHERE routine_id = ?1",
        "SELECT 1 FROM deliveries WHERE message_id = ?1",
        "SELECT 1 FROM tasks WHERE requester_bot_id = ?1",
        "SELECT 1 FROM tasks WHERE crew_id = ?1",
        "SELECT 1 FROM tasks WHERE origin_task_id = ?1",
        "SELECT 1 FROM approvals WHERE bot_id = ?1",
        "SELECT 1 FROM approvals WHERE chat_item_id = ?1",
    ] {
        assert_indexed(conn, sql);
    }
}
