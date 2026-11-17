//! The shared site logins part of the TypeScript bindings (spec 35).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<SiteLogin>();
    out.decl::<LoginList>();
    out.decl::<LoginsChanged>();
    out.decl::<LoginsOpenParams>();
    out.decl::<LoginsOpenResult>();
    out.decl::<LoginsShareParams>();
    out.decl::<LoginsSiteParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(method::LOGINS_LIST, "undefined", &out.name::<LoginList>());
    out.method(
        method::LOGINS_OPEN,
        &out.name::<LoginsOpenParams>(),
        &out.name::<LoginsOpenResult>(),
    );
    out.method(
        method::LOGINS_SHARE,
        &out.name::<LoginsShareParams>(),
        &out.name::<SiteLogin>(),
    );
    out.method(
        method::LOGINS_REMOVE,
        &out.name::<LoginsSiteParams>(),
        &out.name::<LoginList>(),
    );
}

pub(super) fn notifications(out: &mut Out) {
    let changed = out.name::<LoginsChanged>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {changed};",
        notification::LOGINS_CHANGED
    );
}
