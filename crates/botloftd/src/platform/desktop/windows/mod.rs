//! The owner's windows on Windows (spec 24.4): the top-level windows of
//! their session, which program each one is from, and whether it runs as
//! administrator.

mod act;
#[cfg(test)]
mod act_tests;
mod capture;
mod input;
mod notice;
mod outline;
mod outline_cursor;
#[cfg(test)]
mod outline_tests;
mod owner;
mod process;
mod real;
#[cfg(test)]
mod real_tests;
mod stop_key;
#[cfg(test)]
mod test_window;
#[cfg(test)]
mod tests;
mod uia;
mod uia_control;

use std::collections::HashMap;
use std::ffi::c_void;
use std::path::PathBuf;

use windows::Win32::Foundation::{HWND, LPARAM};
use windows::Win32::Graphics::Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, GWL_EXSTYLE, GetClassNameW, GetWindowLongPtrW,
    GetWindowTextLengthW, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    WS_EX_TOOLWINDOW,
};
use windows::core::{BOOL, w};

pub use act::act;
pub use capture::{capture, frame};
pub use input::owner_idle;
pub use notice::{hide as notice_hide, show as notice_show};
pub use outline::{hide as outline_hide, show as outline_show};
pub use real::{click as real_click, press as real_press, type_text as real_type};
pub use stop_key::on_stop_key;
pub use uia::read;

use super::{App, DesktopError, Window};

/// The process that frames the windows of Store apps: the app itself is
/// the process of the frame's core window.
const FRAME_HOST: &str = "applicationframehost.exe";

pub fn list() -> Result<Vec<Window>, DesktopError> {
    let mut handles: Vec<HWND> = Vec::new();
    // SAFETY: the callback only pushes into `handles`, which outlives the
    // call.
    unsafe {
        EnumWindows(
            Some(collect),
            LPARAM(std::ptr::from_mut(&mut handles) as isize),
        )
    }
    .map_err(|err| DesktopError::System(err.message()))?;
    let mut apps: HashMap<PathBuf, App> = HashMap::new();
    let mut found = Vec::new();
    for hwnd in handles {
        if !shown(hwnd) {
            continue;
        }
        let title = text(hwnd);
        if title.trim().is_empty() {
            continue;
        }
        let class = class_of(hwnd);
        let Some((path, elevated)) = process::of(app_window(hwnd)) else {
            continue;
        };
        let app = apps
            .entry(path.clone())
            .or_insert_with(|| process::app(&path))
            .clone();
        found.push(Window {
            id: hwnd.0 as usize as u64,
            title,
            class,
            app,
            // SAFETY: a plain query on a window handle.
            minimized: unsafe { IsIconic(hwnd) }.as_bool(),
            elevated,
        });
    }
    Ok(found)
}

unsafe extern "system" fn collect(hwnd: HWND, found: LPARAM) -> BOOL {
    // SAFETY: `found` is the `Vec` `list` passed in, alive for the call.
    let handles = unsafe { &mut *(found.0 as *mut Vec<HWND>) };
    handles.push(hwnd);
    BOOL(1)
}

/// A window the owner can see on the taskbar or the screen: shown, not a
/// tool window, and not cloaked (Store apps kept hidden, other virtual
/// desktops).
fn shown(hwnd: HWND) -> bool {
    // SAFETY: plain queries on a window handle.
    unsafe {
        if !IsWindowVisible(hwnd).as_bool() {
            return false;
        }
        if GetWindowLongPtrW(hwnd, GWL_EXSTYLE) & WS_EX_TOOLWINDOW.0 as isize != 0 {
            return false;
        }
        let mut cloaked = 0u32;
        let read = DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            std::ptr::from_mut(&mut cloaked).cast::<c_void>(),
            size_of::<u32>() as u32,
        );
        read.is_err() || cloaked == 0
    }
}

pub(super) fn handle(id: u64) -> HWND {
    HWND(id as usize as *mut c_void)
}

fn text(hwnd: HWND) -> String {
    // SAFETY: the buffer is as long as the length asked for, plus the end.
    unsafe {
        let length = GetWindowTextLengthW(hwnd).max(0) as usize;
        let mut buffer = vec![0u16; length + 1];
        let copied = GetWindowTextW(hwnd, &mut buffer).max(0) as usize;
        String::from_utf16_lossy(&buffer[..copied])
    }
}

fn class_of(hwnd: HWND) -> String {
    let mut buffer = [0u16; 256];
    // SAFETY: the buffer is valid for its length.
    let copied = unsafe { GetClassNameW(hwnd, &mut buffer) }.max(0) as usize;
    String::from_utf16_lossy(&buffer[..copied])
}

fn process_id(hwnd: HWND) -> u32 {
    let mut id = 0u32;
    // SAFETY: `id` is a valid out-pointer.
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut id)) };
    id
}

/// The process a window belongs to, for its app: a Store app's frame
/// belongs to the frame host, and the app is its core window's process.
fn app_window(hwnd: HWND) -> u32 {
    let id = process_id(hwnd);
    if !process::is_named(id, FRAME_HOST) {
        return id;
    }
    // SAFETY: a plain query on a window handle and a constant class name.
    let core = unsafe { FindWindowExW(Some(hwnd), None, w!("Windows.UI.Core.CoreWindow"), None) };
    match core {
        Ok(core) if !core.is_invalid() => process_id(core),
        _ => id,
    }
}
