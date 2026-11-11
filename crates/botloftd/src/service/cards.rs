//! Cards a bot shows in its chat (spec 8.11): showing or showing again,
//! from the bot's tool, and `chat.act`, the owner's answer on one version,
//! which goes to the bot as the owner's message.

use botloft_core::envelope::QUOTED_REPLY_MAX_CHARS;
use botloft_core::ids::{BotId, ChatItemId, MessageId};
use botloft_core::protocol::{
    CardAnswer, CardBlock, CardItem, ChatActParams, ChatBody, ChatItem, Message, MessageKind,
    MessageReply, SenderKind,
};
use botloft_core::{cards, chat, validate};
use botloft_store::Store;

use super::{ApiError, ApiResult, bots, messages, reactions};
use crate::chat::items;
use crate::state::Daemon;

/// What a bot shows: a new card, or `card` again, changed.
pub struct Show {
    pub card: Option<ChatItemId>,
    pub title: String,
    pub blocks: Vec<CardBlock>,
}

/// Checks the card and shows it; returns its id and version.
pub fn show(daemon: &Daemon, bot: &BotId, show: Show) -> ApiResult<(ChatItemId, u32)> {
    let (title, blocks) = cards::check(&show.title, show.blocks)?;
    let version = {
        let store = daemon.store();
        bots::active(&store, bot)?;
        match &show.card {
            Some(id) => card_of(&store, id, Some(bot))?.1.version + 1,
            None => 1,
        }
    };
    let body = ChatBody::Card(CardItem {
        version,
        title,
        blocks,
        answer: None,
    });
    let item = match &show.card {
        // Shown again where it was: the chat does not move.
        Some(id) => items::update(daemon, bot, id, body),
        None => items::add(daemon, bot, body),
    }
    .ok_or_else(|| ApiError::Conflict("could not save the card".to_owned()))?;
    Ok((item.id, version))
}

/// The card `id`, of `bot` when given.
fn card_of(store: &Store, id: &ChatItemId, bot: Option<&BotId>) -> ApiResult<(ChatItem, CardItem)> {
    let missing = || ApiError::NotFound(format!("card {id} does not exist"));
    let item = store
        .chat_item(id)?
        .filter(|item| bot.is_none_or(|bot| &item.bot_id == bot))
        .ok_or_else(missing)?;
    match &item.body {
        ChatBody::Card(card) => {
            let card = card.clone();
            Ok((item, card))
        }
        _ => Err(missing()),
    }
}

/// The owner presses a button of the version they saw, or sends its form:
/// the card keeps the answer and the bot gets it as the owner's message.
/// Each version takes one answer.
pub fn act(daemon: &Daemon, params: ChatActParams) -> ApiResult<ChatItem> {
    let store = daemon.store();
    let (item, card) = card_of(&store, &params.item_id, None)?;
    let (crew, bot) = bots::active(&store, &item.bot_id)?;
    if params.version != card.version {
        return Err(ApiError::Conflict(format!(
            "{} changed this card since you saw it; look at the new one",
            bot.name
        )));
    }
    if card.answer.is_some() {
        return Err(ApiError::Conflict(format!(
            "this card was already answered; {} shows it again to ask once more",
            bot.name
        )));
    }
    let (button, text) = cards::answer(&card, params.button.as_deref(), params.form.as_ref())?;
    let body = validate::message("answer", &text)?;
    let now = daemon.clock.now_ms();
    let message = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::Owner,
        from_bot_id: None,
        to_bot_id: bot.id,
        kind: MessageKind::Note,
        body,
        task_id: None,
        routine_id: None,
        question_id: None,
        // It answers the card: the chat shows the title above it.
        reply_to: Some(MessageReply {
            item_id: item.id.clone(),
            text: chat::one_line(&card.title, QUOTED_REPLY_MAX_CHARS),
        }),
        attachments: Vec::new(),
        created_at: now,
    };
    let delivery = messages::pending_delivery(&message);
    let answered = ChatBody::Card(CardItem {
        answer: Some(CardAnswer {
            version: card.version,
            button,
            text: message.body.clone(),
            at: now,
        }),
        ..card
    });
    let (card_item, answer_item) = store
        .answer_card(&item.id, &answered, &message, &delivery)?
        .ok_or_else(|| ApiError::NotFound(format!("card {} does not exist", item.id)))?;
    let reactions = reactions::sent_with(&store, &message)?;
    drop(store);
    items::changed(daemon, card_item.clone());
    messages::announce(daemon, None, &message, delivery, answer_item);
    reactions::announce_sent(daemon, reactions);
    Ok(card_item)
}
