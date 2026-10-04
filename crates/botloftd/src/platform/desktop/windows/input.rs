//! The owner's own use of the computer (spec 24.8), leaving out what a bot
//! typed or clicked with the real mouse and keyboard.

use std::sync::atomic::Ordering;
use std::time::Duration;

use windows::Win32::System::SystemInformation::GetTickCount;
use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};

use super::owner::{BOT_TICK, OWNER_TICK};

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
    // The last input was the bot's: the owner's came before it.
    let last = if last.dwTime == BOT_TICK.load(Ordering::SeqCst) {
        OWNER_TICK.load(Ordering::SeqCst)
    } else {
        last.dwTime
    };
    // SAFETY: a plain query. Both count milliseconds since boot and wrap
    // together, so the difference wraps right.
    let now = unsafe { GetTickCount() };
    Some(Duration::from_millis(u64::from(now.wrapping_sub(last))))
}
