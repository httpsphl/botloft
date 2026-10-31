//! Requests that wait for the owner in the chat: Claude Code's permission
//! requests (spec 10.1) and the chief's bot suggestions (spec 10.2). The
//! MCP call waits here until the owner answers, the request times out, or
//! the bot's process ends.

mod always;
mod prompt;
mod turn;

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::chat::{
    BROWSER_SITE_TOOL, CHANGE_ROUTINE_TOOL, CREW_ACCESS_TOOL, DESKTOP_TOOL, ROUTINE_TOOL,
    SHOW_PLAN_TOOL, SUGGEST_TOOL, clip, tool_input_max, tool_summary,
};
use botloft_core::command::tool_explanation;
use botloft_core::ids::{ApprovalId, BotId, ChatItemId};
use botloft_core::protocol::{
    AllowScope, Approval, ApprovalItem, ApprovalStatus, ApprovalsAnswerParams, ChatBody, ChatItem,
};
use botloft_store::ApprovalRecord;
use serde_json::Value;
use tokio::sync::oneshot;
use tracing::{debug, warn};

pub use self::prompt::{PromptArgs, prompt};
pub use self::turn::{ThisTurn, forget_turns};
use crate::chat::items;
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// Requests waiting for an answer, in memory: they cannot outlive the
/// daemon, since the bots' processes do not either.
#[derive(Default)]
pub struct Approvals {
    waiting: Mutex<HashMap<ApprovalId, oneshot::Sender<Decision>>>,
    /// What the owner allowed only for a bot's turn (spec 24.2, 21.5).
    pub this_turn: ThisTurn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Decision {
    allow: bool,
    note: Option<String>,
    input: Option<String>,
    once: bool,
}

/// How the owner answered a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Answer {
    /// `input` is the request as the owner changed it, if they did.
    Allowed {
        input: Option<String>,
        /// Only until the bot's turn ends, for an app on the desktop or a
        /// site (spec 24.2, 21.5): the tool saves nothing.
        once: bool,
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

    /// Lets go of requests nobody will answer: their bot was deleted.
    pub(crate) fn forget(&self, requests: &[ApprovalId]) {
        let mut waiting = self.lock();
        for request in requests {
            waiting.remove(request);
        }
    }
}

/// How a request of an agent that asks in its own protocol ended (spec 30).
pub(crate) enum Verdict {
    Allow,
    /// Denied, or never answered; the agent is told no.
    Deny,
}

/// A request of an agent for the owner: a rule of the bot may already allow
/// it, and otherwise the owner is asked in the chat like for Claude Code.
/// `tool_use_id` is the id of the item the agent shows for it.
pub(crate) async fn agent_request(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    tool_name: &str,
    input: &Value,
    tool_use_id: &str,
) -> Verdict {
    if always::allowed(daemon, bot, tool_name, input) {
        return Verdict::Allow;
    }
    match ask(daemon, bot, generation, tool_name, input, tool_use_id).await {
        Some(Answer::Allowed { .. }) => Verdict::Allow,
        _ => Verdict::Deny,
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
    ask_with(
        daemon,
        bot,
        generation,
        tool_name,
        input,
        tool_use_id,
        |_| {},
    )
    .await
}

/// `ask`, telling `opened` the request's id once it is in the chat.
pub(crate) async fn ask_with(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    tool_name: &str,
    input: &Value,
    tool_use_id: &str,
    opened: impl FnOnce(&ApprovalId),
) -> Option<Answer> {
    let (pending, waiting) = open(daemon, bot, tool_name, input, tool_use_id)?;
    opened(&pending.record.approval.id);
    let wait = Wait {
        within: daemon.settings.approval_wait(),
        needs_owner: true,
    };
    settle_after(daemon, bot, generation, &pending, waiting, wait).await
}

/// Puts a request in the bot's chat that settles by itself after `within`:
/// the bot keeps working meanwhile, so it does not wait for the owner in
/// the app's eyes (spec 10.5).
pub(crate) async fn ask_briefly(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    tool_name: &str,
    input: &Value,
    within: Duration,
) -> Option<Answer> {
    let (pending, waiting) = open(daemon, bot, tool_name, input, "")?;
    let wait = Wait {
        within,
        needs_owner: false,
    };
    settle_after(daemon, bot, generation, &pending, waiting, wait).await
}

/// How long a request waits, and whether its bot waits for the owner then.
struct Wait {
    within: Duration,
    needs_owner: bool,
}

async fn settle_after(
    daemon: &Daemon,
    bot: &BotId,
    generation: u64,
    pending: &Pending,
    waiting: oneshot::Receiver<Decision>,
    wait: Wait,
) -> Option<Answer> {
    if wait.needs_owner {
        daemon.supervisor.approval_opened(bot, generation);
    }
    let mut guard = Guard {
        daemon,
        bot,
        generation,
        record: &pending.record,
        needs_owner: wait.needs_owner,
        settled: false,
    };
    let decision = tokio::time::timeout(wait.within, waiting).await;
    guard.settled = true;
    daemon.approvals.lock().remove(&pending.record.approval.id);
    if wait.needs_owner {
        daemon.supervisor.approval_closed(bot, generation);
    }
    Some(match decision {
        Ok(Ok(Decision {
            allow: true,
            input,
            once,
            ..
        })) => Answer::Allowed { input, once },
        Ok(Ok(Decision { note, .. })) => Answer::Denied { note },
        // Timed out, or the answer was dropped (the process ended).
        Ok(Err(_)) | Err(_) => {
            settle(
                daemon,
                &pending.record,
                ApprovalStatus::Expired,
                None,
                None,
                false,
            );
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
    // Only a bot or routine suggestion can be changed before it is allowed
    // (spec 10.2, 20.12); a request to reach another crew says for how long
    // (10.4).
    let input = match params.input {
        Some(input) if params.allow && existing.approval.tool_name == SUGGEST_TOOL => {
            let value: Value = serde_json::from_str(&input)
                .map_err(|_| ApiError::validation("the changed suggestion is not valid JSON"))?;
            Some(value.to_string())
        }
        Some(input) if params.allow && existing.approval.tool_name == ROUTINE_TOOL => Some(
            crate::tools::check_changed_routine(daemon, &existing.approval.input, &input)?,
        ),
        Some(input) if params.allow && existing.approval.tool_name == CHANGE_ROUTINE_TOOL => Some(
            crate::tools::check_changed_routine_change(daemon, &existing.approval.input, &input)?,
        ),
        Some(input) if params.allow && existing.approval.tool_name == CREW_ACCESS_TOOL => Some(
            crate::tools::check_crew_access_answer(&existing.approval.input, &input)?,
        ),
        Some(input) if params.allow && existing.approval.tool_name == SHOW_PLAN_TOOL => Some(
            crate::tools::check_plan_answer(&existing.approval.input, &input)?,
        ),
        _ => None,
    };
    // Only an app on the desktop or a site may be allowed for the turn
    // alone (spec 24.2, 21.5).
    let once = params.allow
        && params.once == Some(true)
        && [DESKTOP_TOOL, BROWSER_SITE_TOOL].contains(&existing.approval.tool_name.as_str());
    let settled = settle(
        daemon,
        &existing,
        status,
        note.as_deref(),
        input.as_deref(),
        once,
    );
    let Some(record) = settled else {
        return Err(ApiError::Conflict(format!(
            "approval {} was already answered or expired",
            params.approval_id
        )));
    };
    if params.allow && params.always == Some(true) {
        always::remember(daemon, &record);
    }
    if let Some(waiter) = daemon.approvals.lock().remove(&params.approval_id) {
        let _ = waiter.send(Decision {
            allow: params.allow,
            note,
            input,
            once,
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
        show(daemon, &record, false);
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

/// Saves the request, waits for its answer and puts it in the chat, in
/// that order: the owner may answer as soon as the chat shows it.
fn open(
    daemon: &Daemon,
    bot: &BotId,
    tool_name: &str,
    input: &Value,
    tool_use_id: &str,
) -> Option<(Pending, oneshot::Receiver<Decision>)> {
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
    let shown = item_of(
        &approval,
        tool_explanation(tool_name, input),
        always::scope_of(tool_name, input),
        None,
    );
    let mut body = ChatBody::Approval(shown);
    crate::agent_keys::redact_body(daemon, bot, &mut body);
    let item = ChatItem {
        id: ChatItemId::generate(),
        bot_id: bot.clone(),
        body,
        created_at: now,
        updated_at: now,
    };
    let record = ApprovalRecord {
        approval,
        chat_item_id: item.id.clone(),
        tool_use_id: tool_use_id.to_owned(),
    };
    {
        let store = daemon.store();
        let saved = store
            .insert_chat_item(&item)
            .and_then(|()| store.insert_approval(&record));
        if let Err(err) = saved {
            warn!(bot = %bot, "could not save an approval: {err}");
            return None;
        }
    }
    let (answer, waiting) = oneshot::channel();
    daemon
        .approvals
        .lock()
        .insert(record.approval.id.clone(), answer);
    items::announce(daemon, item);
    Some((Pending { record }, waiting))
}

fn item_of(
    approval: &Approval,
    explanation: Option<String>,
    always: Option<AllowScope>,
    once: Option<bool>,
) -> ApprovalItem {
    ApprovalItem {
        approval_id: approval.id.clone(),
        tool_name: approval.tool_name.clone(),
        summary: approval.summary.clone(),
        explanation,
        input: approval.input.clone(),
        status: approval.status,
        note: approval.note.clone(),
        always,
        once,
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
    once: bool,
) -> Option<ApprovalRecord> {
    let now = daemon.clock.now_ms();
    let settled = daemon
        .store()
        .settle_approval(&record.approval.id, status, note, input, now);
    match settled {
        Ok(Some(settled)) => {
            show(daemon, &settled, once);
            Some(settled)
        }
        Ok(None) => None,
        Err(err) => {
            warn!(approval = %record.approval.id, "could not settle an approval: {err}");
            None
        }
    }
}

/// Puts the settled request in its chat item; `once` when it was allowed
/// for the bot's turn only.
fn show(daemon: &Daemon, record: &ApprovalRecord, once: bool) {
    let item: &ChatItemId = &record.chat_item_id;
    // The bot's explanation and what "Allow always" covers are kept only
    // in the chat item.
    let shown = daemon.store().chat_item(item).ok().flatten();
    let (explanation, always) = match shown.map(|shown| shown.body) {
        Some(ChatBody::Approval(shown)) => (shown.explanation, shown.always),
        _ => (None, None),
    };
    let shown = item_of(&record.approval, explanation, always, once.then_some(true));
    items::update(
        daemon,
        &record.approval.bot_id,
        item,
        ChatBody::Approval(shown),
    );
}

/// Expires the request if the MCP call goes away before it is settled,
/// e.g. Claude Code dropped the connection.
struct Guard<'a> {
    daemon: &'a Daemon,
    bot: &'a BotId,
    generation: u64,
    record: &'a ApprovalRecord,
    needs_owner: bool,
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
            false,
        );
        if self.needs_owner {
            self.daemon
                .supervisor
                .approval_closed(self.bot, self.generation);
        }
    }
}
