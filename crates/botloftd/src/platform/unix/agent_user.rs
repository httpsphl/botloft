//! The system user the agents run as, apart from the daemon's (spec 42.5).
//! The daemon keeps the owner's token, the agents' keys and the cloud and
//! phone secrets in a folder only its own user reads; an agent with a
//! shell, as another user, cannot. The daemon switches each agent process
//! to that user before it runs, with the capabilities its service gives it
//! (`CAP_SETUID`, `CAP_SETGID`, `CAP_KILL`), and the agent keeps none.

use std::ffi::{CStr, CString, c_char};
use std::io;
use std::os::unix::process::CommandExt as _;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;

/// Longest buffer tried for one account entry.
const BUFFER_MAX: usize = 1 << 20;
/// The capabilities the daemon needs to run the agents as another user:
/// `CAP_KILL` (5), `CAP_SETGID` (6) and `CAP_SETUID` (7).
const NEEDED_CAPABILITIES: u64 = (1 << 5) | (1 << 6) | (1 << 7);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentUser {
    pub name: String,
    pub uid: u32,
    pub gid: u32,
    pub home: PathBuf,
    pub shell: PathBuf,
}

static AGENT: OnceLock<AgentUser> = OnceLock::new();

/// From now on the agents run as `name`. Fails, and nothing changes, when
/// the user does not exist, is root or the daemon's own, or the daemon may
/// not switch to it.
pub fn use_agent_user(name: &str) -> Result<(), String> {
    let user = lookup(name).ok_or_else(|| format!("there is no user called {name}"))?;
    // SAFETY: no arguments, never fails.
    let (uid, euid) = unsafe { (libc::getuid(), libc::geteuid()) };
    if user.uid == 0 {
        return Err(format!("{name} is root: the agents must not run as root"));
    }
    if user.uid == uid {
        return Err(format!("{name} is the daemon's own user"));
    }
    if euid != 0 && !can_switch(&std::fs::read_to_string("/proc/self/status").unwrap_or_default()) {
        return Err(
            "the daemon may not switch users: its service needs CAP_SETUID, CAP_SETGID and \
             CAP_KILL (spec 42.5)"
                .to_owned(),
        );
    }
    AGENT
        .set(user)
        .map_err(|_| "the agents' user was already set".to_owned())
}

/// The agents run as a user of their own.
pub fn agents_apart() -> bool {
    AGENT.get().is_some()
}

/// The login shell of the agents' user, when they have one.
pub(super) fn agent_shell() -> Option<PathBuf> {
    AGENT.get().map(|user| user.shell.clone())
}

/// The home folder of whoever runs the agents: where their Claude Code
/// keeps its login, skills and plugins.
pub fn agents_home() -> Option<PathBuf> {
    AGENT
        .get()
        .map(|user| user.home.clone())
        .or_else(dirs::home_dir)
}

/// Makes `command` run as the agents' user, when there is one: its user and
/// group, its home and shell, and none of the daemon's capabilities, nor a
/// way to gain any (`no_new_privs`).
pub fn as_agent(command: &mut Command) {
    if let Some(user) = AGENT.get() {
        switch(command, user);
    }
}

fn switch(command: &mut Command, user: &AgentUser) {
    command
        .uid(user.uid)
        .gid(user.gid)
        .env("HOME", &user.home)
        .env("USER", &user.name)
        .env("LOGNAME", &user.name)
        .env("SHELL", &user.shell);
    // SAFETY: the hook only calls `prctl`, which is async-signal-safe, as
    // the time between fork and exec asks.
    unsafe {
        command.pre_exec(drop_capabilities);
    }
}

/// Runs in the child after the switch, before `exec`.
fn drop_capabilities() -> io::Result<()> {
    #[cfg(target_os = "linux")]
    {
        // The service's ambient capabilities would pass to the agent with
        // `exec`: they go first.
        // SAFETY: plain `prctl` calls with constant arguments.
        unsafe {
            libc::prctl(
                libc::PR_CAP_AMBIENT,
                libc::PR_CAP_AMBIENT_CLEAR_ALL as libc::c_ulong,
                0,
                0,
                0,
            );
            if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0 {
                return Err(io::Error::last_os_error());
            }
        }
    }
    Ok(())
}

/// Whether the effective capabilities in `/proc/self/status` (`CapEff`)
/// hold the three the switch needs.
fn can_switch(status: &str) -> bool {
    status
        .lines()
        .find_map(|line| line.strip_prefix("CapEff:"))
        .and_then(|hex| u64::from_str_radix(hex.trim(), 16).ok())
        .is_some_and(|caps| caps & NEEDED_CAPABILITIES == NEEDED_CAPABILITIES)
}

/// The account called `name` (`getpwnam_r`).
fn lookup(name: &str) -> Option<AgentUser> {
    let wanted = CString::new(name).ok()?;
    let mut buf = vec![0u8; 4096];
    // SAFETY: plain data; `getpwnam_r` fills it in.
    let mut entry: libc::passwd = unsafe { std::mem::zeroed() };
    let mut found: *mut libc::passwd = std::ptr::null_mut();
    loop {
        // SAFETY: `wanted`, `entry`, `buf` and `found` outlive the call, and
        // `buf.len()` is the size of `buf`.
        let code = unsafe {
            libc::getpwnam_r(
                wanted.as_ptr(),
                &raw mut entry,
                buf.as_mut_ptr().cast(),
                buf.len(),
                &raw mut found,
            )
        };
        if code == libc::ERANGE && buf.len() < BUFFER_MAX {
            buf.resize(buf.len() * 2, 0);
            continue;
        }
        if code != 0 || found.is_null() {
            return None;
        }
        break;
    }
    let shell = text(entry.pw_shell);
    Some(AgentUser {
        name: name.to_owned(),
        uid: entry.pw_uid,
        gid: entry.pw_gid,
        home: PathBuf::from(text(entry.pw_dir)),
        shell: PathBuf::from(if shell.is_empty() {
            "/bin/sh".to_owned()
        } else {
            shell
        }),
    })
}

/// A string of the entry, which points into the buffer still alive.
fn text(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }
    // SAFETY: a non-null field of a filled entry is a NUL-terminated string.
    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_switch_needs_all_three_capabilities() {
        // CAP_KILL, CAP_SETGID and CAP_SETUID: bits 5, 6 and 7.
        assert!(can_switch("Name:\tbotloftd\nCapEff:\t00000000000000e0\n"));
        assert!(can_switch("CapEff:\t000001ffffffffff\n"));
        assert!(!can_switch("CapEff:\t00000000000000c0\n"));
        assert!(!can_switch("CapEff:\t0000000000000000\n"));
        assert!(!can_switch("Name:\tbotloftd\n"));
    }

    #[test]
    fn a_user_that_is_not_there_is_refused() {
        assert_eq!(
            use_agent_user("botloft-no-such-user").map_err(|err| err.contains("no user")),
            Err(true)
        );
        assert!(lookup("root").is_some_and(|root| root.uid == 0));
    }

    /// Only as root (a container): `nobody` cannot read a file only root
    /// may, and the command runs as `nobody`.
    #[test]
    fn a_command_as_another_user_cannot_read_the_daemons_secrets() {
        // SAFETY: no arguments, never fails.
        if unsafe { libc::geteuid() } != 0 {
            return;
        }
        let Some(nobody) = lookup("nobody") else {
            return;
        };
        let dir = tempfile::tempdir().expect("dir");
        let secret = dir.path().join("owner.token");
        std::fs::write(&secret, "s3cret").expect("secret");
        std::fs::set_permissions(&secret, std::os::unix::fs::PermissionsExt::from_mode(0o600))
            .expect("mode");
        // Not through the daemon-wide user: the other tests run as they are.
        let mut command = Command::new("sh");
        command
            .args(["-c", "id -u; cat \"$1\" 2>/dev/null || echo refused", "sh"])
            .arg(&secret)
            .current_dir("/");
        switch(&mut command, &nobody);
        let output = command.output().expect("run");
        let text = String::from_utf8_lossy(&output.stdout);
        assert_eq!(
            text.lines().next(),
            Some(nobody.uid.to_string().as_str()),
            "{text}"
        );
        assert!(
            text.contains("refused") && !text.contains("s3cret"),
            "{text}"
        );
    }
}
