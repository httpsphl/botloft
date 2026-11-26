//! The agents' box on a Linux server (spec 42.5): each agent runs under
//! bubblewrap and sees only the system's programs, read-only, and its own
//! folders. Not the data folder, other crews, people's homes, nor anything
//! of the server's other systems. Its processes are the only ones it sees,
//! and its network is a loopback of its own: two ports there, bridged by
//! `botloftd box-bridge` to sockets outside, lead to Botloft (its MCP) and
//! to the gate, the one way to the internet (`crate::gate`).

use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

/// Inside the box: where Botloft's own program and the sockets are.
const INSIDE: &str = "/run/botloft";
/// The parts of `/etc` programs need: users, names, certificates, links to
/// the programs chosen, the libraries' cache, the time zone. Anything else
/// there (a server's other systems keep settings and keys in it) is not.
const ETC: &[&str] = &[
    "passwd",
    "group",
    "nsswitch.conf",
    "hosts",
    "host.conf",
    "gai.conf",
    "resolv.conf",
    "ssl",
    "ca-certificates",
    "ca-certificates.conf",
    "pki",
    "crypto-policies",
    "alternatives",
    "ld.so.cache",
    "ld.so.conf",
    "ld.so.conf.d",
    "localtime",
    "timezone",
    "mime.types",
    "fonts",
    "gitconfig",
    "profile",
    "profile.d",
    "bash.bashrc",
    "bashrc",
    "inputrc",
    "terminfo",
    "os-release",
    "lsb-release",
    "debian_version",
    "alpine-release",
    "locale.alias",
];
/// The system's program folders: a link where the system has a link (a
/// merged `/usr`), a read-only folder where it has a folder.
const SYSTEM: &[&str] = &[
    "/usr", "/bin", "/sbin", "/lib", "/lib32", "/lib64", "/libx32",
];

/// What every box is made with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxSetup {
    pub bwrap: PathBuf,
    /// `botloftd` itself, which bridges the ports inside.
    pub exe: PathBuf,
    /// The agents' home: their Claude login and settings.
    pub home: PathBuf,
    /// The MCP's and the gate's sockets.
    pub mcp_socket: PathBuf,
    pub gate_socket: PathBuf,
    /// Botloft's port, which the agents' `mcp.json` names.
    pub port: u16,
}

static BOX: OnceLock<(BoxSetup, Vec<Mount>)> = OnceLock::new();

/// From now on every agent runs in a box. Fails, and nothing changes, when
/// bubblewrap is not there.
pub fn use_box(setup: BoxSetup) -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err("the agents' box needs Linux".to_owned());
    }
    if !setup.bwrap.is_file() {
        return Err(format!(
            "bubblewrap is not at {}; install it (apt install bubblewrap)",
            setup.bwrap.display()
        ));
    }
    let system = SYSTEM
        .iter()
        .filter_map(|path| system_mount(path))
        .collect();
    BOX.set((setup, system))
        .map_err(|_| "the agents' box was already set".to_owned())
}

/// Whether the agents run in a box.
pub fn boxed() -> bool {
    BOX.get().is_some()
}

/// `program` with `args` in `cwd`, in a box that may also write `writable`
/// when the agents have one; as it is otherwise. The caller sets the rest
/// (environment, pipes, the agents' user) as for any command.
pub fn agent_command(
    program: &Path,
    args: &[OsString],
    cwd: &Path,
    writable: &[PathBuf],
) -> Command {
    let Some((setup, system)) = BOX.get() else {
        let mut command = Command::new(program);
        command.args(args).current_dir(cwd);
        return command;
    };
    // A program kept elsewhere is mounted by itself, and a link cannot be:
    // its real file then. One already in sight runs as it is.
    let real = (!shown(setup, system, writable, program))
        .then(|| std::fs::canonicalize(program).ok())
        .flatten();
    let program = real.as_deref().unwrap_or(program);
    let mut command = Command::new(&setup.bwrap);
    command
        .args(plan(setup, system, program, args, cwd, writable))
        .current_dir("/");
    command
}

/// Whether `path` is inside a folder the box shows anyway.
fn shown(setup: &BoxSetup, system: &[Mount], writable: &[PathBuf], path: &Path) -> bool {
    path.starts_with(&setup.home)
        || writable.iter().any(|folder| path.starts_with(folder))
        || system.iter().any(|mount| match mount {
            Mount::ReadOnly(folder) => path.starts_with(folder),
            Mount::Link { link, .. } => path.starts_with(link),
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mount {
    Link { target: OsString, link: PathBuf },
    ReadOnly(PathBuf),
}

fn system_mount(path: &str) -> Option<Mount> {
    let path = PathBuf::from(path);
    let meta = std::fs::symlink_metadata(&path).ok()?;
    if meta.file_type().is_symlink() {
        let target = std::fs::read_link(&path).ok()?;
        return Some(Mount::Link {
            target: target.into_os_string(),
            link: path,
        });
    }
    meta.is_dir().then_some(Mount::ReadOnly(path))
}

/// The gate's port inside: any free one, apart from Botloft's.
pub(crate) fn gate_port(port: u16) -> u16 {
    if port == 3128 { 3129 } else { 3128 }
}

/// Adds each part, a `&str`, a path or a string, as an argument.
macro_rules! put {
    ($plan:expr, $($part:expr),+ $(,)?) => {
        {
            $( $plan.push(std::ffi::OsStr::new(&$part).to_owned()); )+
        }
    };
}

/// bubblewrap's arguments.
pub(crate) fn plan(
    setup: &BoxSetup,
    system: &[Mount],
    program: &Path,
    args: &[OsString],
    cwd: &Path,
    writable: &[PathBuf],
) -> Vec<OsString> {
    let mut plan: Vec<OsString> = Vec::new();
    // Its own users, processes, network and shared memory. Not a host name
    // of its own: the service's ProtectHostname refuses even that one.
    put!(plan, "--unshare-all", "--die-with-parent");
    for mount in system {
        match mount {
            Mount::Link { target, link } => {
                put!(plan, "--symlink", target, link);
            }
            Mount::ReadOnly(path) => put!(plan, "--ro-bind", path, path),
        }
    }
    for name in ETC {
        let path = Path::new("/etc").join(name);
        put!(plan, "--ro-bind-try", path, path);
    }
    put!(plan, "--proc", "/proc", "--dev", "/dev");
    put!(plan, "--tmpfs", "/tmp", "--tmpfs", "/var/tmp");
    put!(plan, "--tmpfs", "/run");
    // `nproc` and the like read how many processors there are.
    let cpus = Path::new("/sys/devices/system/cpu");
    put!(plan, "--ro-bind-try", cpus, cpus);
    put!(plan, "--bind", setup.home, setup.home);
    for folder in writable {
        put!(plan, "--bind", folder, folder);
    }
    // A program kept outside the visible folders still runs. Not one inside
    // them: there it may be a link (Claude Code's installer makes one in its
    // home), and bubblewrap cannot mount over a link.
    if !shown(setup, system, writable, program) {
        put!(plan, "--ro-bind-try", program, program);
    }
    let inside = Path::new(INSIDE);
    let (exe, mcp, gate) = (
        inside.join("botloftd"),
        inside.join("mcp.sock"),
        inside.join("gate.sock"),
    );
    put!(plan, "--ro-bind", setup.exe, exe);
    put!(plan, "--bind", setup.mcp_socket, mcp);
    put!(plan, "--bind", setup.gate_socket, gate);
    let proxy = format!("http://127.0.0.1:{}", gate_port(setup.port));
    for name in ["HTTPS_PROXY", "https_proxy", "HTTP_PROXY", "http_proxy"] {
        put!(plan, "--setenv", name, proxy);
    }
    for name in ["NO_PROXY", "no_proxy"] {
        put!(plan, "--setenv", name, "127.0.0.1,localhost,::1");
    }
    // Node's own fetch follows those only when asked.
    put!(plan, "--setenv", "NODE_USE_ENV_PROXY", "1");
    put!(plan, "--chdir", cwd);
    let mcp_bridge = format!("127.0.0.1:{}={}", setup.port, mcp.display());
    let gate_bridge = format!("127.0.0.1:{}={}", gate_port(setup.port), gate.display());
    put!(plan, "--", exe, "box-bridge");
    put!(plan, mcp_bridge, gate_bridge, "--", program);
    plan.extend(args.iter().cloned());
    plan
}

/// Makes one box as an agent's, bridges and all, around `true`, as the
/// agents' user: what the service or the system refuses shows here, before
/// any agent starts.
pub fn check_box() -> io::Result<()> {
    if BOX.get().is_none() {
        return Ok(());
    }
    let program = ["/usr/bin/true", "/bin/true"]
        .into_iter()
        .map(Path::new)
        .find(|path| path.exists())
        .unwrap_or(Path::new("/bin/true"));
    let mut command = agent_command(program, &[], Path::new("/"), &[]);
    command.env_clear();
    super::agent_user::as_agent(&mut command);
    let output = command.output()?;
    if output.status.success() {
        return Ok(());
    }
    Err(io::Error::other(format!(
        "bubblewrap cannot make the agents' box: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    )))
}

#[cfg(test)]
#[path = "agent_box_tests.rs"]
mod tests;
