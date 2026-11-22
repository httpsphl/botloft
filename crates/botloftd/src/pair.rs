//! `botloftd pair` (spec 42.6): pairs a new device with the daemon running
//! on this computer, as the owner, and prints the connection key the app on
//! the other computer takes.

use anyhow::Context;
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use botloft_core::protocol::{DevicePaired, method};
use serde::Serialize;
use serde_json::json;

use crate::local::Owner;

/// What a connection key starts with: the format's version is in it.
pub const KEY_PREFIX: &str = "botloft1.";
/// The port on the other computer the suggested tunnel listens on. Not the
/// daemon's own, which an installed Botloft there already uses.
pub const TUNNEL_PORT: u16 = 45720;
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
}

/// The connection key of a device paired with the daemon on `port` of the
/// computer called `name`.
pub fn key(name: &str, port: u16, token: &str) -> String {
    let key = Key {
        name,
        port,
        tunnel: TUNNEL_PORT,
        token,
    };
    let json = serde_json::to_vec(&key).unwrap_or_default();
    format!("{KEY_PREFIX}{}", URL_SAFE_NO_PAD.encode(json))
}

/// What `botloftd pair` prints: the key and how the app gets here.
pub fn instructions(device: &str, port: u16, key: &str) -> String {
    format!(
        "Paired \"{device}\". In the Botloft app, choose \"Connect to a server\" and paste this key:\n\
         \n  {key}\n\n\
         The app reaches this computer through an SSH tunnel. On the computer with the app, run:\n\
         \n  ssh -N -L {TUNNEL_PORT}:127.0.0.1:{port} <user>@<this server's address>\n\n\
         The key works until you remove \"{device}\" from the devices in the app's settings.\n"
    )
}

/// Pairs a device called `device` with the daemon on 127.0.0.1:`port`,
/// signing in with the owner's token.
pub async fn pair(port: u16, owner_token: &str, device: &str) -> anyhow::Result<DevicePaired> {
    let mut owner = Owner::connect(port, owner_token, "botloftd pair").await?;
    let paired = owner
        .call(method::DEVICES_PAIR, json!({ "name": device }))
        .await?;
    owner.close().await;
    serde_json::from_value(paired).context("Botloft answered something unexpected")
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn a_key_carries_the_name_the_ports_and_the_token() {
        let key = key("vps-1", 45710, "abc");
        let body = key.strip_prefix(KEY_PREFIX).expect("prefix");
        assert!(!body.contains(['+', '/', '=']), "base64url without padding");
        let json: Value =
            serde_json::from_slice(&URL_SAFE_NO_PAD.decode(body).expect("base64")).expect("json");
        assert_eq!(
            json,
            json!({ "name": "vps-1", "port": 45710, "tunnel": TUNNEL_PORT, "token": "abc" })
        );
    }

    #[test]
    fn the_instructions_hold_the_key_and_the_tunnel() {
        let text = instructions("Laptop", 45710, "botloft1.xyz");
        assert!(text.contains("\n  botloft1.xyz\n"));
        assert!(text.contains("ssh -N -L 45720:127.0.0.1:45710 "));
        assert!(text.contains("\"Laptop\""));
    }
}
