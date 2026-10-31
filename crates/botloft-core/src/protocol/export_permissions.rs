//! The permissions part of the TypeScript bindings (spec 15.1, 21.5).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<BrowserSite>();
    out.decl::<BotSites>();
    out.decl::<SitesRevokeParams>();
    out.decl::<Permissions>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::PERMISSIONS_LIST,
        "undefined",
        &out.name::<Permissions>(),
    );
    out.method(
        method::SITES_REVOKE,
        &out.name::<SitesRevokeParams>(),
        &out.name::<BotSites>(),
    );
}

pub(super) fn notifications(out: &mut Out) {
    let sites = out.name::<BotSites>();
    let _ = writeln!(out.text, "  \"{}\": {sites};", notification::BOT_SITES);
}
