//! The other-crew access part of the TypeScript bindings (spec 10.4).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<CrewAccess>();
    out.decl::<CrewAccessListParams>();
    out.decl::<CrewAccessIdParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::CREW_ACCESS_LIST,
        &out.name::<CrewAccessListParams>(),
        &out.name::<Vec<CrewAccess>>(),
    );
    out.method(
        method::CREW_ACCESS_REVOKE,
        &out.name::<CrewAccessIdParams>(),
        &out.name::<Vec<CrewAccess>>(),
    );
}
