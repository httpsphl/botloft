//! The shortcut that stops every bot on the desktop, Ctrl+Alt+Esc (spec
//! 24.9): registered for the whole session on a thread of its own, so it
//! works with the app closed.

use std::sync::mpsc;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, RegisterHotKey, VK_ESCAPE,
};
use windows::Win32::UI::WindowsAndMessaging::{GetMessageW, MSG, WM_HOTKEY};

use super::super::DesktopError;

/// The shortcut's id for this thread.
const STOP: i32 = 1;

/// Calls `stop` each time the owner presses the shortcut, from now on.
/// Refused when another program holds the shortcut.
pub fn on_stop_key(stop: impl Fn() + Send + 'static) -> Result<(), DesktopError> {
    let (told, registered) = mpsc::channel();
    std::thread::Builder::new()
        .name("desktop-stop-key".to_owned())
        .spawn(move || {
            // SAFETY: a shortcut for this thread's own message queue.
            let held = unsafe {
                RegisterHotKey(
                    None,
                    STOP,
                    MOD_CONTROL | MOD_ALT | MOD_NOREPEAT,
                    u32::from(VK_ESCAPE.0),
                )
            };
            let ok = held.is_ok();
            let _ = told.send(held.map_err(|err| DesktopError::System(err.message())));
            if !ok {
                return;
            }
            let mut message = MSG::default();
            // SAFETY: this thread's own queue, read until the process ends.
            while unsafe { GetMessageW(&mut message, None, 0, 0) }.as_bool() {
                if message.message == WM_HOTKEY && message.wParam.0 == STOP as usize {
                    stop();
                }
            }
        })
        .map_err(|err| DesktopError::System(err.to_string()))?;
    registered.recv().unwrap_or_else(|_| {
        Err(DesktopError::System(
            "the shortcut's thread ended".to_owned(),
        ))
    })
}
