//! The island part of the TypeScript bindings (spec 31).

use super::super::*;
use super::Out;

/// After the mobile cards, which `Cards` holds.
pub(super) fn decls(out: &mut Out) {
    out.decl::<Cards>();
    out.decl::<ActivityTodayParams>();
    out.decl::<BusyBot>();
    out.decl::<DayActivity>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(method::CARDS_LIST, "undefined", &out.name::<Cards>());
    out.method(
        method::ACTIVITY_TODAY,
        &out.name::<ActivityTodayParams>(),
        &out.name::<DayActivity>(),
    );
}
