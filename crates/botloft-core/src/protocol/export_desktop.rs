//! The desktop part of the TypeScript bindings (spec 24).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<DesktopScope>();
    out.decl::<DesktopLevel>();
    out.decl::<DesktopGrant>();
    out.decl::<DesktopGrantsParams>();
    out.decl::<DesktopGrantIdParams>();
    out.decl::<DesktopOptionsParams>();
    out.decl::<BotDesktop>();
    out.decl::<DesktopWindow>();
    out.decl::<DesktopActionKind>();
    out.decl::<DesktopAction>();
    out.decl::<DesktopState>();
    out.decl::<DesktopFrame>();
    out.decl::<DesktopView>();
    out.decl::<DesktopBotParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::DESKTOP_GRANTS,
        &out.name::<DesktopGrantsParams>(),
        &out.name::<Vec<DesktopGrant>>(),
    );
    out.method(
        method::DESKTOP_REVOKE,
        &out.name::<DesktopGrantIdParams>(),
        &out.name::<BotDesktop>(),
    );
    out.method(
        method::DESKTOP_SET_OPTIONS,
        &out.name::<DesktopOptionsParams>(),
        &out.name::<BotDesktop>(),
    );
    let bot = out.name::<DesktopBotParams>();
    out.method(method::DESKTOP_WATCH, &bot, &out.name::<DesktopView>());
    out.method(method::DESKTOP_UNWATCH, "undefined", "null");
    let state = out.name::<DesktopState>();
    out.method(method::DESKTOP_STOP, &bot, &state);
    out.method(method::DESKTOP_RESUME, &bot, &state);
}

pub(super) fn notifications(out: &mut Out) {
    let desktop = out.name::<BotDesktop>();
    let _ = writeln!(out.text, "  \"{}\": {desktop};", notification::BOT_DESKTOP);
    let state = out.name::<DesktopState>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {state};",
        notification::DESKTOP_CHANGED
    );
    let frame = out.name::<DesktopFrame>();
    let _ = writeln!(out.text, "  \"{}\": {frame};", notification::DESKTOP_FRAME);
}
