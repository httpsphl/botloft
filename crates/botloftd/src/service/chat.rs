//! `chat.history` and `chat.search` (spec 8, 8.8, 11.2).

use botloft_core::protocol::{ChatHistoryParams, ChatItem, ChatSearchParams, SearchHit};
use botloft_store::{SearchFilter, fts_query};

use super::{ApiError, ApiResult, bots, crews};
use crate::state::Daemon;

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;
/// Most items `until` brings back, newest first.
const UNTIL_MAX: u32 = 1_000;
const SEARCH_DEFAULT_LIMIT: u32 = 30;
const SEARCH_MAX_LIMIT: u32 = 100;
/// Fewest letters worth searching for.
const SEARCH_MIN_CHARS: usize = 2;

/// A page of a bot's chat, newest first. Archived bots keep theirs.
pub fn history(daemon: &Daemon, params: ChatHistoryParams) -> ApiResult<Vec<ChatItem>> {
    if let Some(until) = &params.until {
        if params.before.is_some() || params.limit.is_some() {
            return Err(ApiError::validation(
                "until does not go with before or limit",
            ));
        }
        let store = daemon.store();
        bots::find(&store, &params.bot_id)?;
        let items = store.chat_since(&params.bot_id, until, UNTIL_MAX)?;
        if items.is_empty() {
            return Err(ApiError::NotFound(format!(
                "chat item {until} is not in this bot's chat"
            )));
        }
        return Ok(items);
    }
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

/// Chat items of active bots with every word of the query, newest first.
pub fn search(daemon: &Daemon, params: ChatSearchParams) -> ApiResult<Vec<SearchHit>> {
    let limit = params.limit.unwrap_or(SEARCH_DEFAULT_LIMIT);
    if !(1..=SEARCH_MAX_LIMIT).contains(&limit) {
        return Err(ApiError::validation(format!(
            "limit must be between 1 and {SEARCH_MAX_LIMIT}"
        )));
    }
    let letters = params.query.chars().filter(|c| c.is_alphanumeric()).count();
    let query = fts_query(&params.query)
        .filter(|_| letters >= SEARCH_MIN_CHARS)
        .ok_or_else(|| {
            ApiError::validation(format!(
                "query needs at least {SEARCH_MIN_CHARS} letters or digits"
            ))
        })?;
    let store = daemon.store();
    if let Some(bot) = &params.bot_id {
        bots::find(&store, bot)?;
    }
    if let Some(crew) = &params.crew_id {
        crews::find(&store, crew)?;
    }
    let filter = SearchFilter {
        bot: params.bot_id.as_ref(),
        crew: params.crew_id.as_ref(),
        before: params.before.as_ref(),
        limit,
    };
    Ok(store
        .search_chat(&query, filter)?
        .into_iter()
        .map(|found| SearchHit {
            item: found.item,
            crew_id: found.crew,
            snippet: found.snippet,
        })
        .collect())
}
