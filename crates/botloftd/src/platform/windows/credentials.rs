//! A secret in the Windows Credential Manager, for the current user only
//! (spec 27.10): the passphrase the automatic backup seals with.

use std::io;
use std::ptr;

use windows::Win32::Foundation::ERROR_NOT_FOUND;
use windows::Win32::Security::Credentials::{
    CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC, CREDENTIALW, CredDeleteW, CredFree, CredReadW,
    CredWriteW,
};
use windows::core::{HSTRING, PWSTR};

/// The blob of a generic credential holds at most 5 x 512 bytes.
const MAX_SECRET: usize = 2560;

pub fn secret_available() -> bool {
    true
}

pub fn secret_save(target: &str, secret: &str) -> io::Result<()> {
    if secret.len() > MAX_SECRET {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the secret is too long for the credential store",
        ));
    }
    let name = HSTRING::from(target);
    let user = HSTRING::from("Botloft");
    let mut blob = secret.as_bytes().to_vec();
    let credential = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: PWSTR(name.as_ptr().cast_mut()),
        CredentialBlobSize: u32::try_from(blob.len()).unwrap_or(0),
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        UserName: PWSTR(user.as_ptr().cast_mut()),
        ..Default::default()
    };
    // SAFETY: `credential` points at `name`, `user` and `blob`, which all
    // outlive the call; the system copies what it needs.
    let written = unsafe { CredWriteW(&credential, 0) };
    // The passphrase does not stay in this memory longer than it must.
    blob.fill(0);
    written.map_err(|err| io::Error::other(err.to_string()))
}

pub fn secret_load(target: &str) -> io::Result<Option<String>> {
    let name = HSTRING::from(target);
    let mut found: *mut CREDENTIALW = ptr::null_mut();
    // SAFETY: `found` receives a buffer the system owns until `CredFree`.
    let read = unsafe { CredReadW(&name, CRED_TYPE_GENERIC, None, &mut found) };
    match read {
        Ok(()) => {}
        Err(err) if err.code() == ERROR_NOT_FOUND.to_hresult() => return Ok(None),
        Err(err) => return Err(io::Error::other(err.to_string())),
    }
    // SAFETY: a successful read left `found` pointing at a credential whose
    // blob is `CredentialBlobSize` bytes; the copy is made before the free.
    let secret = unsafe {
        let credential = &*found;
        let bytes = if credential.CredentialBlob.is_null() {
            &[][..]
        } else {
            std::slice::from_raw_parts(
                credential.CredentialBlob,
                credential.CredentialBlobSize as usize,
            )
        };
        let secret = String::from_utf8(bytes.to_vec()).ok();
        CredFree(found.cast());
        secret
    };
    Ok(secret)
}

pub fn secret_delete(target: &str) -> io::Result<()> {
    let name = HSTRING::from(target);
    // SAFETY: plain call with a valid, NUL-ended name.
    match unsafe { CredDeleteW(&name, CRED_TYPE_GENERIC, None) } {
        Ok(()) => Ok(()),
        Err(err) if err.code() == ERROR_NOT_FOUND.to_hresult() => Ok(()),
        Err(err) => Err(io::Error::other(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_secret_is_kept_read_back_and_removed() {
        // A name of its own, so a real install's secret is never touched.
        let target = format!("Botloft test {}", std::process::id());
        assert_eq!(secret_load(&target).expect("empty"), None);
        secret_save(&target, "a long passphrase é ü").expect("save");
        assert_eq!(
            secret_load(&target).expect("load").as_deref(),
            Some("a long passphrase é ü")
        );
        secret_save(&target, "another one").expect("overwrite");
        assert_eq!(
            secret_load(&target).expect("again").as_deref(),
            Some("another one")
        );
        secret_delete(&target).expect("delete");
        assert_eq!(secret_load(&target).expect("gone"), None);
        // Deleting what is not there is fine.
        secret_delete(&target).expect("delete twice");
    }
}
