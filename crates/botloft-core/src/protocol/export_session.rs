//! The effort and context part of the TypeScript bindings (spec 7.4, 8.6).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

/// Before `Bot`, which holds them.
pub(super) fn decls(out: &mut Out) {
    out.decl::<BotEffort>();
    out.decl::<ModelEffort>();
    out.decl::<ContextPartKind>();
    out.decl::<ContextPart>();
    out.decl::<ContextFile>();
    out.decl::<ContextUsage>();
    out.decl::<BotContextChanged>();
    out.decl::<BackgroundKind>();
    out.decl::<BackgroundTask>();
    out.decl::<BotBackgroundChanged>();
    out.decl::<BotsStopTaskParams>();
    out.decl::<BotsSetEffortParams>();
}

pub(super) fn methods(out: &mut Out) {
    let bot = out.name::<Bot>();
    out.method(
        method::BOTS_SET_EFFORT,
        &out.name::<BotsSetEffortParams>(),
        &bot,
    );
    let seat = out.name::<BotConversationParams>();
    out.method(method::BOTS_COMPACT, &seat, &bot);
    out.method(method::BOTS_INTERRUPT, &seat, &bot);
    out.method(
        method::BOTS_STOP_TASK,
        &out.name::<BotsStopTaskParams>(),
        "null",
    );
}

pub(super) fn notifications(out: &mut Out) {
    let context = out.name::<BotContextChanged>();
    let _ = writeln!(out.text, "  \"{}\": {context};", notification::BOT_CONTEXT);
    let background = out.name::<BotBackgroundChanged>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {background};",
        notification::BOT_BACKGROUND
    );
}
