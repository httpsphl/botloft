//! Requests that wait for the owner in the chat: Claude Code's permission
//! requests (spec 10.1) and the chief's bot suggestions (spec 10.2). The
//! MCP call waits here until the owner answers, the request times out, or
//! the bot's process ends.

mod prompt;

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use botloft_core::chat::{SUGGEST_TOOL, clip, tool_input_max, tool_summary};
use botloft_core::ids::{ApprovalId, BotId, ChatItemId};
use botloft_core::protocol::{
    Approval, ApprovalItem, ApprovalStatus, ApprovalsAnswerParams, ChatBody,
};
use botloft_store::ApprovalRecord;
use serde_json::Value;
use tokio::sync::oneshot;
use tracing::{debug, warn};

pub use self::prompt::{PromptArgs, prompt};
use crate::chat::items;
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// Requests waiting for an answer, in memory: they cannot outlive the
/// daemon, since the bots' processes do not either.
#[derive(Default)]
pub struct Approvals {
    waiting: Mutex<HashMap<ApprovalId, oneshot::Sender<Decision>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Decision {
    allow: bool,
    note: Option<String>,
    input: Option<String>,
}

/// How the owner answered a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Answer {
    /// `input` is the request as the owner changed it, if they did.
    Allowed {
        input: Option<String>,
    },
    Denied {
        note: Option<String>,
    },
    /// No answer in time, or the process ended.
    Expired,
}

impl Approvals {
    fn lock(&self) -> MutexGuard<'_, HashMap<ApprovalId, oneshot::Sender<Decision>>> {
        self.waiting
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Puts a request in the bot's chat and waits for the owner. `None` if it
/// could not be saved.
pub(crate) async fn ask(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    tool_name: &str,
    input: &Value,
    tool_use_id: &str,
) -> Option<Answer> {
    let pending = open(daemon, bot, tool_name, input, tool_use_id)?;
    let (answer, waiting) = oneshot::channel();
    daemon
        .approvals
        .lock()
        .insert(pending.record.approval.id.clone(), answer);
    daemon.supervisor.approval_opened(bot, generation);
    let mut guard = Guard {
        daemon,
        bot,
        generation,
        record: &pending.record,
        settled: false,
    };
    let decision = tokio::time::timeout(daemon.bots.approval_timeout, waiting).await;
    guard.settled = true;
    daemon.approvals.lock().remove(&pending.record.approval.id);
    daemon.supervisor.approval_closed(bot, generation);
    Some(match decision {
        Ok(Ok(Decision {
            allow: true, input, ..
        })) => Answer::Allowed { input },
        Ok(Ok(Decision { note, .. })) => Answer::Denied { note },
        // Timed out, or the answer was dropped (the process ended).
        Ok(Err(_)) | Err(_) => {
            settle(daemon, &pending.record, ApprovalStatus::Expired, None, None);
            Answer::Expired
        }
    })
}

/// The owner's answer (`approvals.answer`).
pub fn answer(daemon: &Daemon, params: ApprovalsAnswerParams) -> ApiResult<Approval> {
    let note = params
        .note
        .map(|note| note.trim().to_owned())
        .filter(|note| !note.is_empty());
    let status = if params.allow {
        ApprovalStatus::Allowed
    } else {
        ApprovalStatus::Denied
    };
    let existing = daemon
        .store()
        .approval(&params.approval_id)?
        .ok_or_else(|| ApiError::NotFound(format!("approval {}", params.approval_id)))?;
    // Only a bot suggestion can be changed before it is allowed (spec 10.2).
    let input = match params.input {
        Some(input) if params.allow && existing.approval.tool_name == SUGGEST_TOOL => {
            let value: Value = serde_json::from_str(&input)
                .map_err(|_| ApiError::validation("the changed suggestion is not valid JSON"))?;
            Some(value.to_string())
        }
        _ => None,
    };
    let Some(record) = settle(daemon, &existing, status, note.as_deref(), input.as_deref()) else {
        return Err(ApiError::Conflict(format!(
            "approval {} was already answered or expired",
            params.approval_id
        )));
    };
    if let Some(waiter) = daemon.approvals.lock().remove(&params.approval_id) {
        let _ = waiter.send(Decision {
            allow: params.allow,
            note,
            input,
        });
    }
    Ok(record.approval)
}

/// Expires the bot's open requests: its process is gone (spec 10.1).
pub fn expire_for_bot(daemon: &Daemon, bot: &BotId) {
    let now = daemon.clock.now_ms();
    let expired = match daemon.store().expire_approvals(Some(bot), now) {
        Ok(expired) => expired,
        Err(err) => {
            warn!(bot = %bot, "could not expire approvals: {err}");
            return;
        }
    };
    for record in expired {
        daemon.approvals.lock().remove(&record.approval.id);
        show(daemon, &record);
    }
}

/// Expires every open request, at daemon start: no process survived.
pub fn expire_all(daemon: &Daemon) {
    let now = daemon.clock.now_ms();
    match daemon.store().expire_approvals(None, now) {
        Ok(expired) if !expired.is_empty() => {
            debug!(
                count = expired.len(),
                "expired approvals left by the last run"
            );
        }
        Ok(_) => {}
        Err(err) => warn!("could not expire old approvals: {err}"),
    }
}

struct Pending {
    record: ApprovalRecord,
}

/// Saves the request and puts it in the chat.
fn open(
    daemon: &Daemon,
    bot: &BotId,
    tool_name: &str,
    input: &Value,
    tool_use_id: &str,
) -> Option<Pending> {
    let now = daemon.clock.now_ms();
    let approval = Approval {
        id: ApprovalId::generate(),
        bot_id: bot.clone(),
        tool_name: tool_name.to_owned(),
        summary: tool_summary(tool_name, input),
        input: clip(&input.to_string(), tool_input_max(tool_name)),
        status: ApprovalStatus::Pending,
        note: None,
        created_at: now,
        answered_at: None,
    };
    let item = items::add(daemon, bot, ChatBody::Approval(item_of(&approval)))?;
    let record = ApprovalRecord {
        approval,
        chat_item_id: item.id,
        tool_use_id: tool_use_id.to_owned(),
    };
    if let Err(err) = daemon.store().insert_approval(&record) {
        warn!(bot = %bot, "could not save an approval: {err}");
        return None;
    }
    Some(Pending { record })
}

fn item_of(approval: &Approval) -> ApprovalItem {
    ApprovalItem {
        approval_id: approval.id.clone(),
        tool_name: approval.tool_name.clone(),
        summary: approval.summary.clone(),
        input: approval.input.clone(),
        status: approval.status,
        note: approval.note.clone(),
    }
}

/// Settles a pending request and updates its chat item. `None` if it was
/// no longer pending.
fn settle(
    daemon: &Daemon,
    record: &ApprovalRecord,
    status: ApprovalStatus,
    note: Option<&str>,
    input: Option<&str>,
) -> Option<ApprovalRecord> {
    let now = daemon.clock.now_ms();
    let settled = daemon
        .store()
        .settle_approval(&record.approval.id, status, note, input, now);
    match settled {
        Ok(Some(settled)) => {
            show(daemon, &settled);
            Some(settled)
        }
        Ok(None) => None,
        Err(err) => {
            warn!(approval = %record.approval.id, "could not settle an approval: {err}");
            None
        }
    }
}

fn show(daemon: &Daemon, record: &ApprovalRecord) {
    let item: &ChatItemId = &record.chat_item_id;
    items::update(daemon, item, ChatBody::Approval(item_of(&record.approval)));
}

/// Expires the request if the MCP call goes away before it is settled,
/// e.g. Claude Code dropped the connection.
struct Guard<'a> {
    daemon: &'a Daemon,
    bot: &'a BotId,
    generation: u64,
    record: &'a ApprovalRecord,
    settled: bool,
}

impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if self.settled {
            return;
        }
        self.daemon
            .approvals
            .lock()
            .remove(&self.record.approval.id);
        settle(
            self.daemon,
            self.record,
            ApprovalStatus::Expired,
            None,
            None,
        );
        self.daemon
            .supervisor
            .approval_closed(self.bot, self.generation);
    }
}
