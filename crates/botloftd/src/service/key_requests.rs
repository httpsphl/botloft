//! A bot asks the owner for a key (spec 32.5): `ask_key` opens a card in its
//! chat, which is the request itself; the owner saves the key with
//! `keys.set` and the request's id, or says "not now" with `keys.dismiss`.
//! Either way the bot hears it in a message from Botloft.

use botloft_core::ids::{BotId, ChatItemId, MessageId};
use botloft_core::keys;
use botloft_core::protocol::{
    ChatBody, KeyRequestItem, KeyRequestStatus, KeysDismissParams, Message, MessageKind, SenderKind,
};
use tracing::warn;

use super::{ApiError, ApiResult, bots, messages};
use crate::chat::items;
use crate::state::Daemon;

/// Requests a bot may leave open at once.
pub const OPEN_KEY_REQUESTS_MAX: u32 = 5;
/// Longest reason, in characters.
pub const WHY_MAX_CHARS: usize = 500;

/// Checks the request and shows it in the bot's chat; the item's id is the
/// request's.
pub fn ask(daemon: &Daemon, bot: &BotId, name: &str, why: &str) -> ApiResult<ChatItemId> {
    let name = keys::name(name)?;
    let why = why.trim();
    if why.is_empty() || why.chars().count() > WHY_MAX_CHARS {
        return Err(ApiError::validation(format!(
            "why must say in at most {WHY_MAX_CHARS} characters what you need the key for"
        )));
    }
    {
        let store = daemon.store();
        bots::active(&store, bot)?;
        if store.open_key_requests(bot)? >= OPEN_KEY_REQUESTS_MAX {
            return Err(ApiError::Conflict(format!(
                "you already have {OPEN_KEY_REQUESTS_MAX} requests for keys the owner has not \
                 answered; wait for an answer before asking again"
            )));
        }
    }
    let item = items::add(
        daemon,
        bot,
        ChatBody::KeyRequest(KeyRequestItem {
            name,
            why: why.to_owned(),
            status: KeyRequestStatus::Open,
        }),
    )
    .ok_or_else(|| ApiError::Conflict("could not save the request".to_owned()))?;
    Ok(item.id)
}

/// The open request `id` of `bot`, if it asks for `name`.
pub fn open(
    daemon: &Daemon,
    bot: &BotId,
    id: &ChatItemId,
    name: &str,
) -> ApiResult<KeyRequestItem> {
    let item = daemon
        .store()
        .chat_item(id)?
        .filter(|item| &item.bot_id == bot)
        .ok_or_else(|| ApiError::NotFound(format!("request {id} does not exist")))?;
    let ChatBody::KeyRequest(request) = item.body else {
        return Err(ApiError::NotFound(format!("request {id} does not exist")));
    };
    if request.status != KeyRequestStatus::Open {
        return Err(ApiError::Conflict(format!(
            "request {id} was already answered"
        )));
    }
    if request.name != name {
        return Err(ApiError::validation(format!(
            "the request asks for {}, not {name}",
            request.name
        )));
    }
    Ok(request)
}

/// Closes the request as `status` and tells the bot `body`.
pub fn close(
    daemon: &Daemon,
    bot: &BotId,
    id: &ChatItemId,
    mut request: KeyRequestItem,
    status: KeyRequestStatus,
    body: String,
) -> ApiResult<()> {
    request.status = status;
    items::update(daemon, bot, id, ChatBody::KeyRequest(request));
    let store = daemon.store();
    let (crew, _) = bots::active(&store, bot)?;
    let note = Message {
        id: MessageId::generate(),
        crew_id: crew.id,
        from_kind: SenderKind::System,
        from_bot_id: None,
        to_bot_id: bot.clone(),
        kind: MessageKind::System,
        body,
        task_id: None,
        routine_id: None,
        question_id: None,
        reply_to: None,
        attachments: Vec::new(),
        created_at: daemon.clock.now_ms(),
    };
    if let Err(err) = messages::post(daemon, &store, note, None) {
        warn!(bot = %bot, "could not tell a bot about its key: {err}");
    }
    Ok(())
}

/// What the bot hears once its key is saved.
pub fn saved_note(name: &str, replaced: bool) -> String {
    let what = if replaced { "replaced" } else { "saved" };
    format!(
        "The owner {what} the key {name} you asked for. It is in your environment as {name} \
         ($env:{name} in PowerShell, ${name} in Bash): use it by that name, never by its value."
    )
}

pub fn dismiss(daemon: &Daemon, params: KeysDismissParams) -> ApiResult<()> {
    let item = daemon
        .store()
        .chat_item(&params.request_id)?
        .ok_or_else(|| {
            ApiError::NotFound(format!("request {} does not exist", params.request_id))
        })?;
    let ChatBody::KeyRequest(request) = &item.body else {
        return Err(ApiError::NotFound(format!(
            "request {} does not exist",
            params.request_id
        )));
    };
    let request = open(daemon, &item.bot_id, &item.id, &request.name)?;
    let body = format!(
        "The owner did not give the key {} now. Carry on without it, or ask again later.",
        request.name
    );
    close(
        daemon,
        &item.bot_id,
        &item.id,
        request,
        KeyRequestStatus::Dismissed,
        body,
    )
}
