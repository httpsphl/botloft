//! The connected-tools part of the TypeScript bindings (spec 25).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<McpKind>();
    out.decl::<McpServer>();
    out.decl::<McpState>();
    out.decl::<McpServerState>();
    out.decl::<BotMcp>();
    out.decl::<McpOverview>();
    out.decl::<McpSaveParams>();
    out.decl::<McpServerIdParams>();
    out.decl::<BotMcpSetParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(method::MCP_SERVERS, "undefined", &out.name::<McpOverview>());
    out.method(
        method::MCP_SAVE,
        &out.name::<McpSaveParams>(),
        &out.name::<McpOverview>(),
    );
    out.method(
        method::MCP_DELETE,
        &out.name::<McpServerIdParams>(),
        &out.name::<McpOverview>(),
    );
    out.method(
        method::BOT_MCP_SET,
        &out.name::<BotMcpSetParams>(),
        &out.name::<BotMcp>(),
    );
}

pub(super) fn notifications(out: &mut Out) {
    let overview = out.name::<McpOverview>();
    let _ = writeln!(out.text, "  \"{}\": {overview};", notification::MCP_SERVERS);
    let bot = out.name::<BotMcp>();
    let _ = writeln!(out.text, "  \"{}\": {bot};", notification::BOT_MCP);
}
