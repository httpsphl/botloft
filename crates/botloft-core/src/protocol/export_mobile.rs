//! The phone part of the TypeScript bindings (spec 28).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<MobileRelay>();
    out.decl::<MobilePhone>();
    out.decl::<MobileJoined>();
    out.decl::<MobilePairing>();
    out.decl::<MobileStatus>();
    out.decl::<MobilePairStarted>();
    out.decl::<MobilePairIdParams>();
    out.decl::<MobilePairConfirmParams>();
    out.decl::<MobileRevokeParams>();
    out.decl::<MobilePairRequest>();
    out.decl::<CardBot>();
    out.decl::<ApprovalCard>();
    out.decl::<QuestionCard>();
    out.decl::<ChatLine>();
    out.decl::<PhoneItem>();
    out.decl::<ToPhone>();
    out.decl::<FromPhone>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::MOBILE_STATUS,
        "undefined",
        &out.name::<MobileStatus>(),
    );
    out.method(
        method::MOBILE_PAIR_START,
        "undefined",
        &out.name::<MobilePairStarted>(),
    );
    out.method(
        method::MOBILE_PAIR_CANCEL,
        &out.name::<MobilePairIdParams>(),
        "null",
    );
    out.method(
        method::MOBILE_PAIR_CONFIRM,
        &out.name::<MobilePairConfirmParams>(),
        "null",
    );
    out.method(
        method::MOBILE_REVOKE,
        &out.name::<MobileRevokeParams>(),
        &out.name::<MobileStatus>(),
    );
}

pub(super) fn notifications(out: &mut Out) {
    let changed = out.name::<MobileStatus>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {changed};",
        notification::MOBILE_CHANGED
    );
    let request = out.name::<MobilePairRequest>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {request};",
        notification::MOBILE_PAIR_REQUEST
    );
}
