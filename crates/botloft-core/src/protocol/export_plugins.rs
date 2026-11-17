//! The plugins part of the TypeScript bindings (spec 41).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<PluginSource>();
    out.decl::<Plugin>();
    out.decl::<PluginList>();
    out.decl::<PluginsShareParams>();
    out.decl::<PluginsEveryoneParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(method::PLUGINS_LIST, "undefined", &out.name::<PluginList>());
    out.method(
        method::PLUGINS_SHARE,
        &out.name::<PluginsShareParams>(),
        &out.name::<Plugin>(),
    );
    out.method(
        method::PLUGINS_EVERYONE,
        &out.name::<PluginsEveryoneParams>(),
        &out.name::<PluginList>(),
    );
}
