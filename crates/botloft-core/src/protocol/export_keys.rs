//! The agents' keys part of the TypeScript bindings (spec 32).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<AgentKey>();
    out.decl::<KeysListParams>();
    out.decl::<KeysSetParams>();
    out.decl::<KeysDeleteParams>();
    out.decl::<KeysDismissParams>();
    out.decl::<KeyHit>();
    out.decl::<KeyKeep>();
    out.decl::<KeyScrub>();
    out.decl::<KeysScrubParams>();
}

pub(super) fn methods(out: &mut Out) {
    let keys = out.name::<Vec<AgentKey>>();
    out.method(method::KEYS_LIST, &out.name::<KeysListParams>(), &keys);
    out.method(method::KEYS_SET, &out.name::<KeysSetParams>(), &keys);
    out.method(method::KEYS_DELETE, &out.name::<KeysDeleteParams>(), &keys);
    out.method(
        method::KEYS_DISMISS,
        &out.name::<KeysDismissParams>(),
        "null",
    );
    let hits = out.name::<Vec<KeyHit>>();
    out.method(method::KEYS_SCAN, "undefined", &hits);
    out.method(method::KEYS_SCRUB, &out.name::<KeysScrubParams>(), &hits);
}
