//! What the daemon draws on the owner's screen while a bot uses it: the
//! notice of the real mouse and keyboard (spec 24.7) and the outline around
//! the window the bot uses (spec 24.9).

#[cfg(windows)]
use super::windows;

/// The bot's cursor on the owner's screen, over the window it uses.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenCursor {
    /// Where, as fractions of the window's frame from its top left corner.
    pub x: f64,
    pub y: f64,
    /// A ring shows where it clicked.
    pub click: bool,
    /// Dots show while it types.
    pub typing: bool,
    /// The bot's name, beside the arrow.
    pub name: String,
    /// When the bot acted: a new one glides there.
    pub at: i64,
}

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

/// Outlines window `over` in `color`, with the bot's `cursor` in it, or
/// moves the outline to where the window is now; hidden while the window
/// is minimized or gone.
pub fn outline_show(over: u64, color: (u8, u8, u8), cursor: Option<ScreenCursor>) {
    #[cfg(windows)]
    windows::outline_show(over, color, cursor);
    #[cfg(not(windows))]
    let _ = (over, color, cursor);
}

/// Takes the outline away.
pub fn outline_hide() {
    #[cfg(windows)]
    windows::outline_hide();
}
