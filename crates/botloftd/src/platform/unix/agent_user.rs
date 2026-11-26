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
    /// Every group it is in, `gid` among them.
    pub groups: Vec<u32>,
}

static AGENT: OnceLock<AgentUser> = OnceLock::new();

/// From now on the agents run as `name`. Fails, and nothing changes, when
/// the user does not exist, is root or the daemon's own, or the daemon may
/// not switch to it.
pub fn use_agent_user(name: &str) -> Result<(), String> {
    AGENT
        .set(switchable(name)?)
        .map_err(|_| "the agents' user was already set".to_owned())
}

/// The user called `name`, if the daemon may run programs as it: it exists,
/// is not root nor the daemon's own, and the daemon can switch.
pub(super) fn switchable(name: &str) -> Result<AgentUser, String> {
    let user = lookup(name).ok_or_else(|| format!("there is no user called {name}"))?;
    // SAFETY: no arguments, never fails.
    let (uid, euid) = unsafe { (libc::getuid(), libc::geteuid()) };
    if user.uid == 0 {
        return Err(format!(
            "{name} is root: nothing Botloft starts runs as root"
        ));
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
    Ok(user)
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

pub(super) fn switch(command: &mut Command, user: &AgentUser) {
    command
        .env("HOME", &user.home)
        .env("USER", &user.name)
        .env("LOGNAME", &user.name)
        .env("SHELL", &user.shell);
    // With its groups too: the browsers' user reaches its profiles, which
    // the daemon makes, through the daemon's group. `Command::uid` alone
    // leaves none (`setgroups(0)`) and `Command::groups` is not stable.
    let (uid, gid, groups) = (user.uid, user.gid, user.groups.clone());
    // SAFETY: the hook only makes plain system calls (`setgroups`,
    // `setgid`, `setuid`, `prctl`) on data made before the fork, as the
    // time between fork and exec asks.
    unsafe {
        command.pre_exec(move || {
            take_on(uid, gid, &groups)?;
            drop_capabilities()
        });
    }
}

/// Runs in the child, before `exec`: the user's groups, then its group,
/// then the user, in the only order the capabilities allow.
fn take_on(uid: u32, gid: u32, groups: &[u32]) -> io::Result<()> {
    // A `size_t` on Linux, an `int` on macOS.
    #[allow(clippy::useless_conversion)]
    let count = groups
        .len()
        .try_into()
        .map_err(|_| io::ErrorKind::InvalidInput)?;
    // SAFETY: `groups` holds `count` ids and outlives the call.
    let set = unsafe { libc::setgroups(count, groups.as_ptr()) };
    // SAFETY: plain system calls with ids as arguments.
    if set != 0 || unsafe { libc::setgid(gid) } != 0 || unsafe { libc::setuid(uid) } != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// The groups `name` is in, its own `gid` among them (`getgrouplist`).
#[cfg(target_os = "linux")]
fn groups_of(name: &CStr, gid: u32) -> Vec<u32> {
    let mut room: libc::c_int = 32;
    while room <= 1 << 16 {
        let mut found = room;
        let mut list: Vec<libc::gid_t> = vec![gid; usize::try_from(room).unwrap_or(32)];
        // SAFETY: `list` has room for `found` ids, and both outlive the call.
        let code =
            unsafe { libc::getgrouplist(name.as_ptr(), gid, list.as_mut_ptr(), &raw mut found) };
        if code >= 0 {
            list.truncate(usize::try_from(found).unwrap_or_default());
            return list;
        }
        room = found.max(room * 2);
    }
    vec![gid]
}

/// Elsewhere, its own group only, as before.
#[cfg(not(target_os = "linux"))]
fn groups_of(_name: &CStr, gid: u32) -> Vec<u32> {
    vec![gid]
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
        groups: groups_of(&wanted, entry.pw_gid),
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
#[path = "agent_user_tests.rs"]
mod tests;
