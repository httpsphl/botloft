//! Where a reply goes (spec 40.2): the conversation that asked. A task
//! and an agent's own routine keep theirs; what shows in the chat knows
//! it by its item. `None` is the main conversation.

use botloft_core::ids::{ChatItemId, ConversationId, QuestionId, RoutineId, TaskId};
use rusqlite::{OptionalExtension, params};

use crate::{Result, Store, cached_execute, cached_row};

impl Store {
    /// The conversation a chat item is in.
    pub fn item_conversation(&self, item: &ChatItemId) -> Result<Option<ConversationId>> {
        self.conversation_where(
            "SELECT conversation_id FROM chat_items WHERE id = ?1",
            item.as_str(),
        )
    }

    /// The conversation that asked a question: the one its card is in.
    pub fn question_conversation(&self, question: &QuestionId) -> Result<Option<ConversationId>> {
        self.conversation_where(
            "SELECT c.conversation_id FROM questions q \
             JOIN chat_items c ON c.id = q.chat_item_id WHERE q.id = ?1",
            question.as_str(),
        )
    }

    /// The conversation that gave a task, for its result.
    pub fn task_conversation(&self, task: &TaskId) -> Result<Option<ConversationId>> {
        self.conversation_where(
            "SELECT requester_conversation_id FROM tasks WHERE id = ?1",
            task.as_str(),
        )
    }

    pub fn set_task_conversation(
        &self,
        task: &TaskId,
        conversation: &ConversationId,
    ) -> Result<()> {
        cached_execute(
            &self.conn,
            "UPDATE tasks SET requester_conversation_id = ?2 WHERE id = ?1",
            params![task.as_str(), conversation.as_str()],
        )?;
        Ok(())
    }

    /// The conversation that made a routine or reminder; `None` for the
    /// owner's own, which go to the main one.
    pub fn routine_conversation(&self, routine: &RoutineId) -> Result<Option<ConversationId>> {
        self.conversation_where(
            "SELECT conversation_id FROM routines WHERE id = ?1",
            routine.as_str(),
        )
    }

    pub fn set_routine_conversation(
        &self,
        routine: &RoutineId,
        conversation: &ConversationId,
    ) -> Result<()> {
        cached_execute(
            &self.conn,
            "UPDATE routines SET conversation_id = ?2 WHERE id = ?1",
            params![routine.as_str(), conversation.as_str()],
        )?;
        Ok(())
    }

    fn conversation_where(&self, sql: &str, id: &str) -> Result<Option<ConversationId>> {
        let found: Option<Option<String>> =
            cached_row(&self.conn, sql, [id], |row| row.get(0)).optional()?;
        Ok(found.flatten().and_then(|id| id.parse().ok()))
    }
}
