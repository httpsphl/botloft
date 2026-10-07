//! The cloud part of the TypeScript bindings (spec 27.5).

use std::fmt::Write as _;

use super::super::*;
use super::Out;

pub(super) fn decls(out: &mut Out) {
    out.decl::<CloudStatus>();
    out.decl::<CloudSigninParams>();
    out.decl::<CloudSigninStarted>();
    out.decl::<CloudUploadParams>();
    out.decl::<CloudCopy>();
    out.decl::<CloudCopies>();
    out.decl::<CloudCopyParams>();
    out.decl::<CloudDownloaded>();
    out.decl::<CloudSignedIn>();
    out.decl::<CloudDirection>();
    out.decl::<CloudProgress>();
}

pub(super) fn methods(out: &mut Out) {
    out.method(
        method::CLOUD_STATUS,
        "undefined",
        &out.name::<CloudStatus>(),
    );
    out.method(
        method::CLOUD_SIGNIN,
        &out.name::<CloudSigninParams>(),
        &out.name::<CloudSigninStarted>(),
    );
    out.method(method::CLOUD_SIGNIN_CANCEL, "undefined", "null");
    out.method(method::CLOUD_SIGNOUT, "undefined", "null");
    out.method(
        method::CLOUD_UPLOAD,
        &out.name::<CloudUploadParams>(),
        &out.name::<CloudCopy>(),
    );
    out.method(
        method::CLOUD_COPIES,
        "undefined",
        &out.name::<CloudCopies>(),
    );
    out.method(
        method::CLOUD_DOWNLOAD,
        &out.name::<CloudCopyParams>(),
        &out.name::<CloudDownloaded>(),
    );
    out.method(method::CLOUD_DELETE, &out.name::<CloudCopyParams>(), "null");
    out.method(method::CLOUD_DELETE_ACCOUNT, "undefined", "null");
}

pub(super) fn notifications(out: &mut Out) {
    let signed_in = out.name::<CloudSignedIn>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {signed_in};",
        notification::CLOUD_SIGNED_IN
    );
    let progress = out.name::<CloudProgress>();
    let _ = writeln!(
        out.text,
        "  \"{}\": {progress};",
        notification::CLOUD_PROGRESS
    );
    let _ = writeln!(
        out.text,
        "  \"{}\": null;",
        notification::CLOUD_SIGNIN_EXPIRED
    );
}
