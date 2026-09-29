//! `messages.*` operations, and posting a message for the courier to deliver
//! (spec 9.1).

use botloft_core::ids::{DeliveryId, MessageId};
use botloft_core::protocol::{
    ChatItem, Delivery, DeliveryState, Message, MessageKind, MessagesListParams,
    MessagesSendParams, SenderKind, Task,
};
use botloft_core::validate;
use botloft_store::{MessageFilter, Store};

use super::{ApiError, ApiResult, attachments, bots, crews};
use crate::chat::items;
use crate::state::{Daemon, Event};

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;

/// The owner writes to a bot, with files if any (spec 9.5). A paused bot
/// gets it once it runs again.
pub fn send(daemon: &Daemon, params: MessagesSendParams) -> ApiResult<Message> {
    let uploads = params.attachments.unwrap_or_default();
    let body = if uploads.is_empty() || !params.body.trim().is_empty() {
        validate::message("body", &params.body)?
    } else {
        // Files alone are a message too.
        String::new()
    };
    let store = daemon.store();
    let (crew, bot) = bots::active(&store, &params.bot_id)?;
    let now = daemon.clock.now_ms();
    let workspace = daemon.paths.bot_workspace(&crew.slug, &bot.slug);
    let attachments =
        attachments::save(&workspace, uploads, daemon.bots.attachment_max_bytes, now)?;
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
        attachments,
        created_at: now,
    };
    post(daemon, &store, message, None)
}

/// Stores the message with a pending delivery (and the task it creates),
/// tells the app and wakes the courier.
pub(crate) fn post(
    daemon: &Daemon,
    store: &Store,
    message: Message,
    task: Option<Task>,
) -> ApiResult<Message> {
    let delivery = pending_delivery(&message);
    let item = store.insert_message(&message, &delivery, task.as_ref())?;
    announce(daemon, task, &message, delivery, item);
    Ok(message)
}

/// A new delivery of `message`, due now.
pub(crate) fn pending_delivery(message: &Message) -> Delivery {
    Delivery {
        id: DeliveryId::generate(),
        message_id: message.id.clone(),
        bot_id: message.to_bot_id.clone(),
        state: DeliveryState::Pending,
        attempts: 0,
        next_attempt_at: message.created_at,
        last_error: None,
        read_at: None,
        updated_at: message.created_at,
    }
}

/// Tells the app about a stored message (and the task it changed) and wakes
/// the courier for it.
pub(crate) fn announce(
    daemon: &Daemon,
    task: Option<Task>,
    message: &Message,
    delivery: Delivery,
    item: ChatItem,
) {
    if let Some(task) = task {
        daemon.emit(Event::TaskChanged(task));
    }
    daemon.emit(Event::MessageCreated(message.clone()));
    daemon.emit(Event::DeliveryChanged(delivery));
    items::announce(daemon, item);
    daemon.courier.wake();
}

pub fn list(daemon: &Daemon, params: MessagesListParams) -> ApiResult<Vec<Message>> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(ApiError::validation(format!(
            "limit must be between 1 and {MAX_LIMIT}"
        )));
    }
    let store = daemon.store();
    if let Some(crew) = &params.crew_id {
        crews::find(&store, crew)?;
    }
    if let Some(bot) = &params.bot_id {
        bots::find(&store, bot)?;
    }
    if let Some(before) = &params.before
        && store.message(before)?.is_none()
    {
        return Err(ApiError::NotFound(format!(
            "message {before} does not exist"
        )));
    }
    Ok(store.messages(MessageFilter {
        crew: params.crew_id.as_ref(),
        bot: params.bot_id.as_ref(),
        before: params.before.as_ref(),
        limit,
    })?)
}
