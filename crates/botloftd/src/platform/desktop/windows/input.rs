//! The owner's own use of the computer (spec 24.8).

use std::time::Duration;

use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

/// How long ago the session last had mouse or keyboard input.
pub fn owner_idle() -> Option<Duration> {
    let mut last = LASTINPUTINFO {
        cbSize: size_of::<LASTINPUTINFO>() as u32,
        dwTime: 0,
    };
    // SAFETY: `last` is a valid LASTINPUTINFO with its size set.
    if !unsafe { GetLastInputInfo(&mut last) }.as_bool() {
        return None;
    }
    // SAFETY: a plain query. Both count milliseconds since boot and wrap
    // together, so the difference wraps right.
    let now = unsafe { GetTickCount() };
    Some(Duration::from_millis(u64::from(
        now.wrapping_sub(last.dwTime),
    )))
}
