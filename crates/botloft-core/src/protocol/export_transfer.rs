//! Files in pieces for an app on another computer (spec 42.7), in the
//! TypeScript bindings.

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<FilesChunkParams>();
    out.decl::<BackupChunkParams>();
    out.decl::<FileChunk>();
    out.decl::<UploadsStartParams>();
    out.decl::<UploadStarted>();
    out.decl::<UploadsChunkParams>();
    out.decl::<UploadIdParams>();
    out.decl::<UploadFinished>();
}

pub(super) fn methods(out: &mut Out) {
    let chunk = out.name::<FileChunk>();
    out.method(method::FILES_CHUNK, &out.name::<FilesChunkParams>(), &chunk);
    out.method(
        method::BACKUP_CHUNK,
        &out.name::<BackupChunkParams>(),
        &chunk,
    );
    out.method(
        method::UPLOADS_START,
        &out.name::<UploadsStartParams>(),
        &out.name::<UploadStarted>(),
    );
    let id = out.name::<UploadIdParams>();
    out.method(
        method::UPLOADS_CHUNK,
        &out.name::<UploadsChunkParams>(),
        "null",
    );
    out.method(method::UPLOADS_FINISH, &id, &out.name::<UploadFinished>());
    out.method(method::UPLOADS_CANCEL, &id, "null");
}
