//! What `botloftd pair` puts in a connection key for the app's own tunnel
//! (spec 42.6): a key pair made here for the device, this server's host key,
//! which the app pins, and the address the app most likely reaches it at.

use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

use super::public_key;

/// sshd's host keys, the shortest first: the app keeps the first there is.
const HOST_KEYS: &[&str] = &[
    "/etc/ssh/ssh_host_ed25519_key.pub",
    "/etc/ssh/ssh_host_ecdsa_key.pub",
    "/etc/ssh/ssh_host_rsa_key.pub",
];

/// Host key kinds the app accepts.
const HOST_KEY_KINDS: &[&str] = &[
    "ssh-ed25519",
    "ecdsa-sha2-nistp256",
    "ecdsa-sha2-nistp384",
    "ecdsa-sha2-nistp521",
    "ssh-rsa",
];

/// A key pair `ssh-keygen` made for one device: the private half goes in
/// the connection key, the public half to the daemon.
pub struct Keypair {
    /// In OpenSSH's own format, as `ssh-keygen` writes it.
    pub private: String,
    /// `ssh-ed25519 AAAA...`
    pub public: String,
}

impl std::fmt::Debug for Keypair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Keypair")
            .field("private", &"<redacted>")
            .field("public", &self.public)
            .finish()
    }
}

impl Keypair {
    /// A new ed25519 pair, which `ssh-keygen` makes in a folder only this
    /// user can open, gone after.
    pub fn generate() -> io::Result<Self> {
        Self::generate_in(&std::env::temp_dir())
    }

    fn generate_in(temp: &Path) -> io::Result<Self> {
        let folder = Private::create(temp)?;
        let file = folder.0.join("key");
        let output = Command::new("ssh-keygen")
            .args(["-q", "-t", "ed25519", "-N", "", "-C", "botloft", "-f"])
            .arg(&file)
            .stdin(Stdio::null())
            .output()?;
        if !output.status.success() {
            return Err(io::Error::other(format!(
                "ssh-keygen failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }
        let private = std::fs::read_to_string(&file)?;
        let public = public_key(&std::fs::read_to_string(file.with_extension("pub"))?)
            .map_err(|err| io::Error::other(err.0))?;
        Ok(Self { private, public })
    }
}

/// A folder in `temp` that only this user can open, removed with what is
/// in it when dropped.
struct Private(PathBuf);

impl Private {
    fn create(temp: &Path) -> io::Result<Self> {
        let mut bytes = [0u8; 8];
        getrandom::fill(&mut bytes).map_err(|err| io::Error::other(err.to_string()))?;
        let path = temp.join(format!("botloft-pair-{}", hex::encode(bytes)));
        #[cfg_attr(not(unix), allow(unused_mut))]
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        builder.create(&path)?;
        Ok(Self(path))
    }
}

impl Drop for Private {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// This server's host key (`ssh-ed25519 AAAA...`), which the app pins: it
/// then opens the tunnel only to this server.
pub fn host_key() -> Option<String> {
    HOST_KEYS
        .iter()
        .find_map(|path| host_key_in(&std::fs::read_to_string(path).ok()?))
}

fn host_key_in(line: &str) -> Option<String> {
    let mut parts = line.split_ascii_whitespace();
    let (kind, body) = (parts.next()?, parts.next()?);
    (HOST_KEY_KINDS.contains(&kind) && STANDARD.decode(body).is_ok())
        .then(|| format!("{kind} {body}"))
}

/// The address the server's own traffic to the internet leaves from: the
/// one the app most likely reaches it at. A socket that only connects sends
/// nothing; the documentation addresses only pick the default route.
pub fn address() -> Option<IpAddr> {
    let targets: [(SocketAddr, SocketAddr); 2] = [
        (
            (Ipv4Addr::UNSPECIFIED, 0).into(),
            (Ipv4Addr::new(192, 0, 2, 1), 9).into(),
        ),
        (
            (Ipv6Addr::UNSPECIFIED, 0).into(),
            (Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1), 9).into(),
        ),
    ];
    targets.into_iter().find_map(|(local, target)| {
        let socket = UdpSocket::bind(local).ok()?;
        socket.connect(target).ok()?;
        let ip = socket.local_addr().ok()?.ip();
        (!ip.is_unspecified() && !ip.is_loopback()).then_some(ip)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_host_key_line_loses_its_comment() {
        assert_eq!(
            host_key_in("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5 root@vps\n").as_deref(),
            Some("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5")
        );
        assert_eq!(host_key_in("ssh-dss AAAAB3NzaC1kc3M= root@vps"), None);
        assert_eq!(host_key_in("ssh-ed25519 not*base64"), None);
        assert_eq!(host_key_in(""), None);
    }

    #[test]
    fn ssh_keygen_makes_a_pair_and_leaves_nothing() {
        let temp = tempfile::tempdir().expect("temp");
        let keypair = match Keypair::generate_in(temp.path()) {
            Ok(keypair) => keypair,
            // A computer without OpenSSH's client cannot run this test.
            Err(err) if err.kind() == io::ErrorKind::NotFound => return,
            Err(err) => panic!("ssh-keygen: {err}"),
        };
        assert!(
            keypair
                .private
                .starts_with("-----BEGIN OPENSSH PRIVATE KEY-----")
        );
        assert!(keypair.public.starts_with("ssh-ed25519 AAAA"));
        assert!(!format!("{keypair:?}").contains("PRIVATE KEY"));
        let left = std::fs::read_dir(temp.path()).expect("temp").count();
        assert_eq!(left, 0, "the key's folder stayed");
    }
}
