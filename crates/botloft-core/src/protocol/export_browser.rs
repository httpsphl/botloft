//! The browser part of the TypeScript bindings (spec 21.7).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<BrowserStatus>();
    out.decl::<BrowserState>();
    out.decl::<BrowserFrame>();
    out.decl::<BrowserActionKind>();
    out.decl::<BrowserAction>();
    out.decl::<BrowserView>();
    out.decl::<BrowserWatchParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::BROWSER_LIST,
        "undefined",
        &out.name::<Vec<BrowserState>>(),
    );
    out.method(
        method::BROWSER_WATCH,
        &out.name::<BrowserWatchParams>(),
        &out.name::<BrowserView>(),
    );
    out.method(method::BROWSER_UNWATCH, "undefined", "null");
}

pub(super) fn notifications(out: &mut Out) {
    let state = out.name::<BrowserState>();
    let action = out.name::<BrowserAction>();
    let frame = out.name::<BrowserFrame>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {state};",
        notification::BROWSER_CHANGED
    );
    let _ = writeln!(
        out.text,
        "  \"{}\": {action};",
        notification::BROWSER_ACTION
    );
    let _ = writeln!(out.text, "  \"{}\": {frame};", notification::BROWSER_FRAME);
}
