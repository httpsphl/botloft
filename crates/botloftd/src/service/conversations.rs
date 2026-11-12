//! `conversations.*` (spec 40.5): the owner lists, opens, names and
//! archives the conversations of an agent, and the app hears of each one
//! as `conversation.changed`.

use std::sync::Arc;

use botloft_core::ids::ConversationId;
use botloft_core::protocol::{
    AgentKind, BotIdParams, BotState, Conversation, ConversationIdParams,
    ConversationsRenameParams, TitleBy,
};
use botloft_core::validate;
use botloft_store::{BotRecord, ConversationRecord, Store};
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

use super::bots::{active, find};
use super::{ApiError, ApiResult};
use crate::seat::Seat;
use crate::state::{Daemon, Event};

/// The most conversations an agent keeps open at once.
pub const MAX_OPEN: usize = 50;

pub fn list(daemon: &Daemon, params: BotIdParams) -> ApiResult<Vec<Conversation>> {
    let store = daemon.store();
    find(&store, &params.bot_id)?;
    let mut all: Vec<Conversation> = store
        .conversations(Some(&params.bot_id))?
        .into_iter()
        .map(|record| view(daemon, &store, record))
        .collect();
    // The main one first, then the last one used.
    all.sort_by(|a, b| {
        b.main
            .cmp(&a.main)
            .then(b.last_activity.cmp(&a.last_activity))
    });
    Ok(all)
}

pub fn archived(daemon: &Daemon, params: BotIdParams) -> ApiResult<Vec<Conversation>> {
    let store = daemon.store();
    find(&store, &params.bot_id)?;
    Ok(store
        .archived_conversations(&params.bot_id)?
        .into_iter()
        .map(|record| view(daemon, &store, record))
        .collect())
}

/// A new conversation, asleep until its first message (spec 40.1).
pub fn create(daemon: &Daemon, params: BotIdParams) -> ApiResult<Conversation> {
    let store = daemon.store();
    let (_, record) = active(&store, &params.bot_id)?;
    if record.agent != AgentKind::Claude {
        return Err(ApiError::validation(format!(
            "agents that run on {} have only their main conversation",
            record.agent.as_str()
        )));
    }
    if store.conversations(Some(&record.id))?.len() >= MAX_OPEN {
        return Err(ApiError::validation(format!(
            "an agent keeps at most {MAX_OPEN} conversations open: archive one first"
        )));
    }
    let conversation = ConversationRecord::new(&record.id, daemon.clock.now_ms());
    store.insert_conversation(&conversation)?;
    let made = view(daemon, &store, conversation);
    drop(store);
    daemon.emit(Event::ConversationChanged(made.clone()));
    // Its place in the supervisor comes with the next pass.
    daemon.supervisor.wake();
    Ok(made)
}

/// The owner's name, which the AI never changes (spec 40.6).
pub fn rename(daemon: &Daemon, params: ConversationsRenameParams) -> ApiResult<Conversation> {
    let title = validate::name("title", &params.title)?;
    let store = daemon.store();
    let record = other(&store, &params.conversation_id, "renamed")?;
    store.set_conversation_title(&record.id, Some(&title), Some(TitleBy::Owner.as_str()))?;
    announce(daemon, store, &record.id)
}

/// Stops its process and takes it off the list; its session stays on disk
/// for when it comes back (spec 40.3). Archiving twice is not an error.
pub fn archive(daemon: &Daemon, params: ConversationIdParams) -> ApiResult<Conversation> {
    let store = daemon.store();
    let record = other(&store, &params.conversation_id, "archived")?;
    if record.archived_at.is_none() {
        let now = daemon.clock.now_ms();
        store.set_conversation_archived(&record.id, Some(now))?;
        // What still waited for it goes to the main one (40.2), now.
        let main = ConversationId::main_of(&record.bot_id);
        if store.move_deliveries(&record.id, &main)? > 0 {
            store.hasten_deliveries(&record.bot_id, now)?;
            daemon.courier.wake();
        }
    }
    let shown = announce(daemon, store, &record.id)?;
    daemon.supervisor.wake();
    Ok(shown)
}

/// Back on the list, asleep until its next message.
pub fn unarchive(daemon: &Daemon, params: ConversationIdParams) -> ApiResult<Conversation> {
    let store = daemon.store();
    let record = other(&store, &params.conversation_id, "archived")?;
    active(&store, &record.bot_id)?;
    if record.archived_at.is_some() {
        if store.conversations(Some(&record.bot_id))?.len() >= MAX_OPEN {
            return Err(ApiError::validation(format!(
                "an agent keeps at most {MAX_OPEN} conversations open: archive one first"
            )));
        }
        store.set_conversation_archived(&record.id, None)?;
    }
    let shown = announce(daemon, store, &record.id)?;
    daemon.supervisor.wake();
    Ok(shown)
}

/// A conversation that is not the main one, which is never renamed or
/// archived.
fn other(store: &Store, id: &ConversationId, done: &str) -> ApiResult<ConversationRecord> {
    let record = store
        .conversation(id)?
        .ok_or_else(|| ApiError::NotFound(format!("conversation {id} does not exist")))?;
    if record.main {
        return Err(ApiError::validation(format!(
            "the main conversation cannot be {done}"
        )));
    }
    Ok(record)
}

/// Tells the app what `id` is now and gives it back.
fn announce(
    daemon: &Daemon,
    store: std::sync::MutexGuard<'_, Store>,
    id: &ConversationId,
) -> ApiResult<Conversation> {
    let record = store
        .conversation(id)?
        .ok_or_else(|| ApiError::NotFound(format!("conversation {id} does not exist")))?;
    let shown = view(daemon, &store, record);
    drop(store);
    daemon.emit(Event::ConversationChanged(shown.clone()));
    Ok(shown)
}

/// The conversation of `bot` a request names: the main one when it names
/// none. With `open`, an archived one is refused (spec 40.5).
pub(crate) fn seat_of(
    store: &Store,
    bot: &BotRecord,
    asked: Option<ConversationId>,
    open: bool,
) -> ApiResult<Seat> {
    let Some(asked) = asked else {
        return Ok(Seat::main(&bot.id));
    };
    if asked.is_main_of(&bot.id) {
        return Ok(Seat::new(bot.id.clone(), asked));
    }
    match store.conversation(&asked)? {
        Some(found) if found.bot_id == bot.id && (!open || found.archived_at.is_none()) => {
            Ok(Seat::new(bot.id.clone(), asked))
        }
        _ => Err(ApiError::NotFound(format!(
            "conversation {asked} of bot {} does not exist",
            bot.id
        ))),
    }
}

/// The protocol view of a stored conversation, with what is live.
pub(crate) fn view(daemon: &Daemon, store: &Store, record: ConversationRecord) -> Conversation {
    let (state, generation) = if record.archived_at.is_some() {
        (BotState::Archived, None)
    } else {
        // One the supervisor has not placed yet starts asleep (40.1).
        let unseen = if record.main {
            BotState::Offline
        } else {
            BotState::Sleeping
        };
        daemon
            .supervisor
            .status_of(&record.id)
            .unwrap_or((unseen, None))
    };
    let last_activity = store
        .conversation_activity(&record.id)
        .unwrap_or_else(|err| {
            warn!(bot = %record.bot_id, "could not read a conversation's activity: {err}");
            None
        })
        .unwrap_or(record.created_at);
    Conversation {
        context: daemon.contexts.get_in(&record.id),
        background: daemon.background.get_in(&record.id),
        title_by: record.title_by.as_deref().and_then(|by| by.parse().ok()),
        id: record.id,
        bot_id: record.bot_id,
        title: record.title,
        main: record.main,
        state,
        generation,
        last_activity,
        created_at: record.created_at,
        archived_at: record.archived_at,
    }
}

/// Tells the app what a conversation is now. The caller holds no store.
pub(crate) fn changed(daemon: &Daemon, id: &ConversationId) {
    let shown = {
        let store = daemon.store();
        match store.conversation(id) {
            Ok(Some(record)) => view(daemon, &store, record),
            Ok(None) => return,
            Err(err) => {
                warn!("could not read a conversation: {err}");
                return;
            }
        }
    };
    daemon.emit(Event::ConversationChanged(shown));
}

/// Turns what changed in a conversation's process, context or background
/// into `conversation.changed`. The supervisor and the trackers cannot
/// read the store where they change, so they only say which one.
pub async fn follow(daemon: Arc<Daemon>) {
    let mut events = daemon.subscribe();
    loop {
        let touched = match events.recv().await {
            Ok(Event::SeatState(change)) => Some(change.seat.conversation),
            Ok(Event::ConversationTouched(seat)) => Some(seat.conversation),
            Ok(_) | Err(RecvError::Lagged(_)) => None,
            Err(RecvError::Closed) => return,
        };
        if let Some(id) = touched {
            changed(&daemon, &id);
        }
    }
}
