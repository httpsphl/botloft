//! What this computer holds of the account: the token of its sign-in, in
//! `secrets\cloud.json`, readable only by the current user (spec 27.3, 13).

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::platform;

#[derive(Clone, Serialize, Deserialize)]
pub struct Credentials {
    /// The server the token is for: another address is another account.
    pub url: String,
    pub token: String,
    pub email: String,
    pub device: String,
}

impl fmt::Debug for Credentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Credentials(<redacted>)")
    }
}

fn file(secrets_dir: &Path) -> PathBuf {
    secrets_dir.join("cloud.json")
}

/// What is stored; nothing stored, or a file that cannot be read, is signed out.
pub fn load(secrets_dir: &Path) -> Option<Credentials> {
    let bytes = std::fs::read(file(secrets_dir)).ok()?;
    serde_json::from_slice(&bytes).ok()
}

pub fn save(secrets_dir: &Path, credentials: &Credentials) -> io::Result<()> {
    std::fs::create_dir_all(secrets_dir)?;
    platform::restrict_to_current_user(secrets_dir)?;
    let path = file(secrets_dir);
    let tmp = path.with_extension("tmp");
    std::fs::write(
        &tmp,
        serde_json::to_vec(credentials).map_err(io::Error::other)?,
    )?;
    platform::restrict_to_current_user(&tmp)?;
    std::fs::rename(&tmp, &path)
}

pub fn remove(secrets_dir: &Path) {
    let _ = std::fs::remove_file(file(secrets_dir));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_are_kept_and_never_printed() {
        let dir = tempfile::tempdir().expect("dir");
        assert!(load(dir.path()).is_none());
        let credentials = Credentials {
            url: "https://cloud.example.org".to_owned(),
            token: "secret-token".to_owned(),
            email: "ana@exemplo.com".to_owned(),
            device: "dev_1".to_owned(),
        };
        save(dir.path(), &credentials).expect("save");
        let back = load(dir.path()).expect("load");
        assert_eq!(back.token, "secret-token");
        assert_eq!(format!("{back:?}"), "Credentials(<redacted>)");
        remove(dir.path());
        assert!(load(dir.path()).is_none());
    }

    #[test]
    fn a_broken_file_reads_as_signed_out() {
        let dir = tempfile::tempdir().expect("dir");
        std::fs::write(dir.path().join("cloud.json"), "{not json").expect("write");
        assert!(load(dir.path()).is_none());
    }
}
