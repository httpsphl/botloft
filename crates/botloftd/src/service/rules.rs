//! What each bot may do without asking (spec 10.1): `rules.list` and
//! `rules.delete`. Rules are added by answering a request with "Allow
//! always" (`approvals.answer`).

use botloft_core::protocol::{AllowRule, BotRules, RuleIdParams, RulesListParams};

use super::{ApiError, ApiResult};
use crate::state::{Daemon, Event};

pub fn list(daemon: &Daemon, params: RulesListParams) -> ApiResult<Vec<AllowRule>> {
    let store = daemon.store();
    if store.bot(&params.bot_id)?.is_none() {
        return Err(ApiError::NotFound(format!("bot {}", params.bot_id)));
    }
    Ok(store.allow_rules(&params.bot_id)?)
}

/// Removes a rule: the bot asks for that again.
pub fn delete(daemon: &Daemon, params: RuleIdParams) -> ApiResult<BotRules> {
    let store = daemon.store();
    let bot_id = store
        .delete_allow_rule(&params.rule_id)?
        .ok_or_else(|| ApiError::NotFound(format!("rule {}", params.rule_id)))?;
    let rules = BotRules {
        rules: store.allow_rules(&bot_id)?,
        bot_id,
    };
    drop(store);
    daemon.emit(Event::BotRules(rules.clone()));
    Ok(rules)
}
