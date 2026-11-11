//! The history part of the TypeScript bindings (spec 8.10).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<VersionKind>();
    out.decl::<VersionAuthor>();
    out.decl::<TextVersion>();
    out.decl::<VersionsListParams>();
    out.decl::<VersionsRestoreParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::VERSIONS_LIST,
        &out.name::<VersionsListParams>(),
        &out.name::<Vec<TextVersion>>(),
    );
    out.method(
        method::VERSIONS_RESTORE,
        &out.name::<VersionsRestoreParams>(),
        &out.name::<TextVersion>(),
    );
}

pub(super) fn notifications(out: &mut Out) {
    let version = out.name::<TextVersion>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {version};",
        notification::VERSION_CREATED
    );
}
