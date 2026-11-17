//! The claude.ai connectors part of the TypeScript bindings (spec 34).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<ConnectorState>();
    out.decl::<Connector>();
    out.decl::<ConnectorList>();
    out.decl::<ConnectorsListParams>();
    out.decl::<BotsSetConnectorsParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::CONNECTORS_LIST,
        &out.name::<ConnectorsListParams>(),
        &out.name::<ConnectorList>(),
    );
    out.method(
        method::BOTS_SET_CONNECTORS,
        &out.name::<BotsSetConnectorsParams>(),
        &out.name::<Bot>(),
    );
}
