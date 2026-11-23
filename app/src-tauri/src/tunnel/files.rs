//! The tunnel's two files in the app's own folder: the device's private key,
//! which only this user can read and which goes once `ssh` has read it, and
//! the pinned host key. The spec is checked before anything is written: it
//! came from a pasted connection key.

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use super::TunnelSpec;
use super::ssh::HOST_ALIAS;

const KEY_HEAD: &str = "-----BEGIN OPENSSH PRIVATE KEY-----";
const KEY_TAIL: &str = "-----END OPENSSH PRIVATE KEY-----";
/// Far more than an ed25519 key in OpenSSH's format takes.
const KEY_MAX: usize = 16 * 1024;
const HOST_MAX: usize = 253;
const USER_MAX: usize = 32;
const HOST_KEY_KINDS: &[&str] = &[
    "ssh-ed25519",
    "ecdsa-sha2-nistp256",
    "ecdsa-sha2-nistp384",
    "ecdsa-sha2-nistp521",
    "ssh-rsa",
];

/// Whether the app opens a tunnel with `spec`: nothing in it can become an
/// option of `ssh` or another line of its files. Names the part that is not.
pub fn check(spec: &TunnelSpec) -> Result<(), &'static str> {
    if !host_ok(&spec.host) {
        return Err("the server's address");
    }
    if !user_ok(&spec.user) {
        return Err("the user");
    }
    if [spec.port, spec.remote_port, spec.local_port].contains(&0) {
        return Err("a port");
    }
    if !host_key_ok(&spec.host_key) {
        return Err("the server's host key");
    }
    if !key_ok(&spec.key) {
        return Err("the key");
    }
    Ok(())
}

fn host_ok(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= HOST_MAX
        && !host.starts_with('-')
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | ':' | '_'))
}

fn user_ok(user: &str) -> bool {
    !user.is_empty()
        && user.len() <= USER_MAX
        && !user.starts_with('-')
        && user
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_'))
}

fn host_key_ok(line: &str) -> bool {
    let mut parts = line.split(' ');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(kind), Some(body), None) => {
            HOST_KEY_KINDS.contains(&kind)
                && !body.is_empty()
                && body
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '='))
        }
        _ => false,
    }
}

fn key_ok(key: &str) -> bool {
    let key = key.trim();
    key.len() <= KEY_MAX
        && key.starts_with(KEY_HEAD)
        && key.ends_with(KEY_TAIL)
        && key.chars().all(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '+' | '/' | '=' | '-' | ' ' | '\n' | '\r')
        })
}

/// The folder for the files, which only this user may open.
pub fn prepare(folder: &Path) -> io::Result<()> {
    fs::create_dir_all(folder)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        fs::set_permissions(folder, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(windows)]
    super::acl::only_this_user(folder)?;
    Ok(())
}

/// Writes the private key where only this user reads it.
pub fn write_key(folder: &Path, key: &str) -> io::Result<PathBuf> {
    let path = folder.join("key");
    let _ = fs::remove_file(&path);
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(&path)?;
    // Empty until only this user can open it.
    #[cfg(windows)]
    super::acl::only_this_user(&path)?;
    file.write_all(key.trim().replace("\r\n", "\n").as_bytes())?;
    // OpenSSH reads the last line only with its newline.
    file.write_all(b"\n")?;
    Ok(path)
}

/// The one host key `ssh` accepts, under [`HOST_ALIAS`].
pub fn write_known_hosts(folder: &Path, host_key: &str) -> io::Result<PathBuf> {
    let path = folder.join("known_hosts");
    fs::write(&path, format!("{HOST_ALIAS} {host_key}\n"))?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> TunnelSpec {
        TunnelSpec {
            host: "vps-1.example.com".to_owned(),
            port: 22,
            user: "botloft-tunnel".to_owned(),
            host_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIHost+/=".to_owned(),
            key: format!("{KEY_HEAD}\nb3BlbnNzaC1rZXktdjEAAAAA\n{KEY_TAIL}\n"),
            remote_port: 45710,
            local_port: 45720,
        }
    }

    #[test]
    fn a_key_from_botloftd_pair_passes() {
        assert_eq!(check(&spec()), Ok(()));
    }

    /// A change to a good spec, and the part `check` names for it.
    type Case = (fn(&mut TunnelSpec), &'static str);

    #[test]
    fn nothing_can_become_an_option_or_a_line() {
        let cases: [Case; 7] = [
            (
                |s| s.host = "-oProxyCommand=calc".to_owned(),
                "the server's address",
            ),
            (|s| s.host = "a b".to_owned(), "the server's address"),
            (|s| s.user = "-l".to_owned(), "the user"),
            (|s| s.port = 0, "a port"),
            (
                |s| s.host_key = "ssh-ed25519 AAAA\nevil ssh-ed25519 AAAA".to_owned(),
                "the server's host key",
            ),
            (
                |s| s.host_key = "ssh-dss AAAA".to_owned(),
                "the server's host key",
            ),
            (|s| s.key = "not a key".to_owned(), "the key"),
        ];
        for (change, part) in cases {
            let mut spec = spec();
            change(&mut spec);
            assert_eq!(check(&spec), Err(part));
        }
    }

    #[test]
    fn the_files_hold_the_key_and_the_pinned_host_key() {
        let dir = std::env::temp_dir().join(format!("botloft-tunnel-test-{}", std::process::id()));
        prepare(&dir).expect("folder");
        let key = write_key(&dir, &spec().key.replace('\n', "\r\n")).expect("key");
        let text = fs::read_to_string(&key).expect("read");
        assert!(text.starts_with(KEY_HEAD) && text.ends_with(&format!("{KEY_TAIL}\n")));
        assert!(!text.contains('\r'));
        let known = write_known_hosts(&dir, &spec().host_key).expect("known");
        assert_eq!(
            fs::read_to_string(known).expect("read"),
            format!("botloft-server {}\n", spec().host_key)
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            let mode = fs::metadata(&key).expect("meta").permissions().mode();
            assert_eq!(mode & 0o777, 0o600);
        }
        fs::remove_dir_all(&dir).expect("clean");
    }
}
