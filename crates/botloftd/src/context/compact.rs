//! Compacting a bot's conversation on the owner's word (spec 8.6), and
//! forgetting it while the bot sleeps (7.7).

use botloft_core::ids::{BotId, random_uuid};
use botloft_core::protocol::{AgentKind, Bot, BotIdParams, BotState};
use botloft_store::{BotRecord, Store};
use bytes::Bytes;
use serde_json::json;
use tracing::warn;

use super::{COMPACT, update};
use crate::service::bots::{active, to_protocol};
use crate::service::{ApiError, ApiResult};
use crate::state::Daemon;

/// `bots.compact`: writes `/compact` to the bot's process like a message,
/// so it waits its turn behind what the bot is doing. A sleeping bot wakes
/// first, and compacts as soon as it is up. Asking again while one is on
/// its way changes nothing.
pub fn compact(daemon: &Daemon, params: BotIdParams) -> ApiResult<Bot> {
    let store = daemon.store();
    let (crew, record) = active(&store, &params.bot_id)?;
    let agent = crate::agent::of(record.agent);
    if !agent.is_some_and(|agent| agent.can_compact()) {
        return Err(ApiError::validation(format!(
            "bots that run on {} cannot compact their conversation",
            record.agent.as_str()
        )));
    }
    if !daemon.contexts.compacting(&record.id) {
        match daemon.supervisor.status(&record.id) {
            Some((BotState::Idle | BotState::Busy | BotState::NeedsApproval, _)) => {
                send(daemon, &record)?;
                update(daemon, &record.id, |entry| entry.compacting = true);
            }
            Some((BotState::Sleeping, _)) if !record.paused && !crew.paused => {
                update(daemon, &record.id, |entry| {
                    entry.compacting = true;
                    entry.compact_on_wake = true;
                });
                daemon.supervisor.wake_up(&record.id);
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

/// The bot's process is up: a compaction asked while it slept goes now.
pub(crate) fn woke(daemon: &Daemon, bot: &BotId) {
    let mut asked = false;
    update(daemon, bot, |entry| {
        asked = std::mem::take(&mut entry.compact_on_wake);
    });
    if !asked {
        return;
    }
    let record = active(&daemon.store(), bot).map(|(_, record)| record);
    let sent = record.and_then(|record| send(daemon, &record));
    if let Err(err) = sent {
        warn!(bot = %bot, "a compaction asked while the bot slept did not go: {err}");
        update(daemon, bot, |entry| entry.compacting = false);
    }
}

/// The request itself, as each agent takes it.
fn send(daemon: &Daemon, record: &BotRecord) -> ApiResult<()> {
    let not_running = || ApiError::Conflict(format!("bot {} is not running", record.id));
    if record.agent == AgentKind::Codex {
        // Codex compacts as a turn of its own, asked by its wrapper.
        let line = Bytes::from("{\"event\":\"compact\"}\n");
        return daemon
            .supervisor
            .write_control(&record.id, line)
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
        .write_message(&record.id, &uuid, Bytes::from(format!("{line}\n")))
        .map(|_| ())
        .map_err(|_| not_running())
}

/// A sleeping bot starts its next process in a new conversation (spec
/// 7.7): nothing to count until it tells. `false` when it does not sleep.
/// The caller holds the store.
pub(crate) fn forget_while_asleep(daemon: &Daemon, store: &Store, bot: &BotId) -> bool {
    if !daemon.supervisor.start_fresh_asleep(bot) {
        return false;
    }
    if let Err(err) = store.set_session_id(bot, None) {
        warn!(bot = %bot, "could not forget the conversation of a sleeping bot: {err}");
    }
    update(daemon, bot, |entry| {
        entry.size = None;
        entry.compacting = false;
        entry.compact_on_wake = false;
    });
    true
}
