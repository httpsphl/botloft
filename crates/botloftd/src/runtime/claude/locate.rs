//! Where Claude Code is installed (spec 7.4).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::ClaudeError;

/// The configured path, or the first Claude Code executable on `PATH` or in
/// the places its installers use.
pub fn locate(configured: &str, env: &[(OsString, OsString)]) -> Result<PathBuf, ClaudeError> {
    let configured = configured.trim();
    if !configured.is_empty() {
        return executable(PathBuf::from(configured));
    }
    let path_var = var(env, "PATH").unwrap_or_default();
    let dirs: Vec<PathBuf> = path_entries(&path_var.to_string_lossy())
        .chain(usual_places(env))
        .collect();
    // A launcher that cannot run is reported only when nothing later can.
    let mut launcher = None;
    for dir in dirs {
        for name in CANDIDATES {
            let candidate = dir.join(name);
            if !candidate.is_file() {
                continue;
            }
            match executable(candidate) {
                Ok(path) => return Ok(path),
                Err(err) => {
                    launcher.get_or_insert(err);
                }
            }
        }
    }
    Err(launcher.unwrap_or(ClaudeError::NotFound))
}

/// The value of `name` in `env`; Windows names ignore case.
fn var(env: &[(OsString, OsString)], name: &str) -> Option<OsString> {
    env.iter()
        .find(|(key, _)| {
            let key = key.to_string_lossy();
            if cfg!(windows) {
                key.eq_ignore_ascii_case(name)
            } else {
                key == name
            }
        })
        .map(|(_, value)| value.clone())
}

/// Splits `PATH` the way Windows searches it: on `;` only. A stray quote in
/// one entry (common on real systems) must not swallow the rest, which
/// `std::env::split_paths` does because it treats quotes as grouping.
fn path_entries(path: &str) -> impl Iterator<Item = PathBuf> + '_ {
    let separator = if cfg!(windows) { ';' } else { ':' };
    path.split(separator)
        .map(|entry| entry.trim().trim_matches('"'))
        .filter(|entry| !entry.is_empty())
        .map(PathBuf::from)
}

/// Where installers put Claude Code, for a profile that never added it to
/// `PATH` (or added it only in a shell profile). Windows: the native
/// installer, WinGet and npm. Elsewhere: the native installer, the older
/// local install and Homebrew.
fn usual_places(env: &[(OsString, OsString)]) -> Vec<PathBuf> {
    let under = |name: &str, rest: &str| var(env, name).map(|base| PathBuf::from(base).join(rest));
    if cfg!(windows) {
        return [
            under("USERPROFILE", r".local\bin"),
            under("LOCALAPPDATA", r"Microsoft\WinGet\Links"),
            under("APPDATA", "npm"),
        ]
        .into_iter()
        .flatten()
        .collect();
    }
    let mut places: Vec<PathBuf> = [under("HOME", ".local/bin"), under("HOME", ".claude/local")]
        .into_iter()
        .flatten()
        .collect();
    places.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));
    places
}

#[cfg(windows)]
const CANDIDATES: &[&str] = &["claude.exe", "claude.cmd"];
#[cfg(not(windows))]
const CANDIDATES: &[&str] = &["claude"];

/// `path` when it is a real executable. An npm launcher (`claude.cmd`) needs
/// `cmd.exe` in between, which mangles the quoting of arguments passed
/// through the pseudo terminal; the native binary the npm package installs
/// next to it runs instead.
fn executable(path: PathBuf) -> Result<PathBuf, ClaudeError> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase());
    if !matches!(ext.as_deref(), Some("cmd" | "bat" | "ps1")) {
        return Ok(path);
    }
    let native = path.parent().and_then(npm_native);
    native.ok_or(ClaudeError::Launcher(path))
}

/// Where the npm package keeps the native binary, relative to the folder
/// with its launchers (seen with 2.1.287).
const NPM_NATIVE: &[&str] = &[
    "node_modules",
    "@anthropic-ai",
    "claude-code",
    "bin",
    "claude.exe",
];
/// The package ships a small placeholder script there, which its
/// postinstall replaces with the binary; a failed or skipped postinstall
/// leaves the placeholder.
const PLACEHOLDER_MAX: u64 = 4096;

/// The native binary of an npm install in `dir`, if postinstall put it there.
fn npm_native(dir: &Path) -> Option<PathBuf> {
    let exe = dir.join(NPM_NATIVE.iter().collect::<PathBuf>());
    let size = std::fs::metadata(&exe).ok()?.len();
    (size >= PLACEHOLDER_MAX).then_some(exe)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An npm global folder with its launcher and, if `native` is given, a
    /// `bin\claude.exe` of that many bytes.
    fn npm_folder(native: Option<u64>) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("claude.cmd"), b"@echo off").expect("launcher");
        if let Some(size) = native {
            let exe = dir.path().join(NPM_NATIVE.iter().collect::<PathBuf>());
            std::fs::create_dir_all(exe.parent().expect("bin")).expect("bin");
            std::fs::write(&exe, vec![0u8; size as usize]).expect("native");
        }
        dir
    }

    #[test]
    fn refuses_npm_launchers_without_a_native_binary() {
        assert!(matches!(
            locate(r"C:\npm\claude.cmd", &[]),
            Err(ClaudeError::Launcher(_))
        ));
        assert_eq!(
            locate(r"C:\bin\claude.exe", &[]).expect("configured path"),
            PathBuf::from(r"C:\bin\claude.exe")
        );
        let placeholder = npm_folder(Some(500));
        let launcher = placeholder.path().join("claude.cmd");
        assert!(matches!(
            locate(&launcher.to_string_lossy(), &[]),
            Err(ClaudeError::Launcher(_))
        ));
    }

    #[test]
    fn runs_the_native_binary_of_an_npm_install() {
        let npm = npm_folder(Some(PLACEHOLDER_MAX));
        let launcher = npm.path().join("claude.cmd");
        assert_eq!(
            locate(&launcher.to_string_lossy(), &[]).expect("native"),
            npm.path().join(NPM_NATIVE.iter().collect::<PathBuf>())
        );
    }

    #[cfg(windows)]
    #[test]
    fn finds_npm_installs_and_looks_past_launchers_on_path() {
        let npm = npm_folder(Some(PLACEHOLDER_MAX));
        let env = vec![(OsString::from("Path"), npm.path().as_os_str().to_owned())];
        assert_eq!(
            locate("", &env).expect("native"),
            npm.path().join(NPM_NATIVE.iter().collect::<PathBuf>())
        );

        let placeholder = npm_folder(Some(500));
        let native = tempfile::tempdir().expect("tempdir");
        std::fs::write(native.path().join("claude.exe"), b"").expect("fake exe");
        let path = format!(
            "{};{}",
            placeholder.path().display(),
            native.path().display()
        );
        let env = vec![(OsString::from("Path"), OsString::from(path))];
        assert_eq!(
            locate("", &env).expect("later entry"),
            native.path().join("claude.exe")
        );

        let env = vec![(
            OsString::from("Path"),
            placeholder.path().as_os_str().to_owned(),
        )];
        assert!(matches!(locate("", &env), Err(ClaudeError::Launcher(_))));
    }

    #[cfg(windows)]
    #[test]
    fn a_stray_quote_in_path_does_not_hide_later_entries() {
        let path = r#"C:\A";C:\B\;;"C:\Quoted Dir";C:\Users\me\.local\bin"#;
        let entries: Vec<PathBuf> = path_entries(path).collect();
        let expected = [
            r"C:\A",
            r"C:\B\",
            r"C:\Quoted Dir",
            r"C:\Users\me\.local\bin",
        ];
        assert_eq!(entries, expected.map(PathBuf::from));
    }

    #[test]
    fn searches_the_given_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let exe = dir.path().join(CANDIDATES[0]);
        std::fs::write(&exe, b"").expect("fake exe");
        let env = vec![(OsString::from("PATH"), dir.path().as_os_str().to_owned())];
        assert_eq!(locate("", &env).expect("found"), exe);
        assert!(matches!(locate("", &[]), Err(ClaudeError::NotFound)));
    }

    #[test]
    fn finds_the_native_install_off_path() {
        let home = tempfile::tempdir().expect("tempdir");
        let bin = home.path().join(".local").join("bin");
        std::fs::create_dir_all(&bin).expect("bin");
        let exe = bin.join(CANDIDATES[0]);
        std::fs::write(&exe, b"").expect("fake claude");
        let home_var = if cfg!(windows) { "USERPROFILE" } else { "HOME" };
        let env = vec![
            (OsString::from("PATH"), OsString::from("/nowhere")),
            (OsString::from(home_var), home.path().as_os_str().to_owned()),
        ];
        assert_eq!(locate("", &env).expect("found"), exe);
    }
}
