//! Permission requests (spec 10.1). Claude Code calls the MCP tool
//! `permission_prompt`; the call waits here until the owner answers in the
//! chat, the request times out, or the bot's process ends.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::chat::{clip, tool_input_max, tool_summary};
use botloft_core::ids::{ApprovalId, BotId, ChatItemId};
use botloft_core::protocol::{
    Approval, ApprovalItem, ApprovalStatus, ApprovalsAnswerParams, ChatBody, ToolStatus,
};
use botloft_store::ApprovalRecord;
use serde_json::{Value, json};
use tokio::sync::oneshot;
use tracing::{debug, warn};

use crate::chat::items;
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// How long to wait for the `tool_use` event that the request is about;
/// it can arrive a moment after the MCP call.
const MATCH_WINDOW: Duration = Duration::from_secs(2);
const MATCH_POLL: Duration = Duration::from_millis(50);

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
}

impl Approvals {
    fn lock(&self) -> MutexGuard<'_, HashMap<ApprovalId, oneshot::Sender<Decision>>> {
        self.waiting
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Arguments Claude Code sends to the permission tool.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PromptArgs {
    pub tool_name: String,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub tool_use_id: String,
}

/// Handles one `permission_prompt` call and returns the text of the tool
/// result: `{"behavior": "allow" | "deny", ...}`.
pub async fn prompt(daemon: &Daemon, bot: &BotId, generation: u64, args: PromptArgs) -> String {
    if !matches_running_tool(daemon, bot, &args.tool_use_id).await {
        debug!(bot = %bot, "permission request for no running tool; denied");
        return deny("There is no pending tool call with that id.");
    }
    let Some(pending) = open(daemon, bot, &args) else {
        return deny("Botloft could not ask the owner.");
    };
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
    match decision {
        Ok(Ok(Decision { allow: true, .. })) => {
            json!({ "behavior": "allow", "updatedInput": args.input }).to_string()
        }
        Ok(Ok(Decision { allow: false, note })) => match note {
            Some(note) => deny(&format!("The owner denied this: {note}")),
            None => deny("The owner denied this."),
        },
        // Timed out, or the answer was dropped (the process ended).
        Ok(Err(_)) | Err(_) => {
            settle(daemon, &pending.record, ApprovalStatus::Expired, None);
            deny("The owner did not answer in time.")
        }
    }
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
    let Some(record) = settle(daemon, &existing, status, note.as_deref()) else {
        return Err(ApiError::Conflict(format!(
            "approval {} was already answered or expired",
            params.approval_id
        )));
    };
    if let Some(waiter) = daemon.approvals.lock().remove(&params.approval_id) {
        let _ = waiter.send(Decision {
            allow: params.allow,
            note,
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

fn deny(message: &str) -> String {
    json!({ "behavior": "deny", "message": message }).to_string()
}

/// Whether the bot's chat shows `tool_use_id` running, waiting briefly for
/// the event if it has not been read yet.
async fn matches_running_tool(daemon: &Daemon, bot: &BotId, tool_use_id: &str) -> bool {
    if tool_use_id.is_empty() {
        return false;
    }
    let deadline = tokio::time::Instant::now() + MATCH_WINDOW;
    loop {
        let found = daemon.store().tool_item(bot, tool_use_id);
        if let Ok(Some(item)) = found
            && matches!(&item.body, ChatBody::Tool(tool) if tool.status == ToolStatus::Running)
        {
            return true;
        }
        if tokio::time::Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(MATCH_POLL).await;
    }
}

struct Pending {
    record: ApprovalRecord,
}

/// Saves the request and puts it in the chat.
fn open(daemon: &Daemon, bot: &BotId, args: &PromptArgs) -> Option<Pending> {
    let now = daemon.clock.now_ms();
    let approval = Approval {
        id: ApprovalId::generate(),
        bot_id: bot.clone(),
        tool_name: args.tool_name.clone(),
        summary: tool_summary(&args.tool_name, &args.input),
        input: clip(&args.input.to_string(), tool_input_max(&args.tool_name)),
        status: ApprovalStatus::Pending,
        note: None,
        created_at: now,
        answered_at: None,
    };
    let item = items::add(daemon, bot, ChatBody::Approval(item_of(&approval)))?;
    let record = ApprovalRecord {
        approval,
        chat_item_id: item.id,
        tool_use_id: args.tool_use_id.clone(),
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
) -> Option<ApprovalRecord> {
    let now = daemon.clock.now_ms();
    let settled = daemon
        .store()
        .settle_approval(&record.approval.id, status, note, now);
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
        settle(self.daemon, self.record, ApprovalStatus::Expired, None);
        self.daemon
            .supervisor
            .approval_closed(self.bot, self.generation);
    }
}
