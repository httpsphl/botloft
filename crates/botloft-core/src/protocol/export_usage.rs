//! The token part of the TypeScript bindings (spec 8.7).

use super::super::*;
use super::Out;

/// Before `TurnItem`, which holds `TokenUsage`.
pub(super) fn decls(out: &mut Out) {
    out.decl::<TokenUsage>();
    out.decl::<UsageTokensParams>();
    out.decl::<BotTokens>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::USAGE_TOKENS,
        &out.name::<UsageTokensParams>(),
        &out.name::<Vec<BotTokens>>(),
    );
}
