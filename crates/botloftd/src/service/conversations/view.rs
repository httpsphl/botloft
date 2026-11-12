//! What the app is told of a conversation: the stored one with what is
//! live (its process, context and background), and `conversation.changed`.

use std::sync::Arc;

use botloft_core::ids::ConversationId;
use botloft_core::protocol::{BotState, Conversation};
use botloft_store::{ConversationRecord, Store};
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;

use crate::state::{Daemon, Event};

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
    let read = if record.main {
        store.read_through(&record.bot_id)
    } else {
        store.conversation_read_through(&record.id)
    };
    let read_through = read.unwrap_or_else(|err| {
        warn!(bot = %record.bot_id, "could not read what the owner read: {err}");
        None
    });
    let last_reply_at = store
        .conversation_last_reply(&record.id)
        .unwrap_or_else(|err| {
            warn!(bot = %record.bot_id, "could not read a conversation's last reply: {err}");
            None
        });
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
        last_reply_at,
        read_through,
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
