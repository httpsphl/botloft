//! How full each bot's conversation is, and compacting it (spec 8.6).
//! Claude Code does the counting: the daemon asks it when a process starts,
//! when a turn ends and after a compaction, and follows the usage of each
//! model request in between. Nothing is stored; a new daemon asks again.

mod compact;
mod parts;
mod reload;

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};

use botloft_core::ids::BotId;
use botloft_core::protocol::{
    BotContextChanged, ChatBody, ContextFile, ContextPart, ContextUsage, NoticeCode, NoticeItem,
    NoticeLevel,
};
use serde_json::Value;
use tracing::debug;

use crate::chat::{control, items};
use crate::state::{Daemon, Event};

pub use self::compact::compact;
pub(crate) use self::compact::{forget_while_asleep, woke};

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
    /// The owner asked to compact while the bot slept: it goes once the
    /// bot is up again.
    compact_on_wake: bool,
    reload: reload::Tracker,
}

struct Size {
    used: u64,
    window: u64,
    auto_compact: Option<u64>,
    at: i64,
    /// As of Claude Code's last answer (spec 33.3).
    parts: Vec<ContextPart>,
    memory_files: Vec<ContextFile>,
}

impl Entry {
    fn view(&self) -> Option<ContextUsage> {
        self.size.as_ref().map(|size| ContextUsage {
            used_tokens: size.used,
            window_tokens: size.window,
            auto_compact_tokens: size.auto_compact,
            compacting: self.compacting,
            updated_at: size.at,
            parts: size.parts.clone(),
            memory_files: size.memory_files.clone(),
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
    // An agent that does not answer must not be asked.
    let speaks = daemon
        .store()
        .bot(bot)
        .ok()
        .flatten()
        .and_then(|record| crate::agent::of(record.agent))
        .is_some_and(|agent| agent.speaks_control());
    if !speaks {
        return;
    }
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
    let (parts, memory_files) = (parts::parts(body), parts::memory_files(body));
    update(daemon, bot, |entry| {
        entry.reload.measured(used);
        entry.size = Some(Size {
            used,
            window,
            auto_compact,
            at,
            parts,
            memory_files,
        });
    });
}

/// An agent that is not Claude Code says how much its conversation holds
/// (spec 30): the daemon does not ask it.
pub(crate) fn report(daemon: &Daemon, bot: &BotId, used: u64, window: u64) {
    let at = daemon.clock.now_ms();
    update(daemon, bot, |entry| {
        entry.size = Some(Size {
            used,
            window,
            auto_compact: None,
            at,
            parts: Vec::new(),
            memory_files: Vec::new(),
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
    update(daemon, bot, |entry| {
        if !entry.compact_on_wake {
            entry.compacting = false;
        }
    });
}
