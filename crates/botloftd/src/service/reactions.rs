//! `reactions.*` operations (spec 8.9): the owner's emoji on a bot's
//! reply. Most wait for the owner's next message, which takes them along;
//! a 👍 or 👎 on a reply that asks something goes at once, as the owner's
//! answer.

use botloft_core::envelope::QUOTED_REACTION_MAX_CHARS;
use botloft_core::ids::MessageId;
use botloft_core::protocol::{
    ChatBody, Crew, Message, MessageKind, REACTIONS, Reaction, ReactionChanged,
    ReactionsListParams, ReactionsSetParams, SenderKind,
};
use botloft_core::{asks, chat};
use botloft_store::{BotRecord, Store};

use super::{ApiError, ApiResult, bots, messages};
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
    let (crew, bot) = bots::active(&store, &params.bot_id)?;
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
            // A thumb on a reply that asks something is the answer to it.
            let question = asks::ANSWERS
                .iter()
                .any(|(answer, _)| *answer == emoji)
                .then(|| asks::asked(&reply.text, QUOTED_REACTION_MAX_CHARS))
                .flatten();
            let before = store
                .reactions(&bot.id)?
                .into_iter()
                .find(|one| one.item_id == item.id);
            let answers = question.is_some();
            if answers
                && let Some(before) = before
                && before.emoji == emoji
                && before.sent_in.is_some()
            {
                // That answer already went.
                return Ok(Some(before));
            }
            let mut reaction = Reaction {
                bot_id: bot.id.clone(),
                item_id: item.id.clone(),
                emoji,
                created_at: daemon.clock.now_ms(),
                sent_in: None,
            };
            let quote =
                question.unwrap_or_else(|| chat::one_line(&reply.text, QUOTED_REACTION_MAX_CHARS));
            store.set_reaction(&reaction, &quote)?;
            if answers {
                // The owner's message takes it, and tells the app.
                let message = answer(&crew, &bot, &reaction);
                reaction.sent_in = Some(messages::post(daemon, &store, message, None)?.id);
                return Ok(Some(reaction));
            }
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

/// The owner's message that answers with a thumb: the emoji alone, which
/// shows in the chat like any message of theirs. The bot reads the answer
/// in the reaction it takes along (spec 9.3).
fn answer(crew: &Crew, bot: &BotRecord, reaction: &Reaction) -> Message {
    Message {
        id: MessageId::generate(),
        crew_id: crew.id.clone(),
        from_kind: SenderKind::Owner,
        from_bot_id: None,
        to_bot_id: bot.id.clone(),
        kind: MessageKind::Note,
        body: reaction.emoji.clone(),
        task_id: None,
        routine_id: None,
        question_id: None,
        reply_to: None,
        attachments: Vec::new(),
        created_at: reaction.created_at,
    }
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
