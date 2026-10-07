//! The environment a new login of the owner would get (spec 14.1): what
//! their login shell sets up, so bots find what the owner's terminal finds
//! (`~/.local/bin`, Homebrew) even when the system started the daemon with
//! a bare `PATH`.

use std::ffi::OsString;
use std::io::Read as _;
use std::os::unix::ffi::{OsStrExt as _, OsStringExt as _};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::ProcessJob;

/// A login shell that takes longer is stuck (waiting for input, say).
const SHELL_TIMEOUT: Duration = Duration::from_secs(10);
/// How long one reading of the shell's environment is reused. Shorter than
/// the 30 s in which Botloft notices Claude Code installed (15.1).
const FRESH_FOR: Duration = Duration::from_secs(20);
/// Printed before the variables, after anything the profile prints.
const MARKER: &[u8] = b"BOTLOFT_ENVIRONMENT\0";
/// Variables the shell sets for itself.
const SHELL_OWN: &[&str] = &["_", "SHLVL", "PWD", "OLDPWD"];

type Vars = Vec<(OsString, OsString)>;

static CACHE: Mutex<Option<(Instant, Vars)>> = Mutex::new(None);

/// The login shell's environment, else the daemon's, without Claude Code
/// session variables either way.
pub fn user_environment() -> std::io::Result<Vars> {
    let mut cache = CACHE.lock().unwrap_or_else(|p| p.into_inner());
    let vars = match cache.as_ref() {
        Some((read, vars)) if read.elapsed() < FRESH_FOR => vars.clone(),
        _ => {
            let vars = login_environment().unwrap_or_else(|| std::env::vars_os().collect());
            *cache = Some((Instant::now(), vars.clone()));
            vars
        }
    };
    Ok(vars
        .into_iter()
        .filter(|(name, _)| !is_claude_session(&name.to_string_lossy()))
        .collect())
}

fn is_claude_session(name: &str) -> bool {
    name == "CLAUDECODE" || name.starts_with("CLAUDE_CODE_")
}

fn login_shell() -> PathBuf {
    std::env::var_os("SHELL")
        .filter(|shell| !shell.is_empty())
        .map(PathBuf::from)
        .or_else(|| super::account::current().and_then(|account| account.shell))
        .unwrap_or_else(|| PathBuf::from("/bin/sh"))
}

/// Runs the login shell without a terminal and reads `env -0`. `None` when
/// the shell fails, takes too long or prints no variables.
fn login_environment() -> Option<Vars> {
    let mut command = Command::new(login_shell());
    command
        .args([
            "-l",
            "-c",
            r#"printf '%s\0' BOTLOFT_ENVIRONMENT; exec env -0"#,
        ])
        .env_clear()
        .envs(std::env::vars_os().filter(|(name, _)| !is_claude_session(&name.to_string_lossy())))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    ProcessJob::prepare(&mut command);
    let job = ProcessJob::new().ok()?;
    let mut child = command.spawn().ok()?;
    job.assign(&child).ok()?;
    let mut stdout = child.stdout.take()?;
    let (done, output) = mpsc::channel();
    std::thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stdout.read_to_end(&mut bytes);
        let _ = done.send(bytes);
    });
    let bytes = output.recv_timeout(SHELL_TIMEOUT).ok();
    if bytes.is_none() {
        let _ = job.terminate();
    }
    // The pipe closes a moment before the process counts as exited: killing
    // the group right at end of output could turn a clean exit into SIGKILL.
    // With the output complete, `env` is exiting; wait for it, then sweep
    // whatever the profile left behind.
    let status = child.wait().ok();
    let _ = job.terminate();
    let status = status?;
    let vars = parse(&bytes?)?;
    status.success().then_some(vars)
}

/// The `NAME=value` entries after the marker.
fn parse(output: &[u8]) -> Option<Vars> {
    let start = output
        .windows(MARKER.len())
        .position(|window| window == MARKER)?
        + MARKER.len();
    let vars: Vars = output[start..]
        .split(|&byte| byte == 0)
        .filter_map(|entry| {
            let eq = entry.iter().position(|&byte| byte == b'=')?;
            let name = &entry[..eq];
            (!name.is_empty() && !SHELL_OWN.iter().any(|own| own.as_bytes() == name)).then(|| {
                (
                    OsString::from_vec(name.to_vec()),
                    OsString::from_vec(entry[eq + 1..].to_vec()),
                )
            })
        })
        .collect();
    vars.iter()
        .any(|(name, _)| name.as_bytes() == b"PATH")
        .then_some(vars)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_variables_after_what_the_profile_prints() {
        let output = b"Welcome!\nBOTLOFT_ENVIRONMENT\0PATH=/a:/b\0SHLVL=2\0NOTE=two\nlines\0";
        let vars = parse(output).expect("vars");
        assert_eq!(
            vars,
            [
                (OsString::from("PATH"), OsString::from("/a:/b")),
                (OsString::from("NOTE"), OsString::from("two\nlines")),
            ]
        );
    }

    #[test]
    fn without_the_marker_or_a_path_there_is_nothing() {
        assert!(parse(b"PATH=/a\0").is_none());
        assert!(parse(b"BOTLOFT_ENVIRONMENT\0HOME=/h\0").is_none());
    }

    #[test]
    fn the_login_shell_gives_a_path() {
        let vars = login_environment().expect("the login shell's environment");
        assert!(vars.iter().any(|(name, _)| name == "PATH"));
        assert!(!vars.iter().any(|(name, _)| name == "SHLVL"));
    }
}
