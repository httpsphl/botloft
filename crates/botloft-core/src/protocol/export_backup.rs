//! The backup part of the TypeScript bindings (spec 14.2).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<BackupCrew>();
    out.decl::<BackupManifest>();
    out.decl::<BackupExportParams>();
    out.decl::<BackupExported>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::BACKUP_EXPORT,
        &out.name::<BackupExportParams>(),
        &out.name::<BackupExported>(),
    );
}
