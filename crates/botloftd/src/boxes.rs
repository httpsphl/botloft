//! Sets up the agents' boxes when the settings ask for them (spec 42.5):
//! the sockets the boxes reach Botloft and the gate through, the gate's
//! port for the agents' browser, and the box itself. Asked for and not
//! possible, nothing starts rather than run the agents without it.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use tracing::info;

use crate::gate::{self, Gate};
use crate::platform;
use crate::state::Daemon;

/// Where bubblewrap is looked for.
const BWRAP: &[&str] = &["/usr/bin/bwrap", "/bin/bwrap", "/usr/local/bin/bwrap"];

/// Opens the gate and the boxes' sockets and makes every agent run boxed.
/// `reach` are what the owner opened past the gate.
pub async fn open(daemon: &Arc<Daemon>, reach: &[String]) -> Result<(), String> {
    let dir = daemon.paths.agents_scratch("box");
    std::fs::create_dir_all(&dir).map_err(|err| format!("cannot make {}: {err}", dir.display()))?;
    let gate = Gate::new(reach);
    let gate_socket = dir.join("gate.sock");
    let mcp_socket = dir.join("mcp.sock");
    let gate_listener = listen(&gate_socket)?;
    let mcp_listener = listen(&mcp_socket)?;
    // The browsers' way out goes through the same gate (spec 42.5).
    let browsers = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|err| format!("cannot open the gate for the browsers: {err}"))?;
    let browsers_port = browsers.local_addr().map_err(|err| err.to_string())?.port();
    let bwrap = BWRAP
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from(BWRAP[0]));
    let home = platform::agents_home().ok_or("the agents' user has no home")?;
    platform::use_box(platform::BoxSetup {
        bwrap,
        exe: own_program()?,
        home,
        mcp_socket,
        gate_socket,
        port: daemon.port,
    })?;
    platform::check_box().map_err(|err| err.to_string())?;
    gate::use_for_browsers(browsers_port);
    tokio::spawn(gate::serve_unix(gate_listener, Arc::clone(&gate)));
    tokio::spawn(gate::serve_tcp(browsers, gate));
    tokio::spawn(crate::server::serve_box(Arc::clone(daemon), mcp_listener));
    info!(reach = reach.len(), "each agent runs in a box of its own");
    Ok(())
}

/// A socket the agents' user may connect to: the folder is in their group.
fn listen(path: &Path) -> Result<tokio::net::UnixListener, String> {
    let _ = std::fs::remove_file(path);
    let listener = tokio::net::UnixListener::bind(path)
        .map_err(|err| format!("cannot listen on {}: {err}", path.display()))?;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o660))
        .map_err(|err| format!("cannot open {} to the agents: {err}", path.display()))?;
    Ok(listener)
}

/// This program's path. After an update replaced it while it ran, Linux
/// names the old file "(deleted)"; the new one is at the same path, and it
/// bridges the same way.
fn own_program() -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|err| format!("cannot find botloftd: {err}"))?;
    let text = exe.to_string_lossy();
    Ok(match text.strip_suffix(" (deleted)") {
        Some(path) => PathBuf::from(path),
        None => exe,
    })
}
