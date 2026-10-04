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
    out.decl::<BotDesktop>();
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
}

pub(super) fn notifications(out: &mut Out) {
    let desktop = out.name::<BotDesktop>();
    let _ = writeln!(out.text, "  \"{}\": {desktop};", notification::BOT_DESKTOP);
}
