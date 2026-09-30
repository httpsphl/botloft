//! The effort and context part of the TypeScript bindings (spec 7.4, 8.6).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

/// Before `Bot`, which holds them.
pub(super) fn decls(out: &mut Out) {
    out.decl::<BotEffort>();
    out.decl::<ModelEffort>();
    out.decl::<ContextUsage>();
    out.decl::<BotContextChanged>();
    out.decl::<BotsSetEffortParams>();
}

pub(super) fn methods(out: &mut Out) {
    let bot = out.name::<Bot>();
    out.method(
        method::BOTS_SET_EFFORT,
        &out.name::<BotsSetEffortParams>(),
        &bot,
    );
    out.method(method::BOTS_COMPACT, &out.name::<BotIdParams>(), &bot);
}

pub(super) fn notifications(out: &mut Out) {
    let context = out.name::<BotContextChanged>();
    let _ = writeln!(out.text, "  \"{}\": {context};", notification::BOT_CONTEXT);
}
