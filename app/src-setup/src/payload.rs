//! The NSIS installer this setup carries (spec 15.7): written to a folder of
//! its own under %TEMP%, run, and deleted with the folder afterwards.

use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

/// Empty in dev builds, which only rehearse (build.rs).
static PAYLOAD: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/payload.exe"));

/// Without the installer inside, the setup shows its pages and pretends.
pub fn rehearsal() -> bool {
    PAYLOAD.is_empty()
}

/// Why the installer did not finish, for "Details".
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Failure {
    /// The installer's exit code, when it ran.
    pub code: Option<i32>,
    pub detail: String,
}

impl From<io::Error> for Failure {
    fn from(err: io::Error) -> Self {
        Self {
            code: None,
            detail: err.to_string(),
        }
    }
}

/// A folder of our own under %TEMP%, deleted when dropped.
pub struct TempFolder(PathBuf);

impl TempFolder {
    pub fn new(parent: &Path) -> io::Result<Self> {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.subsec_nanos());
        for attempt in 0..16u32 {
            let tag = seed ^ std::process::id().rotate_left(16) ^ attempt.wrapping_mul(0x9e37_79b9);
            let path = parent.join(format!("Botloft-setup-{tag:08x}"));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(err) => return Err(err),
            }
        }
        Err(io::Error::other("no free folder name under %TEMP%"))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Writes the installer into the folder and returns its path.
    fn write_installer(&self, bytes: &[u8]) -> io::Result<PathBuf> {
        let exe = self.0.join("Botloft-installer.exe");
        std::fs::write(&exe, bytes)?;
        Ok(exe)
    }
}

impl Drop for TempFolder {
    fn drop(&mut self) {
        // Best effort: %TEMP% is cleaned by Windows anyway.
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs the installer with `args` from a fresh folder, waits for it and
/// deletes the folder.
fn run(args: &[&str]) -> Result<(), Failure> {
    let folder = TempFolder::new(&std::env::temp_dir())?;
    let exe = folder.write_installer(PAYLOAD)?;
    let status = Command::new(&exe)
        .args(args)
        .current_dir(folder.path())
        .status()?;
    if status.success() {
        return Ok(());
    }
    let name = exe
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    Err(Failure {
        code: status.code(),
        detail: format!("{name} {} stopped with {status}", args.join(" ")),
    })
}

/// The installer with no pages of its own (`/S`).
pub fn install_quietly() -> Result<(), Failure> {
    run(&["/S"])
}

/// The installer with its own pages, for when this window cannot help:
/// no WebView2, or a quiet install that failed.
pub fn install_classic() -> Result<(), Failure> {
    run(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_temp_folder_is_unique_and_goes_away() {
        let parent = std::env::temp_dir();
        let first = TempFolder::new(&parent).expect("first folder");
        let second = TempFolder::new(&parent).expect("second folder");
        assert_ne!(first.path(), second.path());
        let exe = first.write_installer(b"MZ").expect("write");
        assert_eq!(std::fs::read(&exe).expect("read"), b"MZ");
        let path = first.path().to_path_buf();
        drop(first);
        assert!(!path.exists());
    }
}
