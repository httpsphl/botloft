//! The values of a connected tool's headers and environment variables
//! (spec 25.2). They live in `secrets\mcp\<id>.json`, readable only by the
//! current user, and reach a bot only through its environment: never the
//! database, `mcp.json` or the log.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io;
use std::path::{Path, PathBuf};

use botloft_core::ids::McpServerId;
use botloft_core::protocol::McpServer;
use serde::{Deserialize, Serialize};

use crate::platform;
use crate::workspace::connected_var;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpSecrets {
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

fn folder(secrets_dir: &Path) -> PathBuf {
    secrets_dir.join("mcp")
}

fn file(secrets_dir: &Path, id: &McpServerId) -> PathBuf {
    folder(secrets_dir).join(format!("{id}.json"))
}

/// What is stored for `id`; nothing stored reads as no secrets.
pub fn load(secrets_dir: &Path, id: &McpServerId) -> io::Result<McpSecrets> {
    match std::fs::read(file(secrets_dir, id)) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(io::Error::other),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(McpSecrets::default()),
        Err(err) => Err(err),
    }
}

pub fn save(secrets_dir: &Path, id: &McpServerId, secrets: &McpSecrets) -> io::Result<()> {
    let folder = folder(secrets_dir);
    std::fs::create_dir_all(&folder)?;
    platform::restrict_to_current_user(&folder)?;
    let path = file(secrets_dir, id);
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec(secrets).map_err(io::Error::other)?)?;
    platform::restrict_to_current_user(&tmp)?;
    std::fs::rename(&tmp, &path)
}

pub fn remove(secrets_dir: &Path, id: &McpServerId) -> io::Result<()> {
    match std::fs::remove_file(file(secrets_dir, id)) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
        _ => Ok(()),
    }
}

/// The environment variables a bot that uses `servers` starts with: the
/// value of each header and variable, under the name `mcp.json` expands
/// (spec 25.2).
pub fn environment(
    secrets_dir: &Path,
    servers: &[McpServer],
) -> io::Result<Vec<(OsString, OsString)>> {
    let mut vars = Vec::new();
    for server in servers {
        let secrets = load(secrets_dir, &server.id)?;
        for (kind, names, values) in [
            ('H', &server.header_names, &secrets.headers),
            ('E', &server.env_names, &secrets.env),
        ] {
            for (index, name) in names.iter().enumerate() {
                let value = values.get(name).cloned().unwrap_or_default();
                vars.push((
                    OsString::from(connected_var(&server.id, kind, index)),
                    OsString::from(value),
                ));
            }
        }
    }
    Ok(vars)
}
