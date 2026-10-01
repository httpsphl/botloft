//! Signing in to Claude Code from the app (spec 15.2): `claude auth login`
//! in a window of its own, where the owner sees what happens and can paste
//! a code if the browser asks for one. The app launches it, not the daemon,
//! because a window started by the foreground app comes to the front.

use std::path::Path;
#[cfg(windows)]
use std::process::Command;

/// Whether `path` is a Claude Code executable the app may run: the daemon
/// found it, and the app still checks it is `claude.exe` and nothing else.
fn is_claude(path: &Path) -> bool {
    let expected = if cfg!(windows) {
        "claude.exe"
    } else {
        "claude"
    };
    path.is_absolute()
        && path
            .file_name()
            .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(expected))
        && path.is_file()
}

/// Runs `claude auth login` in a new console window (a terminal window off
/// Windows) and waits for it. Returns whether it signed in (exit code 0).
pub fn sign_in(path: &Path) -> Result<bool, String> {
    if !is_claude(path) {
        return Err(format!("{} is not Claude Code", path.display()));
    }
    // An app started from inside a Claude Code session would pass its
    // session variables on, and the login would think it runs inside it.
    let session: Vec<String> = std::env::vars_os()
        .map(|(name, _)| name.to_string_lossy().into_owned())
        .filter(|name| name == "CLAUDECODE" || name.starts_with("CLAUDE_CODE_"))
        .collect();
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;
        let mut command = Command::new(path);
        command
            .args(["auth", "login"])
            .creation_flags(CREATE_NEW_CONSOLE);
        for name in &session {
            command.env_remove(name);
        }
        let status = command
            .status()
            .map_err(|err| format!("cannot run {}: {err}", path.display()))?;
        Ok(status.success())
    }
    #[cfg(unix)]
    {
        crate::terminal::run(path, &["auth", "login"], &session)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_claude_code_itself_runs() {
        let dir = std::env::temp_dir().join(format!("botloft-claude-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let name = if cfg!(windows) {
            "claude.exe"
        } else {
            "claude"
        };
        let claude = dir.join(name);
        std::fs::write(&claude, b"").expect("write");
        let other = dir.join("calc.exe");
        std::fs::write(&other, b"").expect("write");
        assert!(is_claude(&claude));
        assert!(!is_claude(&other));
        assert!(!is_claude(Path::new(name)), "relative paths are refused");
        assert!(!is_claude(&dir.join("missing").join(name)));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
