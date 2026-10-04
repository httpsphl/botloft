//! The program behind a window: its executable, its name for people, and
//! whether it runs with more rights than the daemon.

use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Threading::{
    GetCurrentProcess, OpenProcess, OpenProcessToken, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::core::{PCWSTR, PWSTR};

use super::super::App;

/// Closes a handle when it goes.
struct Owned(HANDLE);

impl Drop for Owned {
    fn drop(&mut self) {
        // SAFETY: the handle was opened by this module and is closed once.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

fn open(id: u32) -> Option<Owned> {
    // SAFETY: a plain open; the handle is closed by `Owned`.
    unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, id) }
        .ok()
        .map(Owned)
}

fn path_of(process: &Owned) -> Option<PathBuf> {
    let mut buffer = vec![0u16; 1024];
    let mut length = buffer.len() as u32;
    // SAFETY: the buffer and its length match.
    unsafe {
        QueryFullProcessImageNameW(
            process.0,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
    }
    .ok()?;
    Some(PathBuf::from(String::from_utf16_lossy(
        &buffer[..length as usize],
    )))
}

/// Whether the process runs elevated; `None` when its rights cannot be
/// read, which a process running as administrator also causes.
fn elevated(process: HANDLE) -> Option<bool> {
    let mut token = HANDLE::default();
    // SAFETY: `token` is a valid out-pointer, closed by `Owned`.
    unsafe { OpenProcessToken(process, TOKEN_QUERY, &mut token) }.ok()?;
    let token = Owned(token);
    let mut elevation = TOKEN_ELEVATION::default();
    let mut length = 0u32;
    // SAFETY: the buffer is a `TOKEN_ELEVATION` of the size given.
    unsafe {
        GetTokenInformation(
            token.0,
            TokenElevation,
            Some(std::ptr::from_mut(&mut elevation).cast::<c_void>()),
            size_of::<TOKEN_ELEVATION>() as u32,
            &mut length,
        )
    }
    .ok()?;
    Some(elevation.TokenIsElevated != 0)
}

/// The executable of process `id` and whether it runs with more rights
/// than the daemon; `None` when the process is gone.
pub fn of(id: u32) -> Option<(PathBuf, bool)> {
    let process = open(id)?;
    let path = path_of(&process)?;
    // SAFETY: the pseudo handle of this process needs no closing.
    let ours = elevated(unsafe { GetCurrentProcess() }).unwrap_or(false);
    let theirs = elevated(process.0).unwrap_or(true);
    Some((path, theirs && !ours))
}

/// Whether process `id` runs the executable named `file` (lower case).
pub fn is_named(id: u32, file: &str) -> bool {
    open(id)
        .and_then(|process| path_of(&process))
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().to_lowercase())
        })
        .is_some_and(|name| name == file)
}

/// The app an executable is: named by its file description.
pub fn app(path: &Path) -> App {
    let fallback = || {
        path.file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    App {
        path: path.to_owned(),
        name: description(path).unwrap_or_else(fallback),
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// The `FileDescription` in the executable's version info, in its first
/// language.
fn description(path: &Path) -> Option<String> {
    let file: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: `file` ends with a zero.
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(file.as_ptr()), None) };
    if size == 0 {
        return None;
    }
    let mut block = vec![0u8; size as usize];
    // SAFETY: the block is as large as asked for.
    unsafe {
        GetFileVersionInfoW(
            PCWSTR(file.as_ptr()),
            None,
            size,
            block.as_mut_ptr().cast::<c_void>(),
        )
    }
    .ok()?;
    let query = |name: &str| -> Option<(*const u8, usize)> {
        let name = wide(name);
        let mut found: *mut c_void = std::ptr::null_mut();
        let mut length = 0u32;
        // SAFETY: `block` holds the version info read above.
        let ok = unsafe {
            VerQueryValueW(
                block.as_ptr().cast::<c_void>(),
                PCWSTR(name.as_ptr()),
                &mut found,
                &mut length,
            )
        };
        (ok.as_bool() && !found.is_null() && length > 0)
            .then_some((found.cast::<u8>().cast_const(), length as usize))
    };
    let (languages, _) = query(r"\VarFileInfo\Translation")?;
    // SAFETY: the translation table starts with two u16: language, code page.
    let (language, page) = unsafe {
        let pair = languages.cast::<u16>();
        (pair.read_unaligned(), pair.add(1).read_unaligned())
    };
    let (text, length) = query(&format!(
        r"\StringFileInfo\{language:04x}{page:04x}\FileDescription"
    ))?;
    // SAFETY: the value is `length` u16, the last one possibly the end.
    let units = unsafe { std::slice::from_raw_parts(text.cast::<u16>(), length) };
    let name = String::from_utf16_lossy(units)
        .trim_end_matches('\0')
        .trim()
        .to_owned();
    (!name.is_empty()).then_some(name)
}
