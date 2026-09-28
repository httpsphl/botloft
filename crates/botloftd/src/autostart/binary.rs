//! The daemon runs from its own copy in `<home>\bin`, never from the app's
//! folder, so the app's installer can replace its files while the daemon
//! runs (spec 14).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const EXE_NAME: &str = if cfg!(windows) {
    "botloftd.exe"
} else {
    "botloftd"
};
const STAGED: &str = "botloftd.new";
const OLD_SUFFIX: &str = ".old";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Installed {
    pub path: PathBuf,
    /// Whether a different binary was put in place.
    pub changed: bool,
}

/// Copies `source` to `<bin_dir>\botloftd.exe`, unless it is that file or
/// has the same bytes. A running exe cannot be overwritten but can be
/// renamed, so the old one moves aside and is deleted on a later install.
pub fn install(source: &Path, bin_dir: &Path) -> io::Result<Installed> {
    fs::create_dir_all(bin_dir)?;
    remove_old(bin_dir);
    let target = bin_dir.join(EXE_NAME);
    let unchanged = Installed {
        path: target.clone(),
        changed: false,
    };
    if is_same_file(source, &target) {
        return Ok(unchanged);
    }
    let bytes = fs::read(source)?;
    match fs::read(&target) {
        Ok(current) if current == bytes => return Ok(unchanged),
        Ok(_) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    let staged = bin_dir.join(STAGED);
    fs::write(&staged, &bytes)?;
    if target.exists() {
        fs::rename(&target, aside(bin_dir))?;
    }
    fs::rename(&staged, &target)?;
    Ok(Installed {
        path: target,
        changed: true,
    })
}

/// Deletes the installed binary and what was moved aside, as far as
/// nothing still runs them.
pub fn remove(bin_dir: &Path) {
    let _ = fs::remove_file(bin_dir.join(EXE_NAME));
    let _ = fs::remove_file(bin_dir.join(STAGED));
    remove_old(bin_dir);
    let _ = fs::remove_dir(bin_dir);
}

fn is_same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// A free `botloftd.<n>.old` name. A previous one may still be running.
fn aside(bin_dir: &Path) -> PathBuf {
    (1..)
        .map(|n| bin_dir.join(format!("botloftd.{n}{OLD_SUFFIX}")))
        .find(|path| !path.exists())
        .unwrap_or_else(|| bin_dir.join(format!("botloftd{OLD_SUFFIX}")))
}

fn remove_old(bin_dir: &Path) {
    let Ok(entries) = fs::read_dir(bin_dir) else {
        return;
    };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().ends_with(OLD_SUFFIX) {
            // Still running: it goes next time.
            let _ = fs::remove_file(entry.path());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .expect("read dir")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        names
    }

    #[test]
    fn copies_once_and_replaces_only_a_different_binary() {
        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("sidecar.exe");
        let bin = dir.path().join("bin");
        fs::write(&source, b"v1").expect("write");

        let first = install(&source, &bin).expect("install");
        assert!(first.changed);
        assert_eq!(fs::read(&first.path).expect("read"), b"v1");
        assert!(!install(&source, &bin).expect("again").changed);
        assert!(
            !install(&first.path, &bin).expect("itself").changed,
            "running install from the installed copy changes nothing"
        );

        fs::write(&source, b"v2").expect("write");
        let second = install(&source, &bin).expect("update");
        assert!(second.changed);
        assert_eq!(fs::read(&second.path).expect("read"), b"v2");
        assert_eq!(names(&bin), ["botloftd.1.old".to_owned(), EXE_NAME.into()]);

        fs::write(&source, b"v3").expect("write");
        install(&source, &bin).expect("update");
        assert_eq!(
            names(&bin),
            ["botloftd.1.old".to_owned(), EXE_NAME.into()],
            "the v1 copy went away and v2 moved aside"
        );

        remove(&bin);
        assert!(!bin.exists());
    }

    /// A running exe is open with share-delete: it can be renamed, not
    /// written.
    #[cfg(windows)]
    #[test]
    fn replaces_a_binary_that_is_running() {
        use std::os::windows::fs::OpenOptionsExt as _;
        const FILE_SHARE_READ: u32 = 1;
        const FILE_SHARE_DELETE: u32 = 4;

        let dir = tempfile::tempdir().expect("tempdir");
        let source = dir.path().join("sidecar.exe");
        let bin = dir.path().join("bin");
        fs::write(&source, b"v1").expect("write");
        let installed = install(&source, &bin).expect("install");
        let running = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_DELETE)
            .open(&installed.path)
            .expect("open");
        assert!(
            fs::write(&installed.path, b"x").is_err(),
            "cannot be written"
        );

        fs::write(&source, b"v2").expect("write");
        let updated = install(&source, &bin).expect("update");
        assert_eq!(fs::read(&updated.path).expect("read"), b"v2");
        drop(running);
    }
}
