//! `usage.tokens` (spec 8.7, 11.2).

use botloft_core::protocol::{BotTokens, UsageTokensParams};

use super::{ApiError, ApiResult};
use crate::state::Daemon;

/// Each bot's tokens since `since`, archived bots included.
pub fn tokens(daemon: &Daemon, params: UsageTokensParams) -> ApiResult<Vec<BotTokens>> {
    if params.since < 0 {
        return Err(ApiError::validation("since must not be negative"));
    }
    Ok(daemon.store().turn_tokens(params.since)?)
}
