//! The bot catalog part of the TypeScript bindings (spec 26).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<BotTemplateCategory>();
    out.decl::<BotTemplate>();
    out.decl::<BotTemplateFull>();
    out.decl::<CatalogListParams>();
    out.decl::<CatalogGetParams>();
    out.decl::<CatalogAddParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::CATALOG_LIST,
        &out.name::<CatalogListParams>(),
        &out.name::<Vec<BotTemplate>>(),
    );
    out.method(
        method::CATALOG_GET,
        &out.name::<CatalogGetParams>(),
        &out.name::<BotTemplateFull>(),
    );
    out.method(
        method::CATALOG_ADD,
        &out.name::<CatalogAddParams>(),
        &out.name::<Bot>(),
    );
    // What the Maker built from the catalog, and undoing it (spec 36.4).
    out.method(method::BUILDS_LIST, "undefined", &out.name::<Vec<Build>>());
    out.method(
        method::BUILDS_UNDO,
        &out.name::<BuildIdParams>(),
        &out.name::<Build>(),
    );
}
