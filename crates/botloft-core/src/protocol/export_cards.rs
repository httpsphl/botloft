//! The cards part of the TypeScript bindings (spec 8.11).

use super::super::*;
use super::Out;

/// Before `ChatBody`, which holds `CardItem`.
pub(super) fn decls(out: &mut Out) {
    out.decl::<CardChartKind>();
    out.decl::<CardSeries>();
    out.decl::<CardButton>();
    out.decl::<CardFieldKind>();
    out.decl::<CardField>();
    out.decl::<CardBlock>();
    out.decl::<CardAnswer>();
    out.decl::<CardItem>();
    out.decl::<ChatActParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::CHAT_ACT,
        &out.name::<ChatActParams>(),
        &out.name::<ChatItem>(),
    );
}
