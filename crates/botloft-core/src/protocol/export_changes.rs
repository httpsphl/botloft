//! The Overview's changes part of the TypeScript bindings (spec 33.5).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<GitFileStatus>();
    out.decl::<GitFile>();
    out.decl::<GitRepo>();
    out.decl::<GitEventKind>();
    out.decl::<GitEvent>();
    out.decl::<OverviewChanges>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::OVERVIEW_CHANGES,
        &out.name::<BotIdParams>(),
        &out.name::<OverviewChanges>(),
    );
}
