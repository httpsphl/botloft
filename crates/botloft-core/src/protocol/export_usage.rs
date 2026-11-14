//! The usage part of the TypeScript bindings (spec 8.7).

use super::super::*;
use super::Out;

/// Before `TurnItem`, which holds `TokenUsage`.
pub(super) fn decls(out: &mut Out) {
    out.decl::<TokenUsage>();
    out.decl::<UsageTokensParams>();
    out.decl::<BotTokens>();
    out.decl::<CostBy>();
    out.decl::<UsageCostsParams>();
    out.decl::<CostGroup>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::USAGE_TOKENS,
        &out.name::<UsageTokensParams>(),
        &out.name::<Vec<BotTokens>>(),
    );
    out.method(
        method::USAGE_COSTS,
        &out.name::<UsageCostsParams>(),
        &out.name::<Vec<CostGroup>>(),
    );
}
