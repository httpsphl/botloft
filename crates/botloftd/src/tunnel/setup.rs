//! `botloftd tunnel setup` and `remove` (spec 42.6), as root on a Linux
//! server: the user the app signs in as and sshd's drop-in, which sshd
//! checks before it takes it.

use std::path::Path;

/// Lets the app open its own tunnel here.
pub fn setup(home: &Path) -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        linux::setup(home)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = home;
        anyhow::bail!("the app's own tunnel is set up only on a Linux server")
    }
}

/// Takes back what [`setup`] did.
pub fn remove() -> anyhow::Result<()> {
    #[cfg(target_os = "linux")]
    {
        linux::remove()
    }
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("the app's own tunnel is set up only on a Linux server")
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::io::Write as _;
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};

    use anyhow::{Context, ensure};

    use super::super::USER;
    use super::super::sshd::{self, DROP_IN};
    use crate::config::Config;
    use crate::platform;

    const NOLOGIN: &[&str] = &["/usr/sbin/nologin", "/sbin/nologin", "/bin/false"];

    pub fn setup(home: &Path) -> anyhow::Result<()> {
        ensure!(
            platform::running_as_root(),
            "run this as root: sudo botloftd tunnel setup"
        );
        let config = Config::load(&home.join("config.toml"), false)?;
        let command = std::env::current_exe()
            .and_then(|path| path.canonicalize())
            .context("cannot find this program")?;
        ensure!(
            platform::only_root_writes(&command)?
                && !command.to_string_lossy().contains(char::is_whitespace),
            "sshd only runs a program that root alone can change, such as /usr/local/bin/botloftd from install.sh; this one is {}",
            command.display()
        );
        let account = platform::owner_of(home)?
            .with_context(|| format!("the owner of {} has no name", home.display()))?;
        let sshd = sshd::find_sshd().context(
            "there is no SSH server (sshd) here: install openssh-server, then run this again",
        )?;
        add_user()?;
        std::fs::write(DROP_IN, sshd::drop_in(&command, &account, config.port))
            .with_context(|| format!("cannot write {DROP_IN}"))?;
        if let Err(err) = check(&sshd, &command) {
            let _ = std::fs::remove_file(DROP_IN);
            return Err(err);
        }
        reload(&sshd);
        println!(
            "The Botloft app can now open its own SSH connection to this server, as {USER},\n\
             which can only reach Botloft. Pair the app with: sudo botloftd pair"
        );
        Ok(())
    }

    pub fn remove() -> anyhow::Result<()> {
        ensure!(
            platform::running_as_root(),
            "run this as root: sudo botloftd tunnel remove"
        );
        if Path::new(DROP_IN).exists() {
            std::fs::remove_file(DROP_IN).with_context(|| format!("cannot remove {DROP_IN}"))?;
            if let Some(sshd) = sshd::find_sshd() {
                reload(&sshd);
            }
        }
        if exists(USER) {
            if let Some(userdel) = program("userdel") {
                run(&userdel, &["--force", USER])?;
            } else if let Some(deluser) = program("deluser") {
                run(&deluser, &[USER])?;
            }
            for groupdel in ["groupdel", "delgroup"].into_iter().filter_map(program) {
                let _ = run(&groupdel, &[USER]);
            }
        }
        println!("The app's own tunnel is off: {USER} and its sshd settings are gone.");
        Ok(())
    }

    /// The user the app signs in as: no shell, no home, and a password
    /// field that matches no password.
    fn add_user() -> anyhow::Result<()> {
        if !exists(USER) {
            let nologin = NOLOGIN
                .iter()
                .copied()
                .find(|path| Path::new(path).exists())
                .unwrap_or("/bin/false");
            if let Some(useradd) = program("useradd") {
                run(
                    &useradd,
                    &[
                        "--system",
                        "--user-group",
                        "--home-dir",
                        "/nonexistent",
                        "--no-create-home",
                        "--shell",
                        nologin,
                        USER,
                    ],
                )?;
            } else {
                // BusyBox (Alpine).
                let addgroup = program("addgroup").context("cannot add users: no useradd")?;
                let adduser = program("adduser").context("cannot add users: no useradd")?;
                run(&addgroup, &["-S", USER])?;
                run(
                    &adduser,
                    &[
                        "-S",
                        "-D",
                        "-H",
                        "-h",
                        "/nonexistent",
                        "-s",
                        nologin,
                        "-G",
                        USER,
                        USER,
                    ],
                )?;
            }
        }
        // A locked password ("!") keeps an sshd without PAM from letting even
        // a key in; "*" matches no password either.
        let chpasswd = program("chpasswd").context("cannot find chpasswd")?;
        let mut child = Command::new(chpasswd)
            .arg("-e")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .context("cannot run chpasswd")?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(format!("{USER}:*\n").as_bytes())?;
        }
        let output = child.wait_with_output()?;
        ensure!(
            output.status.success(),
            "chpasswd failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(())
    }

    /// sshd's own test of its settings, then whether the tunnel user's
    /// settings come from the drop-in.
    fn check(sshd: &Path, command: &Path) -> anyhow::Result<()> {
        let output = Command::new(sshd)
            .arg("-t")
            .stdin(Stdio::null())
            .output()
            .context("cannot run sshd")?;
        ensure!(
            output.status.success(),
            "sshd refused the settings, so they were taken back: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
        let settings = sshd::settings(sshd, Some(USER)).unwrap_or_default();
        ensure!(
            sshd::asks(&settings, command),
            "sshd does not read {DROP_IN}: put `Include /etc/ssh/sshd_config.d/*.conf` at the top of /etc/ssh/sshd_config, then run this again"
        );
        Ok(())
    }

    /// sshd reads its settings again. A connection already open keeps going.
    fn reload(sshd: &Path) {
        if Path::new("/run/systemd/system").is_dir() {
            for unit in ["ssh.service", "sshd.service"] {
                let _ = Command::new("systemctl")
                    .args(["try-reload-or-restart", unit])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            return;
        }
        // Without systemd, the sshd that listens rereads them on SIGHUP.
        let pid = sshd::settings(sshd, None)
            .and_then(|text| std::fs::read_to_string(sshd::value(&text, "pidfile")?).ok())
            .and_then(|pid| pid.trim().parse().ok());
        if let Some(pid) = pid {
            let _ = platform::hang_up(pid);
        }
    }

    fn exists(user: &str) -> bool {
        Command::new("id")
            .arg(user)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
    }

    /// A system program, also where `sudo`'s PATH would not look.
    fn program(name: &str) -> Option<PathBuf> {
        let path = std::env::var_os("PATH").unwrap_or_default();
        std::env::split_paths(&path)
            .chain(["/usr/sbin", "/sbin", "/usr/bin", "/bin"].map(PathBuf::from))
            .map(|dir| dir.join(name))
            .find(|file| file.is_file())
    }

    fn run(program: &Path, args: &[&str]) -> anyhow::Result<()> {
        let output = Command::new(program)
            .args(args)
            .stdin(Stdio::null())
            .output()
            .with_context(|| format!("cannot run {}", program.display()))?;
        ensure!(
            output.status.success(),
            "{} failed: {}",
            program.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
        Ok(())
    }
}
