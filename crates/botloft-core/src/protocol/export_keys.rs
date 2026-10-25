//! The agents' keys part of the TypeScript bindings (spec 32).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<AgentKey>();
    out.decl::<KeysListParams>();
    out.decl::<KeysSetParams>();
    out.decl::<KeysDeleteParams>();
}

pub(super) fn methods(out: &mut Out) {
    let keys = out.name::<Vec<AgentKey>>();
    out.method(method::KEYS_LIST, &out.name::<KeysListParams>(), &keys);
    out.method(method::KEYS_SET, &out.name::<KeysSetParams>(), &keys);
    out.method(method::KEYS_DELETE, &out.name::<KeysDeleteParams>(), &keys);
}
