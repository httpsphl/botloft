//! OpenSSH's `ssh` as the app runs it for its own tunnel (spec 42.6): only
//! what the app passes counts (no config file, no agent, no other key), the
//! server's host key must be the pinned one, and the one forward is from
//! this computer's 127.0.0.1 to Botloft's 127.0.0.1 there.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use super::TunnelSpec;

/// The name the pinned host key goes by, whatever address reaches it.
pub const HOST_ALIAS: &str = "botloft-server";

/// Seconds between the keepalives that find a dead connection.
const ALIVE_INTERVAL: u32 = 15;

/// The `ssh` to run: Windows' own, else the first on PATH.
pub fn program() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        let system = std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
        let own = system.join(r"System32\OpenSSH\ssh.exe");
        if own.is_file() {
            return Some(own);
        }
    }
    let name = if cfg!(windows) { "ssh.exe" } else { "ssh" };
    let path = std::env::var_os("PATH").unwrap_or_default();
    // Only `;` (or `:`) separates; quotes around a folder are dropped. A
    // stray quote must not swallow the folders after it.
    let separator = if cfg!(windows) { ';' } else { ':' };
    path.to_string_lossy()
        .split(separator)
        .map(|dir| dir.trim().trim_matches('"'))
        .filter(|dir| !dir.is_empty())
        .map(|dir| Path::new(dir).join(name))
        .chain(["/usr/bin/ssh", "/bin/ssh"].map(PathBuf::from))
        .find(|file| file.is_file())
}

/// Every argument `ssh` gets: options first, `--`, then the server.
pub fn args(spec: &TunnelSpec, key: &Path, known_hosts: &Path, local_port: u16) -> Vec<OsString> {
    let option = |text: String| [OsString::from("-o"), OsString::from(text)];
    let mut args: Vec<OsString> = ["-F", "none", "-N", "-T"].map(OsString::from).to_vec();
    for text in [
        "BatchMode=yes".to_owned(),
        "IdentitiesOnly=yes".to_owned(),
        "IdentityAgent=none".to_owned(),
        "StrictHostKeyChecking=yes".to_owned(),
        format!("HostKeyAlias={HOST_ALIAS}"),
        "UpdateHostKeys=no".to_owned(),
        "ExitOnForwardFailure=yes".to_owned(),
        "ConnectTimeout=15".to_owned(),
        format!("ServerAliveInterval={ALIVE_INTERVAL}"),
        "ServerAliveCountMax=3".to_owned(),
        "LogLevel=ERROR".to_owned(),
    ] {
        args.extend(option(text));
    }
    args.push("-i".into());
    args.push(key.into());
    let mut known = OsString::from("UserKnownHostsFile=");
    known.push(known_hosts);
    args.extend([OsString::from("-o"), known]);
    args.push("-L".into());
    args.push(format!("127.0.0.1:{local_port}:127.0.0.1:{}", spec.remote_port).into());
    args.push("-p".into());
    args.push(spec.port.to_string().into());
    args.push("-l".into());
    args.push(spec.user.clone().into());
    args.push("--".into());
    args.push(spec.host.clone().into());
    args
}

/// What went wrong, from what `ssh` wrote: the app shows a sentence for it.
pub fn reason(said: &str) -> &'static str {
    let said = said.to_ascii_lowercase();
    if said.contains("host key verification failed")
        || said.contains("remote host identification has changed")
    {
        "host_key"
    } else if said.contains("unprotected private key") || said.contains("bad permissions") {
        // Before "permission denied", which follows when ssh ignores the key.
        "key_file"
    } else if said.contains("permission denied") || said.contains("prohibited") {
        "refused"
    } else if [
        "could not resolve",
        "name or service not known",
        "no such host",
        "timed out",
        "connection refused",
        "network is unreachable",
        "no route to host",
        "connection closed",
        "connection reset",
    ]
    .iter()
    .any(|sign| said.contains(sign))
    {
        "unreachable"
    } else {
        "failed"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> TunnelSpec {
        TunnelSpec {
            host: "203.0.113.10".to_owned(),
            port: 2222,
            user: "botloft-tunnel".to_owned(),
            host_key: "ssh-ed25519 AAAA".to_owned(),
            key: String::new(),
            remote_port: 45710,
            local_port: 45720,
        }
    }

    #[test]
    fn ssh_gets_one_forward_and_the_server_last() {
        let args: Vec<String> = args(&spec(), Path::new("key"), Path::new("known_hosts"), 45721)
            .into_iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let joined = args.join(" ");
        assert!(joined.starts_with("-F none -N -T "));
        for part in [
            "-o BatchMode=yes",
            "-o IdentitiesOnly=yes",
            "-o StrictHostKeyChecking=yes",
            "-o HostKeyAlias=botloft-server",
            "-o ExitOnForwardFailure=yes",
            "-i key",
            "-o UserKnownHostsFile=known_hosts",
            "-L 127.0.0.1:45721:127.0.0.1:45710",
            "-p 2222 -l botloft-tunnel -- 203.0.113.10",
        ] {
            assert!(joined.contains(part), "{part} is missing: {joined}");
        }
        assert_eq!(args.last().map(String::as_str), Some("203.0.113.10"));
    }

    #[test]
    fn what_ssh_said_becomes_a_reason() {
        let cases = [
            (
                "botloft-tunnel@1.2.3.4: Permission denied (publickey).",
                "refused",
            ),
            ("Host key verification failed.", "host_key"),
            (
                "ssh: Could not resolve hostname vps: No such host is known.",
                "unreachable",
            ),
            (
                "ssh: connect to host 1.2.3.4 port 22: Connection timed out",
                "unreachable",
            ),
            (
                "Bad permissions. Try removing permissions for user: PC\\Others\n\
                 This private key will be ignored.\n\
                 botloft-tunnel@127.0.0.1: Permission denied (publickey).",
                "key_file",
            ),
            (
                "channel 2: open failed: administratively prohibited",
                "refused",
            ),
            ("something new", "failed"),
        ];
        for (said, expected) in cases {
            assert_eq!(reason(said), expected, "{said}");
        }
    }
}
