//! Which browser a bot runs (spec 21.2): the one in the config, else the
//! Edge that comes with Windows, or the first Edge, Chrome or Chromium
//! installed on macOS (Chrome or Chromium first on Linux).

use std::path::PathBuf;

/// What is missing when no browser is found.
#[cfg(windows)]
pub const NOT_FOUND: &str = "Microsoft Edge was not found on this computer";
#[cfg(not(windows))]
pub const NOT_FOUND: &str =
    "no Microsoft Edge, Google Chrome or Chromium was found on this computer";

/// The browser to run: `configured`, or one installed on the computer.
pub fn find(configured: &str) -> Option<PathBuf> {
    let configured = configured.trim();
    if !configured.is_empty() {
        let path = PathBuf::from(configured);
        return path.is_file().then_some(path);
    }
    installed().into_iter().find(|path| path.is_file())
}

#[cfg(windows)]
fn installed() -> Vec<PathBuf> {
    ["ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"]
        .iter()
        .filter_map(std::env::var_os)
        .map(|base| {
            PathBuf::from(base)
                .join("Microsoft")
                .join("Edge")
                .join("Application")
                .join("msedge.exe")
        })
        .collect()
}

/// The app bundles in `/Applications`, then in the owner's own.
#[cfg(target_os = "macos")]
fn installed() -> Vec<PathBuf> {
    const APPS: &[&str] = &[
        "Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        "Google Chrome.app/Contents/MacOS/Google Chrome",
        "Chromium.app/Contents/MacOS/Chromium",
    ];
    let folders = [
        Some(PathBuf::from("/Applications")),
        dirs::home_dir().map(|home| home.join("Applications")),
    ];
    folders
        .into_iter()
        .flatten()
        .flat_map(|folder| APPS.iter().map(move |app| folder.join(app)))
        .collect()
}

/// The commands the Chrome, Chromium and Edge packages install, on the
/// owner's `PATH`. Edge comes last: it is the rarer one on Linux, and on
/// the Ubuntu runner its first start with each new profile took seconds
/// where Chrome took under one.
#[cfg(all(unix, not(target_os = "macos")))]
fn installed() -> Vec<PathBuf> {
    const COMMANDS: &[&str] = &[
        "google-chrome-stable",
        "google-chrome",
        "chromium",
        "chromium-browser",
        "microsoft-edge-stable",
        "microsoft-edge",
    ];
    let path = crate::platform::user_environment()
        .ok()
        .and_then(|vars| vars.into_iter().find(|(name, _)| name == "PATH"))
        .map(|(_, value)| value)
        .or_else(|| std::env::var_os("PATH"))
        .unwrap_or_default();
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();
    COMMANDS
        .iter()
        .flat_map(|command| dirs.iter().map(move |dir| dir.join(command)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_configured_browser_must_exist() {
        assert_eq!(find(r"C:\nowhere\msedge.exe"), None);
        assert_eq!(find("/nowhere/chromium"), None);
    }

    #[test]
    fn a_configured_browser_that_exists_is_used() {
        let dir = tempfile::tempdir().expect("tempdir");
        let program = dir.path().join("browser");
        std::fs::write(&program, b"").expect("write");
        let configured = format!("  {}  ", program.display());
        assert_eq!(find(&configured), Some(program));
    }
}
