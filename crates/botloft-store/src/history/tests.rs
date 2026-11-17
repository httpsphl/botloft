use super::*;
use crate::tests::Fixture;

/// Before this moment is old; the history kept starts here.
const CUTOFF: i64 = 1_000;
const OLD: i64 = 10;
const NEW: i64 = 2_000;

fn item(fx: &Fixture, id: &str, at: i64, text: &str) {
    fx.store
        .conn
        .execute(
            "INSERT INTO chat_items (id, bot_id, kind, data, created_at, updated_at) \
             VALUES (?1, ?2, 'reply', json_object('text', ?3), ?4, ?4)",
            params![id, fx.bots[0].id.as_str(), text, at],
        )
        .expect("item");
}

/// A message to the first bot and its delivery in `state`.
fn message(fx: &Fixture, id: &str, at: i64, state: &str, read: bool) {
    let conn = &fx.store.conn;
    conn.execute(
        "INSERT INTO messages (id, crew_id, from_kind, to_bot_id, kind, body, created_at) \
         VALUES (?1, ?2, 'owner', ?3, 'note', 'body', ?4)",
        params![id, fx.crew.id.as_str(), fx.bots[0].id.as_str(), at],
    )
    .expect("message");
    conn.execute(
        "INSERT INTO deliveries (id, message_id, bot_id, state, next_attempt_at, updated_at, read_at) \
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
        params![
            format!("dlv_{id}"),
            id,
            fx.bots[0].id.as_str(),
            state,
            at,
            read.then_some(at)
        ],
    )
    .expect("delivery");
}

fn ids(fx: &Fixture, table: &str) -> Vec<String> {
    let mut stmt = fx
        .store
        .conn
        .prepare(&format!("SELECT id FROM {table} ORDER BY id"))
        .expect("select");
    stmt.query_map([], |row| row.get(0))
        .expect("rows")
        .collect::<rusqlite::Result<_>>()
        .expect("ids")
}

fn seed(fx: &Fixture) {
    let bot = fx.bots[0].id.as_str();
    let crew = fx.crew.id.as_str();
    for (id, at) in [
        ("i_old", OLD),
        ("i_new", NEW),
        ("i_pending", OLD),
        ("i_allowed", OLD),
        ("i_q_open", OLD),
        ("i_q_done", OLD),
        ("i_react", OLD),
    ] {
        item(fx, id, at, &format!("word{id}"));
    }
    message(fx, "m_read", OLD, "sent", true);
    message(fx, "m_dead", OLD, "dead", false);
    message(fx, "m_pending", OLD, "pending", false);
    message(fx, "m_unread", OLD, "sent", false);
    message(fx, "m_task", OLD, "sent", true);
    message(fx, "m_run", OLD, "sent", true);
    message(fx, "m_react", OLD, "sent", true);
    message(fx, "m_answer", NEW, "sent", true);
    fx.store
        .conn
        .execute_batch(&format!(
            "INSERT INTO approvals (id, bot_id, chat_item_id, tool_use_id, tool_name, summary, input, status, created_at) \
              VALUES ('a_pending', '{bot}', 'i_pending', 't1', 'Bash', 's', '{{}}', 'pending', {OLD}), \
                     ('a_allowed', '{bot}', 'i_allowed', 't2', 'Bash', 's', '{{}}', 'allowed', {OLD});
             INSERT INTO questions (id, crew_id, bot_id, chat_item_id, text, options, status, created_at) \
              VALUES ('q_open', '{crew}', '{bot}', 'i_q_open', 'Which?', '[]', 'open', {OLD}), \
                     ('q_done', '{crew}', '{bot}', 'i_q_done', 'Which?', '[]', 'answered', {OLD});
             UPDATE messages SET question_id = 'q_done' WHERE id = 'm_answer';
             INSERT INTO reactions (item_id, bot_id, emoji, quote, created_at, sent_in) \
              VALUES ('i_react', '{bot}', 'x', 'q', {OLD}, 'm_react');
             INSERT INTO attachments (id, message_id, name, media_type, size, path, created_at) \
              VALUES ('att_1', 'm_read', 'a.png', 'image/png', 1, 'attachments/a.png', {OLD});
             INSERT INTO tasks (id, crew_id, requester_bot_id, assignee_bot_id, status, deadline_at, hops, created_at, updated_at) \
              VALUES ('t_open', '{crew}', '{bot}', '{bot}', 'open', 99999, 0, {OLD}, {OLD});
             UPDATE messages SET task_id = 't_open' WHERE id = 'm_task';
             INSERT INTO routines (id, bot_id, name, prompt, schedule, timezone, overlap, missed, enabled, created_at, updated_at) \
              VALUES ('rtn_1', '{bot}', 'Morning', 'p', '{{}}', 'UTC', 'skip', 'skip', 1, {OLD}, {OLD});
             INSERT INTO routine_runs (id, routine_id, scheduled_for, status, message_id, created_at) \
              VALUES ('r_done', 'rtn_1', {OLD}, 'done', 'm_read', {OLD}), \
                     ('r_queued', 'rtn_1', {OLD}, 'queued', 'm_run', {OLD}), \
                     ('r_new', 'rtn_1', {NEW}, 'done', NULL, {NEW});"
        ))
        .expect("seed");
}

#[test]
fn old_history_goes_and_what_still_waits_stays() {
    let fx = Fixture::new();
    seed(&fx);

    let mut total = Pruned::default();
    // A batch of two: it takes several passes, and stops when one finds
    // nothing more.
    loop {
        let pruned = fx.store.prune_history(CUTOFF, 2).expect("prune");
        if pruned.is_empty() {
            break;
        }
        total += pruned;
    }
    assert_eq!(
        total,
        Pruned {
            chat_items: 4,
            messages: 3,
            routine_runs: 1
        }
    );
    assert_eq!(ids(&fx, "chat_items"), ["i_new", "i_pending", "i_q_open"]);
    assert_eq!(
        ids(&fx, "messages"),
        ["m_answer", "m_pending", "m_run", "m_task", "m_unread"]
    );
    assert_eq!(ids(&fx, "approvals"), ["a_pending"]);
    assert_eq!(ids(&fx, "questions"), ["q_open"]);
    assert_eq!(ids(&fx, "routine_runs"), ["r_new", "r_queued"]);
    assert!(ids(&fx, "attachments").is_empty());
    let answer: Option<String> = fx
        .store
        .conn
        .query_row(
            "SELECT question_id FROM messages WHERE id = 'm_answer'",
            [],
            |row| row.get(0),
        )
        .expect("answer");
    assert_eq!(answer, None, "the answer stays, without its old question");
    let found = |word: &str| -> i64 {
        fx.store
            .conn
            .query_row(
                "SELECT count(*) FROM chat_search WHERE chat_search MATCH ?1",
                [word],
                |row| row.get(0),
            )
            .expect("search")
    };
    assert_eq!(found("wordi_old"), 0, "search forgets it too");
    assert_eq!(found("wordi_new"), 1);
}
