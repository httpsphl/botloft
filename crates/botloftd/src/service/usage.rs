//! `usage.tokens` (spec 8.7, 11.2).

use botloft_core::protocol::{BotTokens, UsageTokensParams};

use super::{ApiError, ApiResult};
use crate::state::Daemon;

/// Each bot's tokens since `since`, archived bots included.
pub fn tokens(daemon: &Daemon, params: UsageTokensParams) -> ApiResult<Vec<BotTokens>> {
    if params.since < 0 {
        return Err(ApiError::validation("since must not be negative"));
    }
    let store = daemon.store();
    let mut bots = store.turn_tokens(params.since)?;
    if let Some(shares) = super::plan::shares(&store, params.since, daemon.clock.now_ms()) {
        for bot in &mut bots {
            bot.plan_share = Some(shares.get(&bot.bot_id).copied().unwrap_or_default());
        }
    }
    Ok(bots)
}
