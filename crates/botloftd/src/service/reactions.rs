//! `reactions.*` operations (spec 8.9): the owner's emoji on a bot's
//! reply. Nothing reaches the bot here: the owner's next message takes the
//! waiting ones along.

use botloft_core::chat;
use botloft_core::envelope::QUOTED_REACTION_MAX_CHARS;
use botloft_core::protocol::{
    ChatBody, Message, REACTIONS, Reaction, ReactionChanged, ReactionsListParams,
    ReactionsSetParams, SenderKind,
};
use botloft_store::Store;

use super::{ApiError, ApiResult, bots};
use crate::state::{Daemon, Event};

pub fn list(daemon: &Daemon, params: ReactionsListParams) -> ApiResult<Vec<Reaction>> {
    let store = daemon.store();
    bots::active(&store, &params.bot_id)?;
    Ok(store.reactions(&params.bot_id)?)
}

/// Puts, changes or takes off the owner's reaction on one of the bot's
/// replies; the reaction as it is now.
pub fn set(daemon: &Daemon, params: ReactionsSetParams) -> ApiResult<Option<Reaction>> {
    if let Some(emoji) = &params.emoji
        && !REACTIONS.contains(&emoji.as_str())
    {
        return Err(ApiError::validation(format!(
            "emoji must be one of {}",
            REACTIONS.join(" ")
        )));
    }
    let store = daemon.store();
    let (_, bot) = bots::active(&store, &params.bot_id)?;
    let item = store
        .chat_item(&params.item_id)?
        .filter(|item| item.bot_id == bot.id)
        .ok_or_else(|| {
            ApiError::NotFound(format!("chat item {} does not exist", params.item_id))
        })?;
    let ChatBody::Reply(reply) = &item.body else {
        return Err(ApiError::validation(
            "only a reply of the bot takes a reaction",
        ));
    };
    let reaction = match params.emoji {
        Some(emoji) => {
            let reaction = Reaction {
                bot_id: bot.id.clone(),
                item_id: item.id.clone(),
                emoji,
                created_at: daemon.clock.now_ms(),
                sent_in: None,
            };
            store.set_reaction(
                &reaction,
                &chat::one_line(&reply.text, QUOTED_REACTION_MAX_CHARS),
            )?;
            Some(reaction)
        }
        None if store.remove_reaction(&item.id)? => None,
        // There was none: nothing changed.
        None => return Ok(None),
    };
    drop(store);
    daemon.emit(Event::ReactionChanged(ReactionChanged {
        bot_id: bot.id,
        item_id: item.id,
        reaction: reaction.clone(),
    }));
    Ok(reaction)
}

/// The reactions an owner's message just took to its bot, to tell the app.
pub(crate) fn sent_with(store: &Store, message: &Message) -> ApiResult<Vec<Reaction>> {
    if message.from_kind != SenderKind::Owner {
        return Ok(Vec::new());
    }
    Ok(store
        .reactions_sent_in(&message.id)?
        .into_iter()
        .map(|(reaction, _)| reaction)
        .collect())
}

pub(crate) fn announce_sent(daemon: &Daemon, sent: Vec<Reaction>) {
    for reaction in sent {
        daemon.emit(Event::ReactionChanged(ReactionChanged {
            bot_id: reaction.bot_id.clone(),
            item_id: reaction.item_id.clone(),
            reaction: Some(reaction),
        }));
    }
}
