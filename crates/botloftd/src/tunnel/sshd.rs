//! sshd's side of the app's own tunnel (spec 42.6): the drop-in that lets
//! `botloft-tunnel` in only with the devices' keys and only to Botloft, and
//! what sshd itself says about its settings.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::USER;

/// Written by `botloftd tunnel setup`. sshd reads it where its sshd_config
/// includes sshd_config.d, as on Debian, Ubuntu, Fedora and Alpine.
pub const DROP_IN: &str = "/etc/ssh/sshd_config.d/botloft.conf";

/// Where sshd usually is.
const SSHD: &[&str] = &["/usr/sbin/sshd", "/usr/local/sbin/sshd", "/sbin/sshd"];

/// Whether `tunnel setup` prepared this server, so `pair` makes a key.
pub fn ready() -> bool {
    Path::new(DROP_IN).is_file()
}

/// The drop-in: sshd asks `command`, as `account`, for the keys of the
/// tunnel user, who may only open a tunnel to Botloft on `port`.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn drop_in(command: &Path, account: &str, port: u16) -> String {
    let command = command.display();
    format!(
        "# Botloft (spec 42.6): the Botloft app on another computer opens a tunnel\n\
         # to Botloft as {USER}, with a key of its own. Written by\n\
         # `botloftd tunnel setup`; `botloftd tunnel remove` takes it away.\n\
         Match User {USER}\n\
         \tAuthorizedKeysFile none\n\
         \tAuthorizedKeysCommand {command} tunnel keys %u\n\
         \tAuthorizedKeysCommandUser {account}\n\
         \tAuthenticationMethods publickey\n\
         \tPubkeyAuthentication yes\n\
         \tAllowTcpForwarding local\n\
         \tPermitOpen 127.0.0.1:{port}\n\
         \tAllowAgentForwarding no\n\
         \tAllowStreamLocalForwarding no\n\
         \tX11Forwarding no\n\
         \tPermitTunnel no\n\
         \tPermitTTY no\n\
         \tForceCommand /bin/false\n"
    )
}

pub(super) fn find_sshd() -> Option<PathBuf> {
    SSHD.iter().map(PathBuf::from).find(|path| path.is_file())
}

/// The port sshd listens on, as `sshd -T` says (which needs root); 22 when
/// it cannot say.
pub fn ssh_port() -> u16 {
    find_sshd()
        .and_then(|sshd| settings(&sshd, None))
        .and_then(|text| value(&text, "port")?.parse().ok())
        .unwrap_or(22)
}

/// `sshd -T`, for `user` when given: the settings as sshd would use them.
pub(super) fn settings(sshd: &Path, user: Option<&str>) -> Option<String> {
    let mut command = Command::new(sshd);
    command.arg("-T");
    if let Some(user) = user {
        command
            .arg("-C")
            .arg(format!("user={user},host=localhost,addr=127.0.0.1"));
    }
    let output = command.stdin(Stdio::null()).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// The first value of `keyword` in what `sshd -T` printed.
pub(super) fn value<'a>(settings: &'a str, keyword: &str) -> Option<&'a str> {
    settings.lines().find_map(|line| {
        let (key, value) = line.trim().split_once(' ')?;
        (key == keyword).then(|| value.trim())
    })
}

/// Whether sshd's settings for the tunnel user ask `command` for the keys:
/// the drop-in is read, and nothing before it says otherwise.
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub(super) fn asks(settings: &str, command: &Path) -> bool {
    value(settings, "authorizedkeyscommand")
        == Some(format!("{} tunnel keys %u", command.display()).as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_drop_in_lets_the_tunnel_user_reach_only_botloft() {
        let text = drop_in(Path::new("/usr/local/bin/botloftd"), "botloft", 45710);
        let lines: Vec<&str> = text.lines().map(str::trim).collect();
        for line in [
            "Match User botloft-tunnel",
            "AuthorizedKeysFile none",
            "AuthorizedKeysCommand /usr/local/bin/botloftd tunnel keys %u",
            "AuthorizedKeysCommandUser botloft",
            "AuthenticationMethods publickey",
            "AllowTcpForwarding local",
            "PermitOpen 127.0.0.1:45710",
            "PermitTTY no",
        ] {
            assert!(lines.contains(&line), "{line} is missing");
        }
        // The Match block holds every setting: nothing applies to anyone else.
        let first_setting = lines
            .iter()
            .position(|line| !line.starts_with('#'))
            .expect("settings");
        assert_eq!(lines[first_setting], "Match User botloft-tunnel");
    }

    #[test]
    fn what_sshd_says_is_read() {
        let said = "port 2222\nport 22\naddressfamily any\n\
                    authorizedkeyscommand /usr/local/bin/botloftd tunnel keys %u\n";
        assert_eq!(value(said, "port"), Some("2222"));
        assert_eq!(value(said, "pidfile"), None);
        assert!(asks(said, Path::new("/usr/local/bin/botloftd")));
        assert!(!asks(said, Path::new("/opt/botloftd")));
        assert!(!asks(
            "authorizedkeyscommand none\n",
            Path::new("/usr/local/bin/botloftd")
        ));
    }
}
