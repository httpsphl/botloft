//! Finding and checking the Claude Code executable (spec 7.4).

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use botloft_core::protocol::ClaudeAccount;

use super::AuthStatus;

/// First version with the inbox as a named pipe on native Windows.
pub const MIN_VERSION: (u32, u32, u32) = (2, 1, 234);
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);

/// A usable Claude Code executable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claude {
    pub path: PathBuf,
    pub version: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ClaudeError {
    #[error("Claude Code was not found in PATH; install it or set claude_path in config.toml")]
    NotFound,
    #[error(
        "{0} is the npm launcher; Botloft runs the native claude.exe. Install Claude Code with the native installer or point claude_path at the .exe"
    )]
    Launcher(PathBuf),
    #[error("cannot run {path}: {source}")]
    Run { path: PathBuf, source: io::Error },
    #[error("cannot read the user environment: {0}")]
    Environment(io::Error),
    #[error("`claude --version` did not report a version")]
    UnknownVersion,
    #[error("Claude Code {found} is too old; Botloft needs {needed} or later")]
    TooOld { found: String, needed: String },
}

/// The configured path, or the first `claude` executable on `PATH`.
pub fn locate(configured: &str, env: &[(OsString, OsString)]) -> Result<PathBuf, ClaudeError> {
    let configured = configured.trim();
    if !configured.is_empty() {
        return not_a_launcher(PathBuf::from(configured));
    }
    let path_var = env
        .iter()
        .find(|(name, _)| name.to_string_lossy().eq_ignore_ascii_case("PATH"))
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    let dirs: Vec<PathBuf> = path_entries(&path_var.to_string_lossy())
        .chain(usual_places(env))
        .collect();
    for dir in dirs {
        for name in CANDIDATES {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return not_a_launcher(candidate);
            }
        }
    }
    Err(ClaudeError::NotFound)
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

/// Where installers put Claude Code off Windows, for a profile that never
/// added it to `PATH`: the native installer, the older local install and
/// Homebrew.
fn usual_places(env: &[(OsString, OsString)]) -> Vec<PathBuf> {
    if cfg!(windows) {
        return Vec::new();
    }
    let home = env
        .iter()
        .find(|(name, _)| name == "HOME")
        .map(|(_, value)| PathBuf::from(value));
    let mut places: Vec<PathBuf> = home
        .iter()
        .flat_map(|home| [home.join(".local/bin"), home.join(".claude/local")])
        .collect();
    places.extend(["/opt/homebrew/bin", "/usr/local/bin"].map(PathBuf::from));
    places
}

#[cfg(windows)]
const CANDIDATES: &[&str] = &["claude.exe", "claude.cmd"];
#[cfg(not(windows))]
const CANDIDATES: &[&str] = &["claude"];

/// `claude.cmd` needs `cmd.exe` in between, which mangles the quoting of
/// arguments passed through the pseudo terminal; only real executables run.
fn not_a_launcher(path: PathBuf) -> Result<PathBuf, ClaudeError> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase());
    match ext.as_deref() {
        Some("cmd" | "bat" | "ps1") => Err(ClaudeError::Launcher(path)),
        _ => Ok(path),
    }
}

/// Runs `claude --version` and checks it against [`MIN_VERSION`].
pub fn probe(path: &Path, env: &[(OsString, OsString)]) -> Result<Claude, ClaudeError> {
    let output = run(path, &["--version"], env).map_err(|source| ClaudeError::Run {
        path: path.to_owned(),
        source,
    })?;
    let text = String::from_utf8_lossy(&output.stdout);
    let (version, parsed) = parse_version(&text).ok_or(ClaudeError::UnknownVersion)?;
    if parsed < MIN_VERSION {
        let (major, minor, patch) = MIN_VERSION;
        return Err(ClaudeError::TooOld {
            found: version,
            needed: format!("{major}.{minor}.{patch}"),
        });
    }
    Ok(Claude {
        path: path.to_owned(),
        version,
    })
}

/// Runs `claude auth status`, documented to print JSON with `loggedIn` and
/// to exit with 0 when signed in and 1 when not (spec 19).
pub fn auth_status(path: &Path, env: &[(OsString, OsString)]) -> io::Result<AuthStatus> {
    let output = run(path, &["auth", "status", "--json"], env)?;
    Ok(parse_auth_status(&output.stdout).unwrap_or(AuthStatus {
        signed_in: output.status.success(),
        account: None,
    }))
}

/// Reads `loggedIn`, and when signed in `email`, `subscriptionType` and
/// `orgName` (seen with 2.1.284).
fn parse_auth_status(stdout: &[u8]) -> Option<AuthStatus> {
    let json = serde_json::from_slice::<serde_json::Value>(stdout).ok()?;
    let signed_in = json.get("loggedIn")?.as_bool()?;
    let text = |key: &str| {
        json.get(key)
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    Some(AuthStatus {
        signed_in,
        account: signed_in.then(|| ClaudeAccount {
            email: text("email"),
            plan: text("subscriptionType"),
            organization: text("orgName"),
        }),
    })
}

/// Runs Claude Code without a window and with `env` only, giving up after
/// [`PROBE_TIMEOUT`].
fn run(path: &Path, args: &[&str], env: &[(OsString, OsString)]) -> io::Result<Output> {
    let mut command = Command::new(path);
    command
        .args(args)
        .env_clear()
        .envs(env.iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn()?;
    let started = Instant::now();
    while child.try_wait()?.is_none() {
        if started.elapsed() > PROBE_TIMEOUT {
            let _ = child.kill();
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("`claude {}` did not finish", args.join(" ")),
            ));
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    child.wait_with_output()
}

/// Reads `2.1.283` from output such as `2.1.283 (Claude Code)`.
pub fn parse_version(text: &str) -> Option<(String, (u32, u32, u32))> {
    let token = text.split_whitespace().next()?;
    let mut parts = token.split('.').map(|part| part.parse::<u32>().ok());
    let parsed = (parts.next()??, parts.next()??, parts.next()??);
    Some((token.to_owned(), parsed))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_version_line() {
        assert_eq!(
            parse_version("2.1.283 (Claude Code)\n"),
            Some(("2.1.283".to_owned(), (2, 1, 283)))
        );
        assert_eq!(parse_version("Claude Code"), None);
        assert_eq!(parse_version(""), None);
        assert!(parse_version("2.1.74 (Claude Code)").is_some_and(|(_, v)| v < MIN_VERSION));
    }

    #[test]
    fn reads_the_sign_in_and_account_from_auth_status() {
        let signed_in = parse_auth_status(
            br#"{"loggedIn": true, "authMethod": "claude.ai", "email": "ana@example.com",
                "orgName": "Exemplo", "subscriptionType": "max"}"#,
        )
        .expect("parsed");
        assert!(signed_in.signed_in);
        assert_eq!(
            signed_in.account,
            Some(ClaudeAccount {
                email: Some("ana@example.com".into()),
                plan: Some("max".into()),
                organization: Some("Exemplo".into()),
            })
        );
        let signed_out =
            parse_auth_status(br#"{"loggedIn": false, "authMethod": "none"}"#).expect("parsed");
        assert_eq!(
            signed_out,
            AuthStatus {
                signed_in: false,
                account: None
            }
        );
        assert_eq!(parse_auth_status(b"Logged in"), None);
    }

    #[test]
    fn refuses_npm_launchers() {
        assert!(matches!(
            locate(r"C:\npm\claude.cmd", &[]),
            Err(ClaudeError::Launcher(_))
        ));
        assert_eq!(
            locate(r"C:\bin\claude.exe", &[]).expect("configured path"),
            PathBuf::from(r"C:\bin\claude.exe")
        );
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
        let env = vec![(OsString::from("Path"), dir.path().as_os_str().to_owned())];
        assert_eq!(locate("", &env).expect("found"), exe);
        assert!(matches!(locate("", &[]), Err(ClaudeError::NotFound)));
    }

    #[cfg(unix)]
    #[test]
    fn finds_the_native_install_off_path() {
        let home = tempfile::tempdir().expect("tempdir");
        let bin = home.path().join(".local/bin");
        std::fs::create_dir_all(&bin).expect("bin");
        std::fs::write(bin.join("claude"), b"").expect("fake claude");
        let env = vec![
            (OsString::from("PATH"), OsString::from("/nowhere")),
            (OsString::from("HOME"), home.path().as_os_str().to_owned()),
        ];
        assert_eq!(locate("", &env).expect("found"), bin.join("claude"));
    }
}
