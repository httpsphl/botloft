//! Handing a folder, a file or a web link to the system (spec 15.2):
//! Explorer on Windows, `open` on macOS, `xdg-open` on Linux. The callers
//! check what may be opened; nothing here runs a program the owner did not
//! pick.

use std::path::Path;
use std::process::Command;

#[cfg(windows)]
const LAUNCHER: &str = "explorer.exe";
#[cfg(target_os = "macos")]
const LAUNCHER: &str = "open";
#[cfg(all(unix, not(target_os = "macos")))]
const LAUNCHER: &str = "xdg-open";

fn spawn(command: &mut Command) -> Result<(), String> {
    command
        .spawn()
        .map(drop)
        .map_err(|err| format!("cannot run {LAUNCHER}: {err}"))
}

/// A folder in the file manager. On macOS an app bundle is a folder too,
/// and `open` would start it.
pub fn folder(path: &Path) -> Result<(), String> {
    if cfg!(target_os = "macos")
        && path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
    {
        return Err(format!("{} is an app, not a folder", path.display()));
    }
    spawn(Command::new(LAUNCHER).arg(path))
}

/// A document with the program the system uses for it.
pub fn file(path: &Path) -> Result<(), String> {
    spawn(Command::new(LAUNCHER).arg(path))
}

/// A web link in the default browser.
pub fn url(url: &str) -> Result<(), String> {
    spawn(Command::new(LAUNCHER).arg(url))
}

/// A file shown selected in its folder. Linux has no common way to select
/// it, so its folder opens.
pub fn reveal(path: &Path) -> Result<(), String> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        let text = path.to_string_lossy();
        if text.contains('"') {
            return Err(format!("{text} is not a file"));
        }
        // Explorer reads `/select,` and the path as one argument.
        spawn(Command::new(LAUNCHER).raw_arg(format!("/select,\"{text}\"")))
    }
    #[cfg(target_os = "macos")]
    {
        spawn(Command::new(LAUNCHER).arg("-R").arg(path))
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let folder = path.parent().unwrap_or(path);
        spawn(Command::new(LAUNCHER).arg(folder))
    }
}
