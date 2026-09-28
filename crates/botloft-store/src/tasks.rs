//! `tasks` table (spec 9.4). A task is inserted with the message that asks
//! for it, in [`Store::insert_message`].

use botloft_core::ids::TaskId;
use botloft_core::protocol::Task;
use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::messages::optional_column;
use crate::{Result, Store, parse_column};

const COLUMNS: &str = "id, crew_id, requester_bot_id, assignee_bot_id, status, deadline_at, \
                       hops, origin_task_id, result, created_at, updated_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: parse_column(row, 0)?,
        crew_id: parse_column(row, 1)?,
        requester_bot_id: parse_column(row, 2)?,
        assignee_bot_id: parse_column(row, 3)?,
        status: parse_column(row, 4)?,
        deadline_at: row.get(5)?,
        hops: row.get(6)?,
        origin_task_id: optional_column(row, 7)?,
        result: row.get(8)?,
        created_at: row.get(9)?,
        updated_at: row.get(10)?,
    })
}

impl Store {
    pub(crate) fn insert_task_in(conn: &Connection, task: &Task) -> Result<()> {
        conn.execute(
            &format!(
                "INSERT INTO tasks ({COLUMNS}) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
            ),
            params![
                task.id.as_str(),
                task.crew_id.as_str(),
                task.requester_bot_id.as_str(),
                task.assignee_bot_id.as_str(),
                task.status.as_str(),
                task.deadline_at,
                task.hops,
                task.origin_task_id.as_ref().map(TaskId::as_str),
                task.result,
                task.created_at,
                task.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn task(&self, id: &TaskId) -> Result<Option<Task>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS} FROM tasks WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::{MessageKind, TaskStatus};

    use super::*;
    use crate::tests::{Fixture, message_to};

    #[test]
    fn a_task_is_saved_with_the_message_that_asks_for_it() {
        let fx = Fixture::new();
        let task = Task {
            id: TaskId::generate(),
            crew_id: fx.crew.id.clone(),
            requester_bot_id: fx.bots[0].id.clone(),
            assignee_bot_id: fx.bots[1].id.clone(),
            status: TaskStatus::Open,
            deadline_at: 7_200_000,
            hops: 1,
            origin_task_id: None,
            result: None,
            created_at: 0,
            updated_at: 0,
        };
        let (mut message, delivery) = message_to(&fx.crew.id, &fx.bots[1].id, "do it");
        message.kind = MessageKind::Task;
        message.task_id = Some(task.id.clone());
        fx.store
            .insert_message(&message, &delivery, Some(&task))
            .expect("insert");
        assert_eq!(fx.store.task(&task.id).expect("read"), Some(task));
    }
}
