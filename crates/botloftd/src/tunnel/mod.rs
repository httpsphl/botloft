//! The app's own tunnel to a Linux server (spec 42.6). sshd lets the user
//! `botloft-tunnel` in with the key of a paired device, and only to reach
//! Botloft on 127.0.0.1: `botloftd pair` makes the key, `botloftd tunnel
//! keys` is what sshd asks for the keys, and `botloftd tunnel setup`
//! prepares the user and sshd.

mod keypair;
mod setup;
mod sshd;

use std::path::Path;

use anyhow::Context;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use botloft_core::validate::ValidationError;
use botloft_store::Store;

use crate::config::Config;

pub use self::keypair::{Keypair, address, host_key};
pub use self::setup::{remove, setup};
pub use self::sshd::{DROP_IN, ready, ssh_port};

/// The user the app signs in as: no shell, no password, no home.
pub const USER: &str = "botloft-tunnel";

/// Longest name a host may have.
const HOST_MAX: usize = 253;

/// Whether `host` can be where the app opens its tunnel: a name or an IP
/// address, never something `ssh` would take for an option.
pub fn valid_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= HOST_MAX
        && !host.starts_with('-')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':' | '_'))
}

/// The one kind of key a device may bring.
const KIND: &str = "ssh-ed25519";

/// Length of an ed25519 public key.
const ED25519_LEN: u32 = 32;

/// `ssh-ed25519 AAAA...`, one line checked whole so nothing else reaches
/// sshd through it: a comment after the key is dropped.
pub fn public_key(value: &str) -> Result<String, ValidationError> {
    let refused = || ValidationError("sshKey must be an ssh-ed25519 public key".to_owned());
    let value = value.trim();
    if value.contains(['\n', '\r', '\0']) {
        return Err(refused());
    }
    let mut parts = value.split_ascii_whitespace();
    let (Some(KIND), Some(body)) = (parts.next(), parts.next()) else {
        return Err(refused());
    };
    let blob = STANDARD.decode(body).map_err(|_| refused())?;
    // The key's own record: its kind, then the 32 bytes, each with its length.
    let mut head = Vec::new();
    head.extend_from_slice(&(KIND.len() as u32).to_be_bytes());
    head.extend_from_slice(KIND.as_bytes());
    head.extend_from_slice(&ED25519_LEN.to_be_bytes());
    if blob.len() != head.len() + ED25519_LEN as usize || !blob.starts_with(&head) {
        return Err(refused());
    }
    Ok(format!("{KIND} {}", STANDARD.encode(&blob)))
}

/// What `botloftd tunnel keys` prints for sshd: each device's key, allowed
/// nothing but a tunnel to Botloft's port.
pub fn authorized_keys(keys: &[String], port: u16) -> String {
    keys.iter()
        .filter_map(|key| public_key(key).ok())
        .map(|key| format!("restrict,port-forwarding,permitopen=\"127.0.0.1:{port}\" {key}\n"))
        .collect()
}

/// What `botloftd tunnel keys [user]` prints, sshd's `AuthorizedKeysCommand`:
/// run as the daemon's user, it only reads. Any other user gets no key.
pub fn keys(home: &Path, user: Option<&str>) -> anyhow::Result<String> {
    if user.is_some_and(|user| user != USER) {
        return Ok(String::new());
    }
    let db = home.join("botloft.db");
    if !db.exists() {
        // Botloft never ran: no device either.
        return Ok(String::new());
    }
    let config = Config::load(&home.join("config.toml"), false)?;
    let store = Store::open_read_only(&db)
        .with_context(|| format!("cannot read the database in {}", home.display()))?;
    Ok(authorized_keys(&store.device_ssh_keys()?, config.port))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A key in the form `ssh-keygen -t ed25519` writes.
    const KEY: &str =
        "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIB0fX3dnSrK6yp6CmM8kU4DhkBiQ0jFv1UuSGBNPmSdw";

    #[test]
    fn an_ed25519_key_passes_without_its_comment() {
        assert_eq!(public_key(&format!("{KEY} botloft")).as_deref(), Ok(KEY));
        assert_eq!(public_key(&format!("  {KEY}  \n")).as_deref(), Ok(KEY));
    }

    #[test]
    fn anything_else_is_refused() {
        let body = KEY.split(' ').nth(1).expect("body");
        for bad in [
            "",
            "ssh-ed25519",
            &format!("ssh-rsa {body}"),
            &format!("command=\"sh\" {KEY}"),
            &format!("{KEY}\nssh-ed25519 {body}"),
            "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5",
            "ssh-ed25519 not-base64!",
            // An ed25519 record one byte short.
            &format!(
                "ssh-ed25519 {}",
                STANDARD.encode(&STANDARD.decode(body).expect("b64")[..50])
            ),
        ] {
            assert!(public_key(bad).is_err(), "{bad:?} passed");
        }
    }

    #[test]
    fn a_host_is_a_name_or_an_address() {
        for good in ["vps-1.example.com", "203.0.113.10", "2001:db8::1", "box_2"] {
            assert!(valid_host(good), "{good}");
        }
        for bad in ["", "-oProxyCommand=sh", "a b", "host;rm", &"a".repeat(254)] {
            assert!(!valid_host(bad), "{bad}");
        }
    }

    #[test]
    fn sshd_hears_the_devices_keys_and_only_for_the_tunnel_user() {
        let home = tempfile::tempdir().expect("home");
        assert_eq!(keys(home.path(), Some(USER)).expect("no database yet"), "");
        std::fs::write(home.path().join("config.toml"), "port = 45799\n").expect("config");
        let store = Store::open(&home.path().join("botloft.db")).expect("store");
        store
            .insert_device("Laptop", "aa", Some(KEY), 1)
            .expect("device");
        drop(store);
        let line = format!("restrict,port-forwarding,permitopen=\"127.0.0.1:45799\" {KEY}\n");
        assert_eq!(keys(home.path(), Some(USER)).expect("keys"), line);
        assert_eq!(keys(home.path(), None).expect("keys"), line);
        assert_eq!(keys(home.path(), Some("root")).expect("keys"), "");
    }

    #[test]
    fn each_key_may_only_reach_botloft() {
        let lines = authorized_keys(&[KEY.to_owned(), "junk".to_owned()], 45710);
        assert_eq!(
            lines,
            format!("restrict,port-forwarding,permitopen=\"127.0.0.1:45710\" {KEY}\n")
        );
    }
}
