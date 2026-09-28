//! Finding, checking and installing the local daemon (spec 15.2).
//!
//! The app resolves the daemon's data folder the way the daemon does
//! (spec 5): `BOTLOFT_HOME`, else `%LOCALAPPDATA%\Botloft`. The port comes
//! from `config.toml` in that folder, so a dev daemon with its own
//! `BOTLOFT_HOME` is found without extra setup.
//!
//! The app ships `botloftd.exe` next to itself (the sidecar) and installs
//! it with `botloftd service install`, which copies it into the data
//! folder and runs it as a scheduled task (spec 14).

use std::io::{Read as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde::{Deserialize, Serialize};

const HOME_ENV: &str = "BOTLOFT_HOME";
const DEFAULT_PORT: u16 = 45710;
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const READ_TIMEOUT: Duration = Duration::from_secs(2);
/// The daemon this app ships has the app's version (one Cargo workspace).
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where the daemon lives.
#[derive(Debug, Clone)]
pub struct Endpoint {
    pub home: PathBuf,
    pub port: u16,
}

/// What answers on the daemon's port.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "camelCase")]
pub enum DaemonStatus {
    #[serde(rename_all = "camelCase")]
    Running {
        port: u16,
        version: String,
        protocol: u32,
        /// Older than the daemon this app ships: installing replaces it.
        outdated: bool,
    },
    /// Nothing listens on the port.
    #[serde(rename_all = "camelCase")]
    Stopped { port: u16, home: String },
    /// Something that is not a Botloft daemon holds the port.
    #[serde(rename_all = "camelCase")]
    Foreign { port: u16 },
}

#[derive(Deserialize)]
struct Health {
    status: String,
    version: String,
    protocol: u32,
}

#[derive(Deserialize)]
struct Config {
    port: Option<u16>,
}

pub fn endpoint() -> Result<Endpoint, String> {
    let home = match std::env::var_os(HOME_ENV).filter(|value| !value.is_empty()) {
        Some(home) => std::path::absolute(home).map_err(|err| err.to_string())?,
        None => dirs::data_local_dir()
            .map(|dir| dir.join("Botloft"))
            .ok_or("cannot find the local application data folder")?,
    };
    let port = read_port(&home.join("config.toml"))?;
    Ok(Endpoint { home, port })
}

fn read_port(config: &Path) -> Result<u16, String> {
    let text = match std::fs::read_to_string(config) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(DEFAULT_PORT),
        Err(err) => return Err(format!("cannot read {}: {err}", config.display())),
    };
    let parsed: Config =
        toml::from_str(&text).map_err(|err| format!("invalid {}: {err}", config.display()))?;
    Ok(parsed.port.unwrap_or(DEFAULT_PORT))
}

pub fn status(endpoint: &Endpoint) -> DaemonStatus {
    match health(endpoint.port) {
        Probe::Healthy(health) => DaemonStatus::Running {
            port: endpoint.port,
            outdated: is_older(&health.version, APP_VERSION),
            version: health.version,
            protocol: health.protocol,
        },
        Probe::Closed => DaemonStatus::Stopped {
            port: endpoint.port,
            home: endpoint.home.display().to_string(),
        },
        Probe::Foreign => DaemonStatus::Foreign {
            port: endpoint.port,
        },
    }
}

enum Probe {
    Healthy(Health),
    Closed,
    Foreign,
}

fn health(port: u16) -> Probe {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, CONNECT_TIMEOUT) else {
        return Probe::Closed;
    };
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let _ = stream.set_write_timeout(Some(READ_TIMEOUT));
    let request = "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
    let mut response = Vec::new();
    if stream.write_all(request.as_bytes()).is_err() || stream.read_to_end(&mut response).is_err() {
        return Probe::Foreign;
    }
    let response = String::from_utf8_lossy(&response);
    let Some((head, body)) = response.split_once("\r\n\r\n") else {
        return Probe::Foreign;
    };
    if !head.starts_with("HTTP/1.1 200") {
        return Probe::Foreign;
    }
    match serde_json::from_str::<Health>(body) {
        Ok(health) if health.status == "ok" => Probe::Healthy(health),
        _ => Probe::Foreign,
    }
}

/// Whether version `a` comes before `b` (`major.minor.patch`; a
/// pre-release suffix is ignored). Unreadable versions are never older.
fn is_older(a: &str, b: &str) -> bool {
    fn parse(version: &str) -> Option<(u64, u64, u64)> {
        let core = version.split(['-', '+']).next()?;
        let mut parts = core.split('.').map(|part| part.parse::<u64>().ok());
        Some((parts.next()??, parts.next()??, parts.next()??))
    }
    matches!((parse(a), parse(b)), (Some(a), Some(b)) if a < b)
}

/// Installs the daemon this app ships and starts it: `botloftd service
/// install` copies it into the data folder, registers the scheduled task
/// and waits for the daemon to answer (spec 14).
pub fn install(endpoint: &Endpoint) -> Result<DaemonStatus, String> {
    run_service(endpoint, "install")?;
    Ok(status(endpoint))
}

/// Stops the daemon and starts it again from its scheduled task.
pub fn restart(endpoint: &Endpoint) -> Result<DaemonStatus, String> {
    run_service(endpoint, "restart")?;
    Ok(status(endpoint))
}

/// Runs `botloftd --home <home> service <command>` without a window and
/// returns its error message if it fails.
fn run_service(endpoint: &Endpoint, command: &str) -> Result<(), String> {
    let binary = daemon_binary()?;
    let mut process = Command::new(&binary);
    process
        .arg("--home")
        .arg(&endpoint.home)
        .args(["service", command])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        process.creation_flags(CREATE_NO_WINDOW);
    }
    let output = process
        .output()
        .map_err(|err| format!("cannot run {}: {err}", binary.display()))?;
    if output.status.success() {
        return Ok(());
    }
    let message = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    Err(if message.is_empty() {
        format!("botloftd service {command} failed ({})", output.status)
    } else {
        message
    })
}

/// `botloftd.exe` next to the app: the installer puts it there, and in dev
/// both land in `target\debug`.
fn daemon_binary() -> Result<PathBuf, String> {
    let app = std::env::current_exe().map_err(|err| err.to_string())?;
    let binary = app.with_file_name(if cfg!(windows) {
        "botloftd.exe"
    } else {
        "botloftd"
    });
    if binary.is_file() {
        Ok(binary)
    } else {
        Err(format!(
            "{} was not found. Build it with `cargo build -p botloftd`.",
            binary.display()
        ))
    }
}

/// The owner token the daemon wrote on its first start (spec 13).
pub fn owner_token(endpoint: &Endpoint) -> Result<String, String> {
    let path = endpoint.home.join("secrets").join("owner.token");
    let text = std::fs::read_to_string(&path).map_err(|err| match err.kind() {
        std::io::ErrorKind::NotFound => format!(
            "{} does not exist yet; the daemon creates it on its first start",
            path.display()
        ),
        _ => format!("cannot read {}: {err}", path.display()),
    })?;
    let token = text.trim();
    if token.len() == 64 && token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(token.to_owned())
    } else {
        Err(format!("{} is not a valid token", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    /// The port tests pick free ports; run one at a time so one does not
    /// take the port the other just freed.
    static PORTS: Mutex<()> = Mutex::new(());

    #[test]
    fn port_defaults_and_reads_the_config() {
        let dir = std::env::temp_dir().join(format!("botloft-app-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("dir");
        let config = dir.join("config.toml");
        assert_eq!(read_port(&config), Ok(DEFAULT_PORT));
        std::fs::write(&config, "log_level = \"debug\"\n").expect("write");
        assert_eq!(read_port(&config), Ok(DEFAULT_PORT));
        std::fs::write(&config, "port = 45799\n[courier]\nmax_attempts = 3\n").expect("write");
        assert_eq!(read_port(&config), Ok(45799));
        std::fs::write(&config, "port = \"x\"").expect("write");
        assert!(read_port(&config).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn versions_compare_by_number() {
        assert!(is_older("0.1.0", "0.2.0"));
        assert!(is_older("0.9.3", "0.10.0"));
        assert!(is_older("1.2.3-beta.1", "1.2.4"));
        assert!(!is_older("0.2.0", "0.2.0"));
        assert!(!is_older("0.3.0", "0.2.9"));
        assert!(!is_older("dev", "0.2.0"));
        assert!(!is_older("0.1", "0.2.0"));
    }

    #[test]
    fn a_closed_port_reads_as_stopped() {
        let _ports = PORTS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        // Bind and drop to find a port nothing listens on.
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
        let port = listener.local_addr().expect("addr").port();
        drop(listener);
        assert!(matches!(health(port), Probe::Closed));
    }

    #[test]
    fn a_port_that_is_not_botloft_reads_as_foreign() {
        let _ports = PORTS.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().expect("accept");
            let mut buffer = [0u8; 512];
            let _ = socket.read(&mut buffer);
            let _ = socket.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
        });
        assert!(matches!(health(port), Probe::Foreign));
        server.join().expect("server");
    }
}
