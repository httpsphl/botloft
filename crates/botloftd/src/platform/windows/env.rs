//! The environment a fresh logon of the current user would get.

use std::ffi::{OsString, c_void};
use std::io;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Foundation::HANDLE;
use windows::Win32::Security::{TOKEN_DUPLICATE, TOKEN_IMPERSONATE, TOKEN_QUERY};
use windows::Win32::System::Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

use super::OwnedHandle;

/// System and user variables from the registry, as `CreateEnvironmentBlock`
/// builds them for a new logon, without anything the daemon's own parent
/// process added.
pub fn user_environment() -> io::Result<Vec<(OsString, OsString)>> {
    let mut token = HANDLE::default();
    // SAFETY: the pseudo handle of the current process is always valid.
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY | TOKEN_DUPLICATE | TOKEN_IMPERSONATE,
            &mut token,
        )
    }?;
    let token = OwnedHandle(token);

    let mut block: *mut c_void = std::ptr::null_mut();
    // SAFETY: `block` receives a buffer freed below with DestroyEnvironmentBlock.
    unsafe { CreateEnvironmentBlock(&mut block, Some(token.0), false) }?;
    // SAFETY: the block is a sequence of NUL-terminated UTF-16 strings that
    // ends with an empty one.
    let vars = unsafe { parse_block(block.cast::<u16>()) };
    // SAFETY: `block` came from CreateEnvironmentBlock.
    let _ = unsafe { DestroyEnvironmentBlock(block) };
    Ok(vars)
}

/// # Safety
/// `block` must point at a valid environment block.
unsafe fn parse_block(block: *const u16) -> Vec<(OsString, OsString)> {
    let mut vars = Vec::new();
    let mut cursor = block;
    loop {
        let mut len = 0;
        // SAFETY: every string in the block is NUL-terminated.
        while unsafe { *cursor.add(len) } != 0 {
            len += 1;
        }
        if len == 0 {
            return vars;
        }
        // SAFETY: `len` units were just read from this string.
        let entry = unsafe { std::slice::from_raw_parts(cursor, len) };
        // Skip the leading `=` of hidden per-drive entries like `=C:=C:\`.
        if let Some(eq) = entry.iter().skip(1).position(|&c| c == u16::from(b'=')) {
            let eq = eq + 1;
            vars.push((
                OsString::from_wide(&entry[..eq]),
                OsString::from_wide(&entry[eq + 1..]),
            ));
        }
        // SAFETY: move past this string and its NUL.
        cursor = unsafe { cursor.add(len + 1) };
    }
}
