//! Putting the new daemon in place of the running one (spec 42.11): one
//! rename, so the running daemon keeps its file until the service restarts
//! it.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, bail};

/// The service install.sh sets up.
const SERVICE: &str = "botloft.service";

/// Unpacks the daemon from the release's `tar.gz` and puts it in place of
/// `current` in one rename, so the running daemon keeps its file until it
/// restarts.
pub(super) fn install(archive: &[u8], current: &Path) -> anyhow::Result<()> {
    let folder = current.parent().context("this program has no folder")?;
    let unpacked = tempfile_dir(folder)?;
    let tarball = unpacked.join("botloftd.tar.gz");
    std::fs::write(&tarball, archive)?;
    let status = Command::new("tar")
        .arg("-xzf")
        .arg(&tarball)
        .arg("-C")
        .arg(&unpacked)
        .arg("botloftd")
        .status()
        .context("cannot run tar")?;
    if !status.success() {
        let _ = std::fs::remove_dir_all(&unpacked);
        bail!("the download does not hold a daemon");
    }
    let next = current.with_extension("new");
    let done = std::fs::copy(unpacked.join("botloftd"), &next)
        .and_then(|_| set_runnable(&next))
        .and_then(|()| std::fs::rename(&next, current));
    let _ = std::fs::remove_dir_all(&unpacked);
    done.with_context(|| format!("cannot replace {}", current.display()))
}

/// A new empty folder next to `folder`'s files, for unpacking.
fn tempfile_dir(folder: &Path) -> anyhow::Result<PathBuf> {
    let name = format!(".botloftd-update-{}", std::process::id());
    let dir = folder.join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("cannot write in {}", folder.display()))?;
    Ok(dir)
}

#[cfg(unix)]
fn set_runnable(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
}

#[cfg(not(unix))]
fn set_runnable(_path: &Path) -> std::io::Result<()> {
    Ok(())
}

/// Restarts the service install.sh set up; without one, says so.
pub(super) fn restart() {
    let restarted = Command::new("systemctl")
        .args(["restart", SERVICE])
        .status()
        .is_ok_and(|status| status.success());
    if !restarted {
        println!("Start Botloft again for the new version to run.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The release's package: `botloftd` at the top of a `tar.gz`, made with
    /// the system's `tar`; `None` where there is none.
    fn package(dir: &Path, daemon: &[u8]) -> Option<Vec<u8>> {
        let inside = dir.join("pack");
        std::fs::create_dir_all(&inside).ok()?;
        std::fs::write(inside.join("botloftd"), daemon).ok()?;
        let archive = dir.join("botloftd.tar.gz");
        let made = Command::new("tar")
            .arg("-czf")
            .arg(&archive)
            .arg("-C")
            .arg(&inside)
            .arg("botloftd")
            .status();
        made.is_ok_and(|status| status.success())
            .then(|| std::fs::read(&archive).ok())
            .flatten()
    }

    #[test]
    fn the_new_daemon_takes_the_old_ones_place_and_nothing_is_left() {
        let dir = tempfile::tempdir().expect("dir");
        let bin = dir.path().join("bin");
        std::fs::create_dir_all(&bin).expect("bin");
        let current = bin.join("botloftd");
        std::fs::write(&current, b"old daemon").expect("old");
        let Some(archive) = package(dir.path(), b"new daemon") else {
            return; // No `tar` here.
        };
        install(&archive, &current).expect("install");
        assert_eq!(std::fs::read(&current).expect("read"), b"new daemon");
        let left: Vec<_> = std::fs::read_dir(&bin)
            .expect("list")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .collect();
        assert_eq!(left, ["botloftd"], "no scratch files stay");

        assert!(install(b"not a package", &current).is_err());
        assert_eq!(
            std::fs::read(&current).expect("read"),
            b"new daemon",
            "untouched"
        );
    }
}
