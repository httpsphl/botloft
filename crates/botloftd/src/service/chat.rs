//! `chat.history` (spec 8, 11.2).

use botloft_core::protocol::{ChatHistoryParams, ChatItem};

use super::{ApiError, ApiResult, bots};
use crate::state::Daemon;

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;

/// A page of a bot's chat, newest first. Archived bots keep theirs.
pub fn history(daemon: &Daemon, params: ChatHistoryParams) -> ApiResult<Vec<ChatItem>> {
    let limit = params.limit.unwrap_or(DEFAULT_LIMIT);
    if !(1..=MAX_LIMIT).contains(&limit) {
        return Err(ApiError::validation(format!(
            "limit must be between 1 and {MAX_LIMIT}"
        )));
    }
    let store = daemon.store();
    bots::find(&store, &params.bot_id)?;
    Ok(store.chat_history(&params.bot_id, params.before.as_ref(), limit)?)
}
