//! The app's own SSH tunnel to Botloft on a server (spec 42.6). The app
//! starts OpenSSH's `ssh` with the key `botloftd pair` made for this
//! computer and the server's host key pinned, forwarding one port of this
//! computer's 127.0.0.1 to Botloft's there. Nothing of the owner's own SSH
//! setup takes part, and the key is on disk only while `ssh` reads it.

#[cfg(windows)]
mod acl;
mod files;
#[cfg(windows)]
mod job;
mod ssh;

use std::collections::VecDeque;
use std::io::{BufRead as _, BufReader, Read as _, Write as _};
use std::net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStderr, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tauri::{AppHandle, Manager as _};

/// Longest wait for Botloft to answer through a new tunnel.
const OPEN_TIMEOUT: Duration = Duration::from_secs(30);
/// Lines of what `ssh` said kept for the reason it stopped.
const SAID_MAX: usize = 20;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// How to open the tunnel, from the connection key.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TunnelSpec {
    /// The server's address.
    pub host: String,
    /// sshd's port there.
    pub port: u16,
    /// The user sshd lets in only to reach Botloft.
    pub user: String,
    /// The server's host key, the only one accepted.
    pub host_key: String,
    /// This computer's private key, in OpenSSH's format.
    pub key: String,
    /// Botloft's port on the server.
    pub remote_port: u16,
    /// The port wanted on this computer; another if it is taken.
    pub local_port: u16,
}

/// The tunnel that runs, if one does.
#[derive(Default)]
pub struct Tunnels(Mutex<Option<Running>>);

struct Running {
    spec: TunnelSpec,
    child: Child,
    port: u16,
}

impl Running {
    fn alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    fn stop(mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Opens the tunnel, or finds it open, and answers the port on this
/// computer. An error starts with what went wrong (`refused`, `host_key`,
/// `unreachable`, `no_ssh`, `key_file`, `bad_spec`, `failed`), then a colon
/// and what `ssh` said.
#[tauri::command]
pub async fn tunnel_open(app: AppHandle, spec: TunnelSpec) -> Result<u16, String> {
    tauri::async_runtime::spawn_blocking(move || open(&app, spec))
        .await
        .map_err(|err| format!("failed: {err}"))?
}

/// The port of the tunnel that runs, for the island, which never opens one.
/// It runs on the main thread: while a tunnel opens and holds the lock, the
/// answer is none, and the island asks again.
#[tauri::command]
pub fn tunnel_port(app: AppHandle) -> Option<u16> {
    let tunnels = app.state::<Tunnels>();
    let mut slot = tunnels.0.try_lock().ok()?;
    let running = slot.as_mut()?;
    running.alive().then_some(running.port)
}

/// Stops the tunnel when the app goes back to this computer, after one
/// that is opening.
#[tauri::command]
pub async fn tunnel_close(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let tunnels = app.state::<Tunnels>();
        let running = tunnels.0.lock().ok().and_then(|mut slot| slot.take());
        stop(&app, running);
    })
    .await
    .map_err(|err| err.to_string())
}

/// Stops the tunnel as the app ends, on the main thread: one still opening
/// is left to the job on Windows, which ends with the app.
pub fn close_at_exit(app: &AppHandle) {
    let tunnels = app.state::<Tunnels>();
    let running = tunnels.0.try_lock().ok().and_then(|mut slot| slot.take());
    stop(app, running);
}

fn stop(app: &AppHandle, running: Option<Running>) {
    if let Some(running) = running {
        running.stop();
    }
    if let Ok(folder) = folder(app) {
        let _ = std::fs::remove_dir_all(folder);
    }
}

fn open(app: &AppHandle, spec: TunnelSpec) -> Result<u16, String> {
    files::check(&spec).map_err(|part| format!("bad_spec: {part}"))?;
    let tunnels = app.state::<Tunnels>();
    let mut slot = tunnels
        .0
        .lock()
        .map_err(|_| "failed: the tunnel's state broke".to_owned())?;
    if let Some(running) = slot.as_mut()
        && running.spec == spec
        && running.alive()
    {
        return Ok(running.port);
    }
    if let Some(running) = slot.take() {
        running.stop();
    }
    let folder = folder(app).map_err(|err| format!("failed: {err}"))?;
    let (child, port) = start(&folder, &spec)?;
    *slot = Some(Running { spec, child, port });
    Ok(port)
}

/// `<the app's local data>/tunnel`.
fn folder(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_local_data_dir()
        .map(|dir| dir.join("tunnel"))
        .map_err(|err| err.to_string())
}

/// Starts `ssh` with the files in `folder` and waits until Botloft answers
/// through it.
fn start(folder: &Path, spec: &TunnelSpec) -> Result<(Child, u16), String> {
    let failed = |err: std::io::Error| format!("failed: {err}");
    let program = ssh::program().ok_or("no_ssh: OpenSSH's ssh is not on this computer")?;
    files::prepare(folder).map_err(failed)?;
    let key = files::write_key(folder, &spec.key).map_err(failed)?;
    let known = files::write_known_hosts(folder, &spec.host_key).map_err(failed)?;
    let port = free_port(spec.local_port).map_err(failed)?;
    let started = run(&program, &ssh::args(spec, &key, &known, port), port);
    // By now `ssh` has read the key, or never will.
    let _ = std::fs::remove_file(&key);
    started.map(|child| (child, port))
}

fn run(program: &Path, args: &[std::ffi::OsString], port: u16) -> Result<Child, String> {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    let mut child = command.spawn().map_err(|err| format!("no_ssh: {err}"))?;
    #[cfg(windows)]
    job::keep(&child);
    let said = Arc::new(Mutex::new(VecDeque::new()));
    let listener = child
        .stderr
        .take()
        .map(|stderr| listen(stderr, Arc::clone(&said)));
    let deadline = Instant::now() + OPEN_TIMEOUT;
    loop {
        if matches!(child.try_wait(), Ok(Some(_))) {
            if let Some(listener) = listener {
                let _ = listener.join();
            }
            let text = said
                .lock()
                .map(|lines| lines.iter().cloned().collect::<Vec<String>>().join("\n"))
                .unwrap_or_default();
            let last = text.lines().last().unwrap_or("ssh stopped");
            return Err(format!("{}: {last}", ssh::reason(&text)));
        }
        if answers(port) {
            return Ok(child);
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err("unreachable: Botloft did not answer through the tunnel".to_owned());
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}

/// Keeps the last lines `ssh` writes; reading them also keeps it from
/// waiting on a full pipe.
fn listen(stderr: ChildStderr, said: Arc<Mutex<VecDeque<String>>>) -> JoinHandle<()> {
    std::thread::spawn(move || {
        for line in BufReader::new(stderr).lines().map_while(Result::ok) {
            if let Ok(mut lines) = said.lock() {
                if lines.len() == SAID_MAX {
                    lines.pop_front();
                }
                lines.push_back(line);
            }
        }
    })
}

/// Whether Botloft answers `/health` on this computer's `port`.
fn answers(port: u16) -> bool {
    let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    let Ok(mut stream) = TcpStream::connect_timeout(&address, Duration::from_millis(500)) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    if stream
        .write_all(b"GET /health HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")
        .is_err()
    {
        return false;
    }
    let mut answer = String::new();
    let _ = stream.take(4096).read_to_string(&mut answer);
    answer.starts_with("HTTP/1.") && answer.contains("\"status\":\"ok\"")
}

/// `wanted` when nothing listens there, else one the system gives.
fn free_port(wanted: u16) -> std::io::Result<u16> {
    if TcpListener::bind((Ipv4Addr::LOCALHOST, wanted)).is_ok() {
        return Ok(wanted);
    }
    Ok(TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?
        .local_addr()?
        .port())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_taken_port_gives_way_to_a_free_one() {
        let taken = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
        let port = taken.local_addr().expect("addr").port();
        let other = free_port(port).expect("free");
        assert_ne!(other, port);
        drop(taken);
        assert_eq!(free_port(port).expect("free"), port);
    }

    #[test]
    fn nothing_answers_on_a_closed_port() {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).expect("bind");
        let port = listener.local_addr().expect("addr").port();
        drop(listener);
        assert!(!answers(port));
    }

    /// A real tunnel, against a server `deploy/server/test-tunnel.sh` style
    /// set up: `BOTLOFT_TUNNEL_SPEC` holds the spec as JSON.
    #[test]
    #[ignore = "needs a server with sshd and Botloft"]
    fn a_real_tunnel_reaches_botloft() {
        let spec: TunnelSpec = serde_json::from_str(
            &std::env::var("BOTLOFT_TUNNEL_SPEC").expect("BOTLOFT_TUNNEL_SPEC"),
        )
        .expect("spec");
        let folder = std::env::temp_dir().join("botloft-real-tunnel");
        let (mut child, port) = start(&folder, &spec).expect("open");
        assert!(answers(port));
        assert!(!folder.join("key").exists(), "the key stayed on disk");
        let _ = child.kill();
        let _ = child.wait();
        let _ = std::fs::remove_dir_all(folder);
    }
}
