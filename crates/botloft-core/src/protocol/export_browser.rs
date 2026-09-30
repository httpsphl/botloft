//! The browser and screens part of the TypeScript bindings (spec 21.7, 21.10, 22.4).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<BrowserStatus>();
    out.decl::<BrowserControl>();
    out.decl::<BrowserTab>();
    out.decl::<BrowserState>();
    out.decl::<BrowserFrame>();
    out.decl::<BrowserActionKind>();
    out.decl::<BrowserAction>();
    out.decl::<BrowserView>();
    out.decl::<BrowserWatchParams>();
    out.decl::<BrowserResizeParams>();
    out.decl::<MouseAction>();
    out.decl::<MouseButton>();
    out.decl::<BrowserInput>();
    out.decl::<BrowserControlParams>();
    out.decl::<BrowserInputParams>();
    out.decl::<BrowserTabParams>();
    out.decl::<BrowserOpenParams>();
    out.decl::<ScreenDevice>();
    out.decl::<Screen>();
    out.decl::<ScreensListParams>();
    out.decl::<ScreenDraft>();
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
    out.method(
        method::BROWSER_RESIZE,
        &out.name::<BrowserResizeParams>(),
        "null",
    );
    let control = out.name::<BrowserControlParams>();
    let state = out.name::<BrowserState>();
    out.method(method::BROWSER_TAKE, &control, &state);
    out.method(method::BROWSER_RELEASE, &control, &state);
    out.method(
        method::BROWSER_INPUT,
        &out.name::<BrowserInputParams>(),
        "null",
    );
    out.method(method::BROWSER_RELOAD, &control, "null");
    out.method(method::BROWSER_NEW_TAB, &control, "null");
    out.method(
        method::BROWSER_SWITCH_TAB,
        &out.name::<BrowserTabParams>(),
        "null",
    );
    out.method(
        method::BROWSER_OPEN,
        &out.name::<BrowserOpenParams>(),
        "null",
    );
    out.method(
        method::SCREENS_LIST,
        &out.name::<ScreensListParams>(),
        &out.name::<Vec<Screen>>(),
    );
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
    let draft = out.name::<ScreenDraft>();
    let _ = writeln!(out.text, "  \"{}\": {draft};", notification::SCREEN_DRAFT);
}
