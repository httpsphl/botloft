//! Windows ACLs, Job Objects, the user environment, power requests and
//! console signals.

mod console;
mod env;
mod job;
mod owner;
mod power;
mod recycle;
mod sign_in;
mod task;
mod task_xml;

pub use console::leave_own_console;
pub use env::user_environment;
pub use job::ProcessJob;
pub use owner::owner_name;
pub use power::KeepAwake;
pub use recycle::recycle;
pub use sign_in::sign_in_id;
pub use task::{delete_task, find_task, register_task, run_task, stop_task};

/// What starts the daemon for the owner here, for messages (spec 14).
pub const TASK_KIND: &str = "scheduled task";

/// Nothing to track: every Job Object dies with the daemon (spec 7.3).
pub fn track_groups(_dir: &Path) {}

use std::io;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;

use windows::Win32::Foundation::{
    CloseHandle, ERROR_SUCCESS, GENERIC_ALL, HANDLE, HLOCAL, LocalFree, WIN32_ERROR,
};
use windows::Win32::Security::Authorization::{
    ConvertSidToStringSidW, EXPLICIT_ACCESS_W, NO_MULTIPLE_TRUSTEE, SE_FILE_OBJECT, SET_ACCESS,
    SetEntriesInAclW, SetNamedSecurityInfoW, TRUSTEE_IS_SID, TRUSTEE_IS_USER, TRUSTEE_W,
};
use windows::Win32::Security::{
    ACL, DACL_SECURITY_INFORMATION, GetTokenInformation, NO_INHERITANCE,
    PROTECTED_DACL_SECURITY_INFORMATION, PSID, SUB_CONTAINERS_AND_OBJECTS_INHERIT, TOKEN_QUERY,
    TOKEN_USER, TokenUser,
};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};
use windows::core::{PCWSTR, PWSTR};

/// Replaces the DACL of `path` with a single ACE granting full control to
/// the current user, and turns inheritance off (spec 13). Folders pass the
/// ACE on to what is created inside them.
pub fn restrict_to_current_user(path: &Path) -> io::Result<()> {
    let user = CurrentUser::query()?;
    let inheritance = if path.is_dir() {
        SUB_CONTAINERS_AND_OBJECTS_INHERIT
    } else {
        NO_INHERITANCE
    };
    let access = EXPLICIT_ACCESS_W {
        grfAccessPermissions: GENERIC_ALL.0,
        grfAccessMode: SET_ACCESS,
        grfInheritance: inheritance,
        Trustee: TRUSTEE_W {
            pMultipleTrustee: std::ptr::null_mut(),
            MultipleTrusteeOperation: NO_MULTIPLE_TRUSTEE,
            TrusteeForm: TRUSTEE_IS_SID,
            TrusteeType: TRUSTEE_IS_USER,
            ptstrName: PWSTR(user.sid().0.cast()),
        },
    };

    let mut acl: *mut ACL = std::ptr::null_mut();
    // SAFETY: `access` points at a SID owned by `user`, alive for the call.
    check(unsafe { SetEntriesInAclW(Some(&[access]), None, &mut acl) })?;
    let acl = LocalBox(acl.cast());

    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: `wide` is NUL-terminated and `acl` stays alive until return.
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

fn check(status: WIN32_ERROR) -> io::Result<()> {
    if status == ERROR_SUCCESS {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(status.0 as i32))
    }
}

/// Memory returned by Win32 that must go back through `LocalFree`.
struct LocalBox(*mut core::ffi::c_void);

impl Drop for LocalBox {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: the pointer came from a Win32 allocation freed with LocalFree.
            unsafe { LocalFree(Some(HLOCAL(self.0))) };
        }
    }
}

/// The `TOKEN_USER` of the current process, which holds the user's SID.
pub(crate) struct CurrentUser {
    buf: Vec<u64>,
}

impl CurrentUser {
    pub(crate) fn query() -> io::Result<Self> {
        let mut token = HANDLE::default();
        // SAFETY: the pseudo handle of the current process is always valid.
        unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) }?;
        let token = OwnedHandle(token);

        let mut len = 0u32;
        // SAFETY: a size query with no buffer; failing with "insufficient
        // buffer" is the expected outcome and `len` receives the size.
        let _ = unsafe { GetTokenInformation(token.0, TokenUser, None, 0, &mut len) };
        let mut buf = vec![0u64; (len as usize).div_ceil(8)];
        // SAFETY: `buf` is at least `len` bytes and 8-byte aligned.
        unsafe {
            GetTokenInformation(
                token.0,
                TokenUser,
                Some(buf.as_mut_ptr().cast()),
                len,
                &mut len,
            )
        }?;
        Ok(Self { buf })
    }

    pub(crate) fn sid(&self) -> PSID {
        // SAFETY: `buf` was filled by GetTokenInformation(TokenUser).
        unsafe { (*self.buf.as_ptr().cast::<TOKEN_USER>()).User.Sid }
    }

    /// The SID as text (`S-1-5-21-...`).
    pub(crate) fn sid_string(&self) -> io::Result<String> {
        let mut text = PWSTR::null();
        // SAFETY: `sid()` is valid while `self` lives; the string is freed
        // by LocalBox.
        unsafe { ConvertSidToStringSidW(self.sid(), &mut text) }?;
        let _text = LocalBox(text.0.cast());
        // SAFETY: a NUL-terminated string allocated by the call above.
        unsafe { text.to_string() }.map_err(io::Error::other)
    }
}

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: the handle was opened by us and is closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

pub async fn shutdown_signal() {
    use tokio::signal::windows::{ctrl_close, ctrl_logoff, ctrl_shutdown};

    async fn wait<T>(signal: io::Result<T>, recv: impl AsyncFnOnce(T)) {
        match signal {
            Ok(sig) => recv(sig).await,
            Err(_) => std::future::pending::<()>().await,
        }
    }

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        () = wait(ctrl_close(), async |mut s| { s.recv().await; }) => {}
        () = wait(ctrl_logoff(), async |mut s| { s.recv().await; }) => {}
        () = wait(ctrl_shutdown(), async |mut s| { s.recv().await; }) => {}
    }
}

#[cfg(test)]
mod tests;
