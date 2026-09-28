//! `messages.*` operations, and posting a message for the courier to deliver
//! (spec 9.1).

use botloft_core::ids::{DeliveryId, MessageId};
use botloft_core::protocol::{
    Delivery, DeliveryState, Message, MessageKind, MessagesListParams, MessagesSendParams,
    SenderKind, Task,
};
use botloft_core::validate;
use botloft_store::{MessageFilter, Store};

use super::{ApiError, ApiResult, bots, crews};
use crate::state::{Daemon, Event};

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;

/// The owner writes to a bot. A paused bot gets it once it runs again.
pub fn send(daemon: &Daemon, params: MessagesSendParams) -> ApiResult<Message> {
    let body = validate::message("body", &params.body)?;
    let store = daemon.store();
    let (crew, bot) = bots::active(&store, &params.bot_id)?;
    let message = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::Owner,
        from_bot_id: None,
        to_bot_id: bot.id,
        kind: MessageKind::Note,
        body,
        task_id: None,
        created_at: daemon.clock.now_ms(),
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
    let delivery = Delivery {
        id: DeliveryId::generate(),
        message_id: message.id.clone(),
        bot_id: message.to_bot_id.clone(),
        state: DeliveryState::Pending,
        attempts: 0,
        next_attempt_at: message.created_at,
        last_error: None,
        updated_at: message.created_at,
    };
    store.insert_message(&message, &delivery, task.as_ref())?;
    if let Some(task) = task {
        daemon.emit(Event::TaskChanged(task));
    }
    daemon.emit(Event::MessageCreated(message.clone()));
    daemon.emit(Event::DeliveryChanged(delivery));
    daemon.courier.wake();
    Ok(message)
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
