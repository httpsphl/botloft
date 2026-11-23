//! `botloftd pair` (spec 42.6): pairs a new device with the daemon running
//! on this computer, as the owner, and prints the connection key the app on
//! the other computer takes. Where `botloftd tunnel setup` prepared sshd,
//! the key also carries what the app needs to open the tunnel by itself.

use std::path::Path;

use anyhow::{Context, ensure};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use botloft_core::protocol::{DevicePaired, DevicesPairParams, method};
use serde::Serialize;

use crate::config::Config;
use crate::local::Owner;
use crate::tunnel::{self, Keypair};
use crate::{platform, secrets};

/// What a connection key starts with: the format's version is in it.
pub const KEY_PREFIX: &str = "botloft1.";
/// The port on the other computer the tunnel listens on. Not the daemon's
/// own, which an installed Botloft there already uses.
pub const TUNNEL_PORT: u16 = 45720;

/// What `botloftd pair` was asked.
#[derive(Debug, Clone)]
pub struct Options {
    /// The device's name, as the list of devices shows it.
    pub name: String,
    /// The address the app reaches this server at, instead of the one found.
    pub host: Option<String>,
    /// sshd's port, instead of the one sshd says.
    pub ssh_port: Option<u16>,
}

/// What a connection key holds, as JSON in base64url after [`KEY_PREFIX`].
#[derive(Debug, Serialize)]
struct Key<'a> {
    /// This computer's name, for the app to show.
    name: &'a str,
    /// The daemon's port here, where the tunnel ends.
    port: u16,
    /// The port on the other computer where the tunnel starts.
    tunnel: u16,
    token: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssh: Option<&'a Ssh>,
}

/// How the app opens the tunnel by itself (spec 42.6).
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ssh {
    /// The user sshd lets in, only to reach Botloft.
    pub user: String,
    /// The address the app reaches this server at.
    pub host: String,
    /// sshd's port.
    pub port: u16,
    /// This server's host key, which the app pins.
    pub host_key: String,
    /// The device's private key, in OpenSSH's format.
    pub key: String,
}

impl std::fmt::Debug for Ssh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ssh")
            .field("user", &self.user)
            .field("host", &self.host)
            .field("port", &self.port)
            .field("key", &"<redacted>")
            .finish_non_exhaustive()
    }
}

/// The connection key of a device paired with the daemon on `port` of the
/// computer called `name`.
pub fn key(name: &str, port: u16, token: &str, ssh: Option<&Ssh>) -> String {
    let key = Key {
        name,
        port,
        tunnel: TUNNEL_PORT,
        token,
        ssh,
    };
    let json = serde_json::to_vec(&key).unwrap_or_default();
    format!("{KEY_PREFIX}{}", URL_SAFE_NO_PAD.encode(json))
}

/// What `botloftd pair` prints: the key and how the app gets here.
pub fn instructions(device: &str, port: u16, key: &str, ssh: Option<&Ssh>) -> String {
    let how = match ssh {
        Some(ssh) => format!(
            "The app opens its own SSH connection to {host}, port {ssh_port}, as {user},\n\
             which can only reach Botloft. If the app reaches this server by another\n\
             address, run this again with --host <address>.\n",
            host = ssh.host,
            ssh_port = ssh.port,
            user = ssh.user,
        ),
        None => format!(
            "The app reaches this computer through an SSH tunnel. On the computer with the app, run:\n\
             \n  ssh -N -L {TUNNEL_PORT}:127.0.0.1:{port} <user>@<this server's address>\n\n\
             On a server set up by install.sh, `sudo botloftd tunnel setup` lets the app open it by itself.\n"
        ),
    };
    format!(
        "Paired \"{device}\". In the Botloft app, choose \"Connect to a server\" and paste this key:\n\
         \n  {key}\n\n\
         {how}\n\
         The key works until you remove \"{device}\" from the devices in the app's settings.\n\
         Keep it to yourself: it opens Botloft.\n"
    )
}

/// `botloftd pair`: asks the daemon that runs here, as the owner, for a new
/// device, and prints its key.
pub fn run(home: &Path, options: &Options) -> anyhow::Result<()> {
    let config = Config::load(&home.join("config.toml"), false)?;
    let token = secrets::read_owner_token(&home.join("secrets")).with_context(|| {
        format!(
            "cannot read Botloft's token in {}; run this as the user Botloft runs as, or with sudo",
            home.display()
        )
    })?;
    let computer = platform::computer_name().unwrap_or_else(|| "Botloft".to_owned());
    let own = own_tunnel(options, &computer)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let params = DevicesPairParams {
        name: options.name.clone(),
        ssh_key: own.as_ref().map(|(_, public)| public.clone()),
    };
    let paired = runtime.block_on(pair(config.port, &token, &params))?;
    let ssh = own.map(|(ssh, _)| ssh);
    let key = key(&computer, config.port, &paired.token, ssh.as_ref());
    print!(
        "{}",
        instructions(&paired.device.name, config.port, &key, ssh.as_ref())
    );
    Ok(())
}

/// What the app needs to open the tunnel by itself, with the device's
/// public key, when `tunnel setup` prepared sshd here.
fn own_tunnel(options: &Options, computer: &str) -> anyhow::Result<Option<(Ssh, String)>> {
    if let Some(host) = &options.host {
        ensure!(
            tunnel::valid_host(host),
            "--host takes a name or an IP address"
        );
    }
    if !tunnel::ready() {
        return Ok(None);
    }
    let Some(host_key) = tunnel::host_key() else {
        eprintln!("sshd has no host key in /etc/ssh, so the app needs a tunnel of yours.");
        return Ok(None);
    };
    let keypair = Keypair::generate().context("cannot make the device's SSH key")?;
    let host = options
        .host
        .clone()
        .or_else(|| tunnel::address().map(|ip| ip.to_string()))
        .unwrap_or_else(|| computer.to_owned());
    let ssh = Ssh {
        user: tunnel::USER.to_owned(),
        host,
        port: options.ssh_port.unwrap_or_else(tunnel::ssh_port),
        host_key,
        key: keypair.private,
    };
    Ok(Some((ssh, keypair.public)))
}

/// Pairs a device with the daemon on 127.0.0.1:`port`, signing in with the
/// owner's token.
pub async fn pair(
    port: u16,
    owner_token: &str,
    params: &DevicesPairParams,
) -> anyhow::Result<DevicePaired> {
    let mut owner = Owner::connect(port, owner_token, "botloftd pair").await?;
    let paired = owner
        .call(method::DEVICES_PAIR, serde_json::to_value(params)?)
        .await?;
    owner.close().await;
    serde_json::from_value(paired).context("Botloft answered something unexpected")
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn decoded(key: &str) -> Value {
        let body = key.strip_prefix(KEY_PREFIX).expect("prefix");
        assert!(!body.contains(['+', '/', '=']), "base64url without padding");
        serde_json::from_slice(&URL_SAFE_NO_PAD.decode(body).expect("base64")).expect("json")
    }

    fn ssh() -> Ssh {
        Ssh {
            user: "botloft-tunnel".to_owned(),
            host: "203.0.113.10".to_owned(),
            port: 2222,
            host_key: "ssh-ed25519 AAAAhost".to_owned(),
            key: "-----BEGIN OPENSSH PRIVATE KEY-----\n...".to_owned(),
        }
    }

    #[test]
    fn a_key_carries_the_name_the_ports_and_the_token() {
        assert_eq!(
            decoded(&key("vps-1", 45710, "abc", None)),
            json!({ "name": "vps-1", "port": 45710, "tunnel": TUNNEL_PORT, "token": "abc" })
        );
    }

    #[test]
    fn a_key_for_the_apps_own_tunnel_carries_how_to_open_it() {
        let json = decoded(&key("vps-1", 45710, "abc", Some(&ssh())));
        assert_eq!(
            json["ssh"],
            json!({
                "user": "botloft-tunnel",
                "host": "203.0.113.10",
                "port": 2222,
                "hostKey": "ssh-ed25519 AAAAhost",
                "key": "-----BEGIN OPENSSH PRIVATE KEY-----\n...",
            })
        );
        assert!(!format!("{:?}", ssh()).contains("PRIVATE"));
    }

    #[test]
    fn the_instructions_hold_the_key_and_the_tunnel() {
        let text = instructions("Laptop", 45710, "botloft1.xyz", None);
        assert!(text.contains("\n  botloft1.xyz\n"));
        assert!(text.contains("ssh -N -L 45720:127.0.0.1:45710 "));
        assert!(text.contains("\"Laptop\""));
        let own = instructions("Laptop", 45710, "botloft1.xyz", Some(&ssh()));
        assert!(own.contains("203.0.113.10, port 2222, as botloft-tunnel"));
        assert!(!own.contains("ssh -N"));
    }
}
