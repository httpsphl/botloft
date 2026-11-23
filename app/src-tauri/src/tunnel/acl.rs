//! On Windows, the tunnel's folder and key file open for this user alone, as
//! OpenSSH insists: a folder may pass rights on to others (another
//! program's group, say), and `ssh` then ignores the key.

use std::io;
use std::os::windows::ffi::OsStrExt as _;
use std::path::Path;

use windows::Win32::Foundation::{
    CloseHandle, ERROR_SUCCESS, GENERIC_ALL, HANDLE, HLOCAL, LocalFree, WIN32_ERROR,
};
use windows::Win32::Security::Authorization::{
    EXPLICIT_ACCESS_W, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, SET_ACCESS, SetEntriesInAclW,
    SetNamedSecurityInfoW, TRUSTEE_IS_SID, TRUSTEE_IS_USER, TRUSTEE_W,
};
use windows::Win32::Security::{
    ACL, DACL_SECURITY_INFORMATION, GetTokenInformation, NO_INHERITANCE,
    PROTECTED_DACL_SECURITY_INFORMATION, SUB_CONTAINERS_AND_OBJECTS_INHERIT, TOKEN_QUERY,
    TOKEN_USER, TokenUser,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{PCWSTR, PWSTR};

/// Gives `path` one ACE, full control for this user, and nothing inherited.
/// A folder passes that one ACE on to what is made in it.
pub fn only_this_user(path: &Path) -> io::Result<()> {
    let user = token_user()?;
    // SAFETY: `user` was filled by GetTokenInformation(TokenUser).
    let sid = unsafe { (*user.as_ptr().cast::<TOKEN_USER>()).User.Sid };
    let access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: GENERIC_ALL.0,
        grfAccessMode: SET_ACCESS,
        grfInheritance: if path.is_dir() {
            SUB_CONTAINERS_AND_OBJECTS_INHERIT
        } else {
            NO_INHERITANCE
        },
        Trustee: TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_USER,
            ptstrName: PWSTR(sid.0.cast()),
        },
    };
    let mut acl: *mut ACL = std::ptr::null_mut();
    // SAFETY: `access` points at the SID in `user`, alive for the call.
    check(unsafe { SetEntriesInAclW(Some(&[access]), None, &mut acl) })?;
    let acl = Local(acl.cast());
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: `wide` ends with NUL and `acl` lives until the call returns.
    check(unsafe {
        SetNamedSecurityInfoW(
            PCWSTR(wide.as_ptr()),
            SE_FILE_OBJECT,
            DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
            None,
            None,
            Some(acl.0.cast::<ACL>().cast_const()),
            None,
        )
    })
}

/// This process's `TOKEN_USER`, in a buffer aligned for it.
fn token_user() -> io::Result<Vec<u64>> {
    let mut token = HANDLE::default();
    // SAFETY: the current process's pseudo handle is always valid.
    unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }?;
    let mut len = 0u32;
    // SAFETY: a size query without a buffer, which fails and sets `len`.
    let _ = unsafe { GetTokenInformation(token, TokenUser, None, 0, &mut len) };
    let mut buf = vec![0u64; (len as usize).div_ceil(8)];
    // SAFETY: `buf` holds at least `len` bytes, 8-byte aligned.
    let filled = unsafe {
        GetTokenInformation(
            token,
            TokenUser,
            Some(buf.as_mut_ptr().cast()),
            len,
            &mut len,
        )
    };
    // SAFETY: opened above, closed once.
    let _ = unsafe { CloseHandle(token) };
    filled?;
    Ok(buf)
}

fn check(status: WIN32_ERROR) -> io::Result<()> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(status.0 as i32))
    }
}

/// Memory from Win32 that goes back through `LocalFree`.
struct Local(*mut core::ffi::c_void);

impl Drop for Local {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: allocated by Win32 for LocalFree.
            unsafe { LocalFree(Some(HLOCAL(self.0))) };
        }
    }
}
