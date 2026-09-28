//! Finding, checking and starting the local daemon (spec 15.2).
//!
//! The app resolves the daemon's data folder the way the daemon does
//! (spec 5): `BOTLOFT_HOME`, else `%LOCALAPPDATA%\Botloft`. The port comes
//! from `config.toml` in that folder, so a dev daemon with its own
//! `BOTLOFT_HOME` is found without extra setup.

use std::io::{Read as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

const HOME_ENV: &str = "BOTLOFT_HOME";
const DEFAULT_PORT: u16 = 45710;
const CONNECT_TIMEOUT: Duration = Duration::from_millis(500);
const READ_TIMEOUT: Duration = Duration::from_secs(2);
/// How long a freshly started daemon may take to answer `/health`.
const START_TIMEOUT: Duration = Duration::from_secs(15);

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

/// Starts `botloftd.exe serve` from the app's folder, detached so it keeps
/// running after the app closes (spec 1), and waits for it to answer.
pub fn start(endpoint: &Endpoint) -> Result<DaemonStatus, String> {
    if let running @ DaemonStatus::Running { .. } = status(endpoint) {
        return Ok(running);
    }
    let binary = daemon_binary()?;
    spawn_detached(&binary).map_err(|err| format!("cannot start {}: {err}", binary.display()))?;
    let started = Instant::now();
    loop {
        let current = status(endpoint);
        if matches!(current, DaemonStatus::Running { .. }) || started.elapsed() > START_TIMEOUT {
            return Ok(current);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// `botloftd.exe` next to the app: the installer puts it there (M5), and
/// in dev both land in `target\debug`.
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

fn spawn_detached(binary: &Path) -> std::io::Result<()> {
    let mut command = Command::new(binary);
    command
        .arg("serve")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        const CREATE_BREAKAWAY_FROM_JOB: u32 = 0x0100_0000;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let flags = CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP;
        // Leave the app's job, if it runs in one that would kill children
        // on close. Jobs that forbid breaking away refuse the flag.
        command.creation_flags(flags | CREATE_BREAKAWAY_FROM_JOB);
        if command.spawn().is_ok() {
            return Ok(());
        }
        command.creation_flags(flags);
    }
    command.spawn().map(drop)
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
    use super::*;

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
    fn a_closed_port_reads_as_stopped() {
        // Bind and drop to find a port nothing listens on.
        let listener = std::net::TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
        let port = listener.local_addr().expect("addr").port();
        drop(listener);
        assert!(matches!(health(port), Probe::Closed));
    }

    #[test]
    fn a_port_that_is_not_botloft_reads_as_foreign() {
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
