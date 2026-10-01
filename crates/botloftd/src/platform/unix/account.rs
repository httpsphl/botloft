//! The owner's account as the system knows it (`getpwuid_r`): the name the
//! app shows and the login shell the bots' environment comes from.

use std::ffi::{CStr, c_char};
use std::path::PathBuf;

/// Longest buffer tried for one account entry.
const BUFFER_MAX: usize = 1 << 20;

#[derive(Debug, Default)]
pub(super) struct Account {
    pub login: String,
    /// The first field of the comment ("Ana Lima,,,"), when there is one.
    pub full_name: String,
    pub shell: Option<PathBuf>,
}

pub(super) fn current() -> Option<Account> {
    // SAFETY: no arguments, never fails.
    let uid = unsafe { libc::getuid() };
    let mut buf = vec![0u8; 4096];
    // SAFETY: plain data; `getpwuid_r` fills it in.
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut found: *mut libc::passwd = std::ptr::null_mut();
    loop {
        // SAFETY: `entry`, `buf` and `found` outlive the call, and `buf.len()`
        // is the size of `buf`.
        let code = unsafe {
            libc::getpwuid_r(
                uid,
                &raw mut entry,
                buf.as_mut_ptr().cast(),
                buf.len(),
                &raw mut found,
            )
        };
        if code == libc::ERANGE && buf.len() < BUFFER_MAX {
            buf.resize(buf.len() * 2, 0);
            continue;
        }
        if code != 0 || found.is_null() {
            return None;
        }
        break;
    }
    let full_name = text(entry.pw_gecos)
        .split(',')
        .next()
        .unwrap_or_default()
        .trim()
        .to_owned();
    let shell = Some(text(entry.pw_shell))
        .filter(|shell| !shell.is_empty())
        .map(PathBuf::from);
    Some(Account {
        login: text(entry.pw_name),
        full_name,
        shell,
    })
}

/// A string of the entry, which points into the buffer still alive.
fn text(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: a non-null field of a filled entry is a NUL-terminated string.
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

/// The account's full name ("Ana Lima"), else the login name. `USER` and
/// the home folder cover a uid the system has no entry for (containers).
pub fn owner_name() -> String {
    let account = current().unwrap_or_default();
    [account.full_name, account.login]
        .into_iter()
        .chain(
            ["USER", "LOGNAME"]
                .iter()
                .filter_map(std::env::var_os)
                .map(|name| name.to_string_lossy().into_owned()),
        )
        .chain(dirs::home_dir().and_then(|home| {
            home.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        }))
        .map(|name| name.trim().to_owned())
        .find(|name| !name.is_empty())
        .unwrap_or_default()
}
