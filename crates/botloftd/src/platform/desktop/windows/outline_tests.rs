//! With a real window: the outline goes around its frame, right above it
//! in the order of windows, follows it when it moves, steps aside while it
//! is minimized and goes when asked. It draws a window, but sends no input.

use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, GW_HWNDPREV, GetWindow, GetWindowRect, IsWindowVisible, SW_MINIMIZE, SW_RESTORE,
    SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SetWindowPos, ShowWindow,
};
use windows::core::w;

use super::test_window::TestWindow;
use super::{capture, handle, outline};

fn outline_window() -> Option<HWND> {
    // SAFETY: a plain query.
    unsafe { FindWindowW(w!("BotloftDesktopOutline"), None) }.ok()
}

fn visible() -> bool {
    // SAFETY: a plain query.
    outline_window().is_some_and(|hwnd| unsafe { IsWindowVisible(hwnd) }.as_bool())
}

fn place() -> RECT {
    let mut place = RECT::default();
    if let Some(hwnd) = outline_window() {
        // SAFETY: `place` is a valid out-pointer.
        let _ = unsafe { GetWindowRect(hwnd, &mut place) };
    }
    place
}

/// Waits a little for the outline's thread to catch up.
fn soon(what: &str, done: impl Fn() -> bool) {
    let start = Instant::now();
    while !done() {
        assert!(start.elapsed() < Duration::from_secs(3), "never: {what}");
        std::thread::sleep(Duration::from_millis(30));
    }
}

/// The frame grown by the ring on each side.
fn around(frame: [i32; 4], rect: RECT) -> bool {
    let ring = frame[0] - rect.left;
    ring > 0
        && rect.top == frame[1] - ring
        && rect.right == frame[0] + frame[2] + ring
        && rect.bottom == frame[1] + frame[3] + ring
}

#[test]
fn the_outline_goes_around_the_window_and_follows_it() {
    let window = TestWindow::open("Botloft desktop test: outline");
    let app = handle(window.id);
    outline::show(window.id, (0x5E, 0xC8, 0xFF), None);
    soon("shown", visible);
    let frame = capture::frame(window.id).expect("frame");
    soon("around the frame", || around(frame, place()));
    // SAFETY: a plain query.
    let above = unsafe { GetWindow(app, GW_HWNDPREV) }.ok();
    assert_eq!(above, outline_window(), "right above the app");

    // The window moves; the outline follows on the next show.
    // SAFETY: moves the test's own window.
    let _ = unsafe {
        SetWindowPos(
            app,
            None,
            frame[0] + 40,
            frame[1] + 30,
            0,
            0,
            SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
        )
    };
    let moved = capture::frame(window.id).expect("frame");
    outline::show(window.id, (0x5E, 0xC8, 0xFF), None);
    soon("moved along", || around(moved, place()));

    // SAFETY: minimizes and restores the test's own window.
    let _ = unsafe { ShowWindow(app, SW_MINIMIZE) };
    outline::show(window.id, (0x5E, 0xC8, 0xFF), None);
    soon("out of the way", || !visible());
    let _ = unsafe { ShowWindow(app, SW_RESTORE) };
    outline::show(window.id, (0x5E, 0xC8, 0xFF), None);
    soon("back", visible);

    outline::hide();
    soon("hidden", || !visible());
}
