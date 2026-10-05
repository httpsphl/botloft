//! What the daemon draws on the owner's screen while a bot uses it: the
//! notice of the real mouse and keyboard (spec 24.7) and the outline around
//! the window the bot uses (spec 24.9).

#[cfg(windows)]
use super::windows;

/// Shows the notice that a bot uses the real mouse and keyboard: `text`
/// in a pill, a border in `color` around the monitor of window `over`
/// (spec 24.7).
pub fn notice_show(text: &str, color: (u8, u8, u8), over: u64) {
    #[cfg(windows)]
    windows::notice_show(text, color, over);
    #[cfg(not(windows))]
    let _ = (text, color, over);
}

/// Takes the notice away.
pub fn notice_hide() {
    #[cfg(windows)]
    windows::notice_hide();
}

/// Outlines window `over` in `color`, or moves the outline to where the
/// window is now; hidden while the window is minimized or gone.
pub fn outline_show(over: u64, color: (u8, u8, u8)) {
    #[cfg(windows)]
    windows::outline_show(over, color);
    #[cfg(not(windows))]
    let _ = (over, color);
}

/// Takes the outline away.
pub fn outline_hide() {
    #[cfg(windows)]
    windows::outline_hide();
}
