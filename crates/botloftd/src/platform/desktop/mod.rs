//! The owner's desktop (spec 24): the windows open on their screen, which
//! app each one is from, what is never granted, a window read through its
//! accessibility tree, and a picture of a window.
//! Only on Windows; elsewhere every call says the desktop is not there.

mod blocked;
mod read;
// Only Windows takes pictures; the tests run everywhere.
#[cfg_attr(not(windows), allow(dead_code))]
mod shrink;
#[cfg(windows)]
mod windows;

use std::path::PathBuf;

pub use blocked::{Never, never};
pub use read::{Control, LINES_MAX, by_reference, reference, render};

/// The largest picture of a window the bot gets, in logical pixels (24.4).
pub const PICTURE_MAX: (u32, u32) = (1600, 1200);
/// The JPEG quality of a picture, as `browser_screenshot`'s.
#[cfg_attr(not(windows), allow(dead_code))]
const QUALITY: u8 = 70;

/// A program on the owner's computer, by its executable (spec 24.2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct App {
    pub path: PathBuf,
    /// The description in the file's version info ("Microsoft Excel"), or
    /// the file's name without `.exe`.
    pub name: String,
}

/// A window open on the owner's desktop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Window {
    /// The window's handle, as the number the bot names it by.
    pub id: u64,
    pub title: String,
    /// The window class, which tells some system windows apart.
    pub class: String,
    pub app: App,
    pub minimized: bool,
    /// Its process runs with more rights than the daemon (as administrator),
    /// or its rights could not be read.
    pub elevated: bool,
}

/// A picture of a window, as a JPEG.
#[derive(Debug, Clone)]
pub struct Picture {
    pub jpeg: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum DesktopError {
    #[error("the desktop is not available on this system yet")]
    Unavailable,
    #[error("that window is not open anymore")]
    Gone,
    #[error("that window is minimized, so there is nothing to see in it")]
    Minimized,
    #[error("Windows could not do it: {0}")]
    System(String),
    /// The control a reading named is not in the window anymore.
    #[error("that control is not in the window anymore")]
    NotThere,
    /// The control does not take that action through accessibility.
    #[error("that control cannot be {0} through accessibility")]
    Cannot(&'static str),
    /// A password field: the owner types there, never a bot (spec 24.3).
    #[error("that is a password field")]
    Password,
    /// A field that cannot be changed.
    #[error("that field cannot be changed")]
    ReadOnly,
    /// No option with that text in the list.
    #[error("there is no option {0:?} there")]
    NoOption(String),
}

/// What a bot does to a control through accessibility (spec 24.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Invokes a button or link, ticks a box, picks an item, opens or
    /// closes what expands.
    Click,
    /// Replaces a field's text.
    Type(String),
    /// Picks the option with this text in a list or combo box.
    Select(String),
    Scroll(Scroll),
}

/// Where `Action::Scroll` goes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scroll {
    Down,
    Up,
    Top,
    Bottom,
}

/// How an action went.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acted {
    Done,
    /// The app did not answer in time: it may be showing a dialog, or still
    /// busy with it.
    Waiting,
}

/// The windows open on the owner's desktop: shown ones, minimized too,
/// with a title, from every app (spec 24.4). What the bot may see of them
/// is the caller's to decide.
pub fn windows() -> Result<Vec<Window>, DesktopError> {
    #[cfg(windows)]
    {
        windows::list()
    }
    #[cfg(not(windows))]
    {
        Err(DesktopError::Unavailable)
    }
}

/// Calls `stop` each time the owner presses Ctrl+Alt+End, from now on
/// (spec 24.9). Refused when another program holds the shortcut.
pub fn on_stop_key(stop: impl Fn() + Send + 'static) -> Result<(), DesktopError> {
    #[cfg(windows)]
    {
        windows::on_stop_key(stop)
    }
    #[cfg(not(windows))]
    {
        let _ = stop;
        Err(DesktopError::Unavailable)
    }
}

/// How long ago the owner last used the mouse or keyboard (spec 24.8);
/// `None` where that cannot be known.
pub fn owner_idle() -> Option<std::time::Duration> {
    #[cfg(windows)]
    {
        windows::owner_idle()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

/// The controls of window `id` through its accessibility tree, in the
/// order they sit in it, the ones shown on screen (spec 24.5).
pub fn read(id: u64) -> Result<Vec<Control>, DesktopError> {
    #[cfg(windows)]
    {
        windows::read(id)
    }
    #[cfg(not(windows))]
    {
        let _ = id;
        Err(DesktopError::Unavailable)
    }
}

/// Does `action` to the control `target` (its `RuntimeId`, from a reading)
/// in window `id`, through accessibility, without the owner's mouse or
/// keyboard (spec 24.5).
pub fn act(id: u64, target: &[i32], action: &Action) -> Result<Acted, DesktopError> {
    #[cfg(windows)]
    {
        windows::act(id, target, action)
    }
    #[cfg(not(windows))]
    {
        let _ = (id, target, action);
        Err(DesktopError::Unavailable)
    }
}

/// A picture of the window `id`, only of it, even behind other windows, in
/// logical pixels up to [`PICTURE_MAX`] (spec 24.4).
pub fn picture(id: u64) -> Result<Picture, DesktopError> {
    #[cfg(windows)]
    {
        let (rgb, width, height, scale) = windows::capture(id)?;
        Ok(shrink::picture(
            &rgb,
            width,
            height,
            scale,
            PICTURE_MAX,
            QUALITY,
        ))
    }
    #[cfg(not(windows))]
    {
        let _ = id;
        Err(DesktopError::Unavailable)
    }
}
