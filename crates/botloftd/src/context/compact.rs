//! Compacting a conversation on the owner's word (spec 8.6), and
//! forgetting it while it sleeps (7.7). Each conversation of a bot
//! compacts on its own (spec 40.5).

use botloft_core::ids::random_uuid;
use botloft_core::protocol::{
    AgentKind, Bot, BotConversationParams, BotState, ChatBody, NoticeCode, NoticeItem, NoticeLevel,
};
use botloft_store::{BotRecord, Store};
use bytes::Bytes;
use serde_json::{Value, json};
use tracing::warn;

use super::{COMPACT, ask, update};
use crate::chat::items;
use crate::seat::Seat;
use crate::service::bots::{active, to_protocol};
use crate::service::conversations::seat_of;
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// `bots.compact`: writes `/compact` to the conversation's process like a
/// message, so it waits its turn behind what it is doing. A sleeping one
/// wakes first, and compacts as soon as it is up. Asking again while one
/// is on its way changes nothing.
pub fn compact(daemon: &Daemon, params: BotConversationParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, record) = active(&store, &params.bot_id)?;
    let agent = crate::agent::of(record.agent);
    if !agent.is_some_and(|agent| agent.can_compact()) {
        return Err(ApiError::validation(format!(
            "bots that run on {} cannot compact their conversation",
            record.agent.as_str()
        )));
    }
    let seat = seat_of(&store, &record, params.conversation_id, true)?;
    if !daemon.contexts.compacting(&seat.conversation) {
        match daemon.supervisor.status_of(&seat.conversation) {
            Some((BotState::Idle | BotState::Busy | BotState::NeedsApproval, _)) => {
                send(daemon, &record, &seat)?;
                update(daemon, &seat, |entry| entry.compacting = true);
            }
            Some((BotState::Sleeping, _)) if !record.paused && !crew.paused => {
                update(daemon, &seat, |entry| {
                    entry.compacting = true;
                    entry.compact_on_wake = true;
                });
                daemon.supervisor.wake_conversation(&seat.conversation);
            }
            _ => {
                return Err(ApiError::Conflict(format!(
                    "bot {} is not running",
                    record.id
                )));
            }
        }
    }
    Ok(to_protocol(daemon, &store, &crew, record))
}

/// The conversation's process is up: a compaction asked while it slept
/// goes now.
pub(crate) fn woke(daemon: &Daemon, seat: &Seat) {
    let mut asked = false;
    update(daemon, seat, |entry| {
        asked = std::mem::take(&mut entry.compact_on_wake);
    });
    if !asked {
        return;
    }
    let record = active(&daemon.store(), seat).map(|(_, record)| record);
    let sent = record.and_then(|record| send(daemon, &record, seat));
    if let Err(err) = sent {
        warn!(bot = %seat, "a compaction asked while the bot slept did not go: {err}");
        update(daemon, seat, |entry| entry.compacting = false);
    }
}

/// The request itself, as each agent takes it.
fn send(daemon: &Daemon, record: &BotRecord, seat: &Seat) -> ApiResult<()> {
    let not_running = || ApiError::Conflict(format!("bot {} is not running", record.id));
    if record.agent == AgentKind::Codex {
        // Codex compacts as a turn of its own, asked by its wrapper.
        let line = Bytes::from("{\"event\":\"compact\"}\n");
        return daemon
            .supervisor
            .write_control_to(&seat.conversation, line)
            .map_err(|_| not_running());
    }
    let uuid = random_uuid();
    let line = json!({
        "type": "user",
        "uuid": uuid,
        "message": { "role": "user", "content": [{ "type": "text", "text": COMPACT }] },
    });
    daemon
        .supervisor
        .write_message_to(
            &record.id,
            &seat.conversation,
            &uuid,
            Bytes::from(format!("{line}\n")),
        )
        .map(|_| ())
        .map_err(|_| not_running())
}

/// A sleeping conversation starts its next process in a new session (spec
/// 7.7): nothing to count until it tells. `false` when it does not sleep.
/// The caller holds the store.
pub(crate) fn forget_while_asleep(daemon: &Daemon, store: &Store, seat: &Seat) -> bool {
    if !daemon.supervisor.start_fresh_asleep(&seat.conversation) {
        return false;
    }
    if let Err(err) = store.set_conversation_session(seat, &seat.conversation, None) {
        warn!(bot = %seat, "could not forget the conversation of a sleeping bot: {err}");
    }
    update(daemon, seat, |entry| {
        entry.size = None;
        entry.compacting = false;
        entry.compact_on_wake = false;
    });
    true
}

/// `system/compact_boundary`: the conversation is now a summary. The chat
/// says so, and Claude Code is asked for the new size.
pub(crate) fn compacted(daemon: &Daemon, bot: &Seat, event: &Value) {
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
    items::add_to(
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
pub(crate) fn command_output(daemon: &Daemon, bot: &Seat, event: &Value) -> bool {
    if event["local_command_run"]["command"] != "compact" {
        return false;
    }
    update(daemon, bot, |entry| entry.compacting = false);
    if event["local_command_outcome"]["kind"] == "failed" {
        let detail = event["message"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default();
        let detail = detail.strip_prefix("Error: ").unwrap_or(detail);
        items::add_to(
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
