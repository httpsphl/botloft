//! The devices part of the TypeScript bindings (spec 42.6).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<Device>();
    out.decl::<DeviceList>();
    out.decl::<DevicesPairParams>();
    out.decl::<DevicePaired>();
    out.decl::<DeviceIdParams>();
}

pub(super) fn methods(out: &mut Out) {
    let list = out.name::<DeviceList>();
    out.method(method::DEVICES_LIST, "undefined", &list);
    out.method(
        method::DEVICES_PAIR,
        &out.name::<DevicesPairParams>(),
        &out.name::<DevicePaired>(),
    );
    out.method(method::DEVICES_REVOKE, &out.name::<DeviceIdParams>(), &list);
}

pub(super) fn notifications(out: &mut Out) {
    let list = out.name::<DeviceList>();
    let _ = writeln!(out.text, "  \"{}\": {list};", notification::DEVICES_CHANGED);
}
