//! The "Allow always" part of the TypeScript bindings (spec 10.1).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

/// Before `ApprovalItem`, which holds `AllowScope`.
pub(super) fn decls(out: &mut Out) {
    out.decl::<AllowKind>();
    out.decl::<AllowScope>();
    out.decl::<AllowRule>();
    out.decl::<RulesListParams>();
    out.decl::<RuleIdParams>();
    out.decl::<BotRules>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::RULES_LIST,
        &out.name::<RulesListParams>(),
        &out.name::<Vec<AllowRule>>(),
    );
    out.method(
        method::RULES_DELETE,
        &out.name::<RuleIdParams>(),
        &out.name::<BotRules>(),
    );
}

pub(super) fn notifications(out: &mut Out) {
    let rules = out.name::<BotRules>();
    let _ = writeln!(out.text, "  \"{}\": {rules};", notification::BOT_RULES);
}
