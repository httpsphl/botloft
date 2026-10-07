//! The backup part of the TypeScript bindings (spec 14.2).

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<BackupScope>();
    out.decl::<BackupCrew>();
    out.decl::<BackupManifest>();
    out.decl::<BackupExportParams>();
    out.decl::<BackupExported>();
    out.decl::<BackupStageParams>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::BACKUP_EXPORT,
        &out.name::<BackupExportParams>(),
        &out.name::<BackupExported>(),
    );
    out.method(
        method::BACKUP_STAGE,
        &out.name::<BackupStageParams>(),
        &out.name::<BackupManifest>(),
    );
    out.method(method::BACKUP_CONFIRM, "undefined", "null");
    out.method(method::BACKUP_CANCEL, "undefined", "null");
}
