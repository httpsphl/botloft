//! The owner's Chrome part of the TypeScript bindings (spec 37).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<ChromeGrant>();
    out.decl::<ChromeSetParams>();
    out.decl::<BotChrome>();
    out.decl::<ChromeTurn>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::CHROME_SET,
        &out.name::<ChromeSetParams>(),
        &out.name::<BotChrome>(),
    );
    out.method(
        method::CHROME_STATUS,
        "undefined",
        &out.name::<ChromeTurn>(),
    );
    out.method(method::CHROME_STOP, "undefined", &out.name::<ChromeTurn>());
}

pub(super) fn notifications(out: &mut Out) {
    let chrome = out.name::<BotChrome>();
    let _ = writeln!(out.text, "  \"{}\": {chrome};", notification::BOT_CHROME);
    let turn = out.name::<ChromeTurn>();
    let _ = writeln!(out.text, "  \"{}\": {turn};", notification::CHROME_TURN);
}
