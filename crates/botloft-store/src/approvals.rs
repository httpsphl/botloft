//! `approvals` table: permission requests waiting for the owner (spec 10.1).

use botloft_core::ids::{ApprovalId, BotId, ChatItemId};
use botloft_core::protocol::{Approval, ApprovalStatus};
use rusqlite::{OptionalExtension, Row, params};

use crate::{Result, Store, parse_column};

const COLUMNS: &str =
    "id, bot_id, tool_name, summary, input, status, note, created_at, answered_at, chat_item_id";

/// An approval and the chat item that shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRecord {
    pub approval: Approval,
    pub chat_item_id: ChatItemId,
    pub tool_use_id: String,
}

fn from_row(row: &Row<'_>) -> rusqlite::Result<ApprovalRecord> {
    Ok(ApprovalRecord {
        approval: Approval {
            id: parse_column(row, 0)?,
            bot_id: parse_column(row, 1)?,
            tool_name: row.get(2)?,
            summary: row.get(3)?,
            input: row.get(4)?,
            status: parse_column(row, 5)?,
            note: row.get(6)?,
            created_at: row.get(7)?,
            answered_at: row.get(8)?,
        },
        chat_item_id: parse_column(row, 9)?,
        tool_use_id: row.get(10)?,
    })
}

impl Store {
    pub fn insert_approval(&self, record: &ApprovalRecord) -> Result<()> {
        let approval = &record.approval;
        self.conn.execute(
            &format!(
                "INSERT INTO approvals ({COLUMNS}, tool_use_id) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
            ),
            params![
                approval.id.as_str(),
                approval.bot_id.as_str(),
                approval.tool_name,
                approval.summary,
                approval.input,
                approval.status.as_str(),
                approval.note,
                approval.created_at,
                approval.answered_at,
                record.chat_item_id.as_str(),
                record.tool_use_id,
            ],
        )?;
        Ok(())
    }

    pub fn approval(&self, id: &ApprovalId) -> Result<Option<ApprovalRecord>> {
        Ok(self
            .conn
            .query_row(
                &format!("SELECT {COLUMNS}, tool_use_id FROM approvals WHERE id = ?1"),
                [id.as_str()],
                from_row,
            )
            .optional()?)
    }

    /// Answers a pending approval; `input` replaces what was asked, when the
    /// owner changed it. `None` if it was already settled.
    pub fn settle_approval(
        &self,
        id: &ApprovalId,
        status: ApprovalStatus,
        note: Option<&str>,
        input: Option<&str>,
        now: i64,
    ) -> Result<Option<ApprovalRecord>> {
        Ok(self
            .conn
            .query_row(
                &format!(
                    "UPDATE approvals SET status = ?2, note = ?3, answered_at = ?4, \
                     input = COALESCE(?5, input) \
                     WHERE id = ?1 AND status = 'pending' RETURNING {COLUMNS}, tool_use_id"
                ),
                params![id.as_str(), status.as_str(), note, now, input],
                from_row,
            )
            .optional()?)
    }

    /// The bot's requests that still wait for the owner.
    pub fn pending_approvals(&self, bot: &BotId) -> Result<Vec<ApprovalId>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM approvals WHERE bot_id = ?1 AND status = 'pending'")?;
        let rows = stmt.query_map([bot.as_str()], |row| parse_column(row, 0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }

    /// Expires every pending approval, of one bot or of all: their process
    /// is gone, so nobody waits for the answer.
    pub fn expire_approvals(&self, bot: Option<&BotId>, now: i64) -> Result<Vec<ApprovalRecord>> {
        let mut stmt = self.conn.prepare(&format!(
            "UPDATE approvals SET status = 'expired', answered_at = ?2 \
             WHERE status = 'pending' AND (?1 IS NULL OR bot_id = ?1) \
             RETURNING {COLUMNS}, tool_use_id"
        ))?;
        let rows = stmt.query_map(params![bot.map(BotId::as_str), now], from_row)?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}

#[cfg(test)]
mod tests {
    use botloft_core::protocol::{ApprovalItem, ChatBody, ChatItem};

    use super::*;
    use crate::tests::Fixture;

    fn pending(fx: &Fixture, bot: usize) -> ApprovalRecord {
        let bot_id = fx.bots[bot].id.clone();
        let approval = Approval {
            id: ApprovalId::generate(),
            bot_id: bot_id.clone(),
            tool_name: "Bash".into(),
            summary: "rm -rf build".into(),
            input: r#"{"command":"rm -rf build"}"#.into(),
            status: ApprovalStatus::Pending,
            note: None,
            created_at: 10,
            answered_at: None,
        };
        let item = ChatItem {
            id: ChatItemId::generate(),
            bot_id,
            body: ChatBody::Approval(ApprovalItem {
                approval_id: approval.id.clone(),
                tool_name: approval.tool_name.clone(),
                summary: approval.summary.clone(),
                explanation: None,
                input: approval.input.clone(),
                status: ApprovalStatus::Pending,
                note: None,
            }),
            created_at: 10,
            updated_at: 10,
        };
        fx.store.insert_chat_item(&item).expect("item");
        let record = ApprovalRecord {
            approval,
            chat_item_id: item.id,
            tool_use_id: "toolu_1".into(),
        };
        fx.store.insert_approval(&record).expect("insert");
        record
    }

    #[test]
    fn an_approval_is_answered_once() {
        let fx = Fixture::new();
        let record = pending(&fx, 0);
        assert_eq!(
            fx.store.approval(&record.approval.id).expect("read"),
            Some(record.clone())
        );
        let denied = fx
            .store
            .settle_approval(
                &record.approval.id,
                ApprovalStatus::Denied,
                Some("not now"),
                None,
                20,
            )
            .expect("settle")
            .expect("was pending");
        assert_eq!(denied.approval.status, ApprovalStatus::Denied);
        assert_eq!(denied.approval.note.as_deref(), Some("not now"));
        assert_eq!(denied.approval.answered_at, Some(20));
        let again = fx
            .store
            .settle_approval(&record.approval.id, ApprovalStatus::Allowed, None, None, 30)
            .expect("settle");
        assert_eq!(again, None);
    }

    #[test]
    fn an_answer_can_replace_what_was_asked() {
        let fx = Fixture::new();
        let record = pending(&fx, 0);
        let allowed = fx
            .store
            .settle_approval(
                &record.approval.id,
                ApprovalStatus::Allowed,
                None,
                Some(r#"{"name":"Designer"}"#),
                40,
            )
            .expect("settle")
            .expect("was pending");
        assert_eq!(allowed.approval.input, r#"{"name":"Designer"}"#);
    }

    #[test]
    fn a_bots_pending_approvals_expire_together() {
        let fx = Fixture::new();
        let first = pending(&fx, 0);
        let other = pending(&fx, 1);
        assert_eq!(
            fx.store.pending_approvals(&fx.bots[0].id).expect("pending"),
            vec![first.approval.id.clone()]
        );
        let expired = fx
            .store
            .expire_approvals(Some(&fx.bots[0].id), 40)
            .expect("expire");
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].approval.id, first.approval.id);
        assert_eq!(expired[0].approval.status, ApprovalStatus::Expired);
        let rest = fx.store.expire_approvals(None, 50).expect("expire all");
        assert_eq!(rest.len(), 1);
        assert_eq!(rest[0].approval.id, other.approval.id);
    }
}
