//! Saving chat items and telling the app (spec 8.3).

use botloft_core::chat::activity;
use botloft_core::ids::{BotId, ChatItemId};
use botloft_core::protocol::{ChatBody, ChatItem, ChatItemChanged};
use tracing::warn;

use crate::state::{Daemon, Event};

/// Saves a new item for `bot` and announces it with the bot's new
/// conversation-list line.
pub(crate) fn add(daemon: &Daemon, bot: &BotId, body: ChatBody) -> Option<ChatItem> {
    let now = daemon.clock.now_ms();
    let item = ChatItem {
        id: ChatItemId::generate(),
        bot_id: bot.clone(),
        body,
        created_at: now,
        updated_at: now,
    };
    if let Err(err) = daemon.store().insert_chat_item(&item) {
        warn!(bot = %bot, "could not save a chat item: {err}");
        return None;
    }
    announce(daemon, item.clone());
    Some(item)
}

/// Announces an item that is already saved, e.g. the one a message created.
pub(crate) fn announce(daemon: &Daemon, item: ChatItem) {
    let activity = activity(&item.body, item.updated_at);
    daemon.emit(Event::ChatItem(ChatItemChanged { item, activity }));
}

/// Announces an item saved again with a new body, which leaves the
/// conversation-list line as it was.
pub(crate) fn changed(daemon: &Daemon, item: ChatItem) {
    daemon.emit(Event::ChatItem(ChatItemChanged {
        item,
        activity: None,
    }));
}

/// Replaces an item's body, e.g. a tool that finished, and announces it.
pub(crate) fn update(daemon: &Daemon, id: &ChatItemId, body: ChatBody) -> Option<ChatItem> {
    let now = daemon.clock.now_ms();
    match daemon.store().update_chat_item(id, &body, now) {
        Ok(Some(item)) => {
            daemon.emit(Event::ChatItem(ChatItemChanged {
                item: item.clone(),
                activity: None,
            }));
            Some(item)
        }
        Ok(None) => None,
        Err(err) => {
            warn!(item = %id, "could not update a chat item: {err}");
            None
        }
    }
}
