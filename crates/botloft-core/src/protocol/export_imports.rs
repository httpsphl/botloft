//! Bringing the owner's history to the Maker, in the TypeScript bindings
//! (spec 36.5, 36.7).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<ImportSource>();
    out.decl::<ImportsScanParams>();
    out.decl::<ImportProject>();
    out.decl::<ImportConversations>();
    out.decl::<ImportScan>();
    out.decl::<ImportsCreateParams>();
    out.decl::<ImportDraft>();
    out.decl::<ImportIdParams>();
    out.decl::<ImportSent>();
    out.decl::<Import>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::IMPORTS_SCAN,
        &out.name::<ImportsScanParams>(),
        &out.name::<ImportScan>(),
    );
    out.method(
        method::IMPORTS_CREATE,
        &out.name::<ImportsCreateParams>(),
        &out.name::<ImportDraft>(),
    );
    out.method(
        method::IMPORTS_SEND,
        &out.name::<ImportIdParams>(),
        &out.name::<ImportSent>(),
    );
    out.method(
        method::IMPORTS_FORGET,
        &out.name::<ImportIdParams>(),
        "null",
    );
    out.method(
        method::IMPORTS_LIST,
        "undefined",
        &out.name::<Vec<Import>>(),
    );
}
