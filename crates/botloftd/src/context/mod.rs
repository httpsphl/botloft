//! How full each bot's conversation is, and compacting it (spec 8.6).
//! Claude Code does the counting: the daemon asks it when a process starts,
//! when a turn ends and after a compaction, and follows the usage of each
//! model request in between. Nothing is stored; a new daemon asks again.

mod reload;

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use botloft_core::ids::{BotId, random_uuid};
use botloft_core::protocol::{
    Bot, BotContextChanged, BotIdParams, BotState, ChatBody, ContextUsage, NoticeCode, NoticeItem,
    NoticeLevel,
};
use bytes::Bytes;
use serde_json::{Value, json};
use tracing::debug;

use crate::chat::{control, items};
use crate::service::bots::{active, to_protocol};
use crate::service::{ApiError, ApiResult};
use crate::state::{Daemon, Event};

/// Claude Code's command that compacts the conversation.
const COMPACT: &str = "/compact";

#[derive(Default)]
pub struct Contexts {
    bots: Mutex<HashMap<BotId, Entry>>,
}

#[derive(Default)]
struct Entry {
    /// `None` until Claude Code told the size of this conversation.
    size: Option<Size>,
    /// A compaction is running, or waits behind the turn in progress.
    compacting: bool,
    reload: reload::Tracker,
}

struct Size {
    used: u64,
    window: u64,
    auto_compact: Option<u64>,
    at: i64,
}

impl Entry {
    fn view(&self) -> Option<ContextUsage> {
        self.size.as_ref().map(|size| ContextUsage {
            used_tokens: size.used,
            window_tokens: size.window,
            auto_compact_tokens: size.auto_compact,
            compacting: self.compacting,
            updated_at: size.at,
        })
    }
}

impl Contexts {
    fn lock(&self) -> MutexGuard<'_, HashMap<BotId, Entry>> {
        self.bots
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// What the app shows for `bot`; `None` while its size is not known.
    pub fn get(&self, bot: &BotId) -> Option<ContextUsage> {
        self.lock().get(bot).and_then(Entry::view)
    }

    fn compacting(&self, bot: &BotId) -> bool {
        self.lock().get(bot).is_some_and(|entry| entry.compacting)
    }
}

/// Changes what is known about `bot` and tells the app if it shows.
fn update(daemon: &Daemon, bot: &BotId, change: impl FnOnce(&mut Entry)) {
    let (before, after) = {
        let mut bots = daemon.contexts.lock();
        let entry = bots.entry(bot.clone()).or_default();
        let before = entry.view();
        change(entry);
        (before, entry.view())
    };
    if before != after {
        daemon.emit(Event::BotContext(BotContextChanged {
            bot_id: bot.clone(),
            context: after,
        }));
    }
}

/// Asks the bot's process how full its conversation is. The answer comes
/// as a `control_response` ([`answered`]).
pub(crate) fn ask(daemon: &Daemon, bot: &BotId) {
    let _ = daemon
        .supervisor
        .write_control(bot, control::context_request());
}

/// Claude Code's answer to `get_context_usage`.
pub(crate) fn answered(daemon: &Daemon, bot: &BotId, body: &Value) {
    let used = body["totalTokens"].as_u64();
    let window = body["maxTokens"]
        .as_u64()
        .or_else(|| body["rawMaxTokens"].as_u64())
        .filter(|window| *window > 0);
    let (Some(used), Some(window)) = (used, window) else {
        debug!(bot = %bot, "context usage without the sizes");
        return;
    };
    let auto_compact = body["autoCompactThreshold"]
        .as_u64()
        .filter(|_| body["isAutoCompactEnabled"].as_bool() != Some(false))
        .filter(|at| *at > 0 && *at <= window);
    let at = daemon.clock.now_ms();
    debug!(bot = %bot, used, window, "context usage");
    update(daemon, bot, |entry| {
        entry.reload.measured(used);
        entry.size = Some(Size {
            used,
            window,
            auto_compact,
            at,
        });
    });
}

/// The usage of one model request in the bot's conversation: what it sent
/// is what the conversation holds. Only moves a size that is known.
pub(crate) fn used(daemon: &Daemon, bot: &BotId, message: &Value) {
    if message["model"] == "<synthetic>" {
        return;
    }
    let usage = &message["usage"];
    let used: u64 = [
        "input_tokens",
        "cache_creation_input_tokens",
        "cache_read_input_tokens",
    ]
    .iter()
    .filter_map(|field| usage[*field].as_u64())
    .sum();
    if used == 0 {
        return;
    }
    let at = daemon.clock.now_ms();
    update(daemon, bot, |entry| {
        entry.reload.request(message["id"].as_str(), usage);
        if let Some(size) = &mut entry.size
            && size.used != used
        {
            size.used = used;
            size.at = at;
        }
    });
}

/// `system/status`: Claude Code began to compact, by itself or because it
/// was asked, or stopped.
pub(crate) fn status(daemon: &Daemon, bot: &BotId, event: &Value) {
    if event["status"] == "compacting" {
        update(daemon, bot, |entry| entry.compacting = true);
    } else if !event["compact_result"].is_null() {
        update(daemon, bot, |entry| entry.compacting = false);
    }
}

/// `system/compact_boundary`: the conversation is now a summary. The chat
/// says so, and Claude Code is asked for the new size.
pub(crate) fn compacted(daemon: &Daemon, bot: &BotId, event: &Value) {
    update(daemon, bot, |entry| {
        entry.compacting = false;
        entry.reload.restart();
    });
    let (code, text) = if event["compact_metadata"]["trigger"] == "auto" {
        (
            NoticeCode::AutoCompacted,
            "The conversation was full, so it was compacted: earlier messages are now a summary.",
        )
    } else {
        (
            NoticeCode::Compacted,
            "The conversation was compacted: earlier messages are now a summary.",
        )
    };
    items::add(
        daemon,
        bot,
        ChatBody::Notice(NoticeItem {
            level: NoticeLevel::Info,
            code: Some(code),
            text: text.to_owned(),
        }),
    );
    ask(daemon, bot);
}

/// Whether `event` is what `/compact` printed instead of compacting, such
/// as "No messages to compact". The chat gets a notice, not a reply.
pub(crate) fn command_output(daemon: &Daemon, bot: &BotId, event: &Value) -> bool {
    if event["local_command_run"]["command"] != "compact" {
        return false;
    }
    update(daemon, bot, |entry| entry.compacting = false);
    if event["local_command_outcome"]["kind"] == "failed" {
        let detail = event["message"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        let detail = detail.strip_prefix("Error: ").unwrap_or(detail);
        items::add(
            daemon,
            bot,
            ChatBody::Notice(NoticeItem {
                level: NoticeLevel::Warning,
                code: Some(NoticeCode::CompactFailed),
                text: detail.to_owned(),
            }),
        );
    }
    true
}

/// A `result` closed a turn; returns whether it was a compaction, which
/// ends like a turn without being the bot's work.
pub(crate) fn turn_ended(daemon: &Daemon, bot: &BotId, event: &Value) -> bool {
    let compaction = event["local_command"] == "compact";
    if compaction {
        update(daemon, bot, |entry| entry.compacting = false);
    }
    ask(daemon, bot);
    compaction
}

/// How much of the turn that ended was the conversation written again to
/// the prompt cache (spec 8.7).
pub(crate) fn reloaded(daemon: &Daemon, bot: &BotId) -> u64 {
    let mut reloaded = 0;
    update(daemon, bot, |entry| reloaded = entry.reload.take());
    reloaded
}

/// A process started. A new conversation has a size of its own, not known
/// until the process tells. A turn the old process left open counts
/// nothing for the next.
pub(crate) fn process_started(daemon: &Daemon, bot: &BotId, resumed: bool) {
    update(daemon, bot, |entry| {
        if !resumed {
            entry.size = None;
            entry.reload.restart();
        }
        entry.reload.take();
    });
}

/// The process is gone, and a compaction it had not finished with it.
pub(crate) fn process_ended(daemon: &Daemon, bot: &BotId) {
    update(daemon, bot, |entry| entry.compacting = false);
}

/// `bots.compact`: writes `/compact` to the bot's process like a message,
/// so it waits its turn behind what the bot is doing. Asking again while
/// one is on its way changes nothing.
pub fn compact(daemon: &Daemon, params: BotIdParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, record) = active(&store, &params.bot_id)?;
    if !daemon.contexts.compacting(&record.id) {
        let not_running = || ApiError::Conflict(format!("bot {} is not running", record.id));
        let ready = matches!(
            daemon.supervisor.status(&record.id),
            Some((BotState::Idle | BotState::Busy | BotState::NeedsApproval, _))
        );
        if !ready {
            return Err(not_running());
        }
        let uuid = random_uuid();
        let line = json!({
            "type": "user",
            "uuid": uuid,
            "message": { "role": "user", "content": [{ "type": "text", "text": COMPACT }] },
        });
        daemon
            .supervisor
            .write_message(&record.id, &uuid, Bytes::from(format!("{line}\n")))
            .map_err(|_| not_running())?;
        update(daemon, &record.id, |entry| entry.compacting = true);
    }
    Ok(to_protocol(daemon, &store, &crew, record))
}
