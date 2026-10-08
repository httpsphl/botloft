//! The phones this computer connected, and the keys it shares with each: in
//! `secrets\mobile.json`, readable only by the current user (spec 28.5, 13).
//! Nothing here goes in the log, and nothing of it is in the backup.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD as B64;
use serde::{Deserialize, Serialize};

use super::seal::Keys;
use crate::platform;

/// One connected phone.
#[derive(Clone, Serialize, Deserialize)]
pub struct Phone {
    /// The id the server gave the phone's device.
    pub id: String,
    pub name: String,
    pub paired_at: i64,
    pub last_seen_at: Option<i64>,
    c2p: String,
    p2c: String,
    /// The last number used to seal a message to the phone. Saved before the
    /// message goes, so a crash never reuses a number under the same key.
    pub sent: u64,
    /// The highest number taken from the phone; the next must be greater.
    pub received: u64,
}

impl Phone {
    pub fn new(id: String, name: String, paired_at: i64, keys: &Keys) -> Self {
        Self {
            id,
            name,
            paired_at,
            last_seen_at: None,
            c2p: B64.encode(keys.c2p),
            p2c: B64.encode(keys.p2c),
            sent: 0,
            received: 0,
        }
    }

    pub fn keys(&self) -> Option<Keys> {
        let decode = |text: &str| -> Option<[u8; 32]> { B64.decode(text).ok()?.try_into().ok() };
        Some(Keys {
            c2p: decode(&self.c2p)?,
            p2c: decode(&self.p2c)?,
        })
    }
}

/// What the file holds: the phones, and whose they are. Another server or
/// another sign-in is another account, and these phones do not carry over.
#[derive(Default, Serialize, Deserialize)]
struct File {
    url: String,
    computer: String,
    phones: Vec<Phone>,
}

pub struct Phones {
    path: PathBuf,
    file: Mutex<File>,
}

impl Phones {
    pub fn open(secrets_dir: &Path) -> Self {
        let path = secrets_dir.join("mobile.json");
        let file = std::fs::read(&path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        Self {
            path,
            file: Mutex::new(file),
        }
    }

    fn lock(&self) -> MutexGuard<'_, File> {
        self.file
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// The phones that belong to this sign-in (`url` and the computer's
    /// device id); none if the file is of another.
    pub fn list(&self, url: &str, computer: &str) -> Vec<Phone> {
        let file = self.lock();
        if file.url == url && file.computer == computer {
            file.phones.clone()
        } else {
            Vec::new()
        }
    }

    pub fn get(&self, url: &str, computer: &str, id: &str) -> Option<Phone> {
        self.list(url, computer)
            .into_iter()
            .find(|phone| phone.id == id)
    }

    /// Changes one phone and saves. False when there is no such phone.
    pub fn update(
        &self,
        url: &str,
        computer: &str,
        id: &str,
        change: impl FnOnce(&mut Phone),
    ) -> io::Result<bool> {
        let mut file = self.lock();
        if file.url != url || file.computer != computer {
            return Ok(false);
        }
        let Some(phone) = file.phones.iter_mut().find(|phone| phone.id == id) else {
            return Ok(false);
        };
        change(phone);
        self.save(&file)?;
        Ok(true)
    }

    /// Adds a phone, replacing one with the same id. A file of another
    /// sign-in is started over.
    pub fn add(&self, url: &str, computer: &str, phone: Phone) -> io::Result<()> {
        let mut file = self.lock();
        if file.url != url || file.computer != computer {
            *file = File {
                url: url.to_owned(),
                computer: computer.to_owned(),
                phones: Vec::new(),
            };
        }
        file.phones.retain(|other| other.id != phone.id);
        file.phones.push(phone);
        self.save(&file)
    }

    /// Forgets a phone and its keys. False when there was none.
    pub fn remove(&self, url: &str, computer: &str, id: &str) -> io::Result<bool> {
        let mut file = self.lock();
        if file.url != url || file.computer != computer {
            return Ok(false);
        }
        let before = file.phones.len();
        file.phones.retain(|phone| phone.id != id);
        let removed = file.phones.len() != before;
        if removed {
            self.save(&file)?;
        }
        Ok(removed)
    }

    /// Forgets every phone: the computer left the account.
    pub fn clear(&self) {
        let mut file = self.lock();
        *file = File::default();
        let _ = std::fs::remove_file(&self.path);
    }

    fn save(&self, file: &File) -> io::Result<()> {
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
            platform::restrict_to_current_user(dir)?;
        }
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, serde_json::to_vec(file).map_err(io::Error::other)?)?;
        platform::restrict_to_current_user(&tmp)?;
        std::fs::rename(&tmp, &self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> Keys {
        Keys {
            c2p: [1; 32],
            p2c: [2; 32],
        }
    }

    #[test]
    fn phones_are_kept_per_sign_in_and_survive_a_restart() {
        let dir = tempfile::tempdir().expect("dir");
        let phones = Phones::open(dir.path());
        assert!(phones.list("https://a", "dev_1").is_empty());
        phones
            .add(
                "https://a",
                "dev_1",
                Phone::new("dev_p".into(), "Celular".into(), 5, &keys()),
            )
            .expect("add");
        assert_eq!(phones.list("https://a", "dev_1").len(), 1);
        // Another server, or another sign-in of this computer, has none.
        assert!(phones.list("https://b", "dev_1").is_empty());
        assert!(phones.list("https://a", "dev_2").is_empty());

        let changed = phones
            .update("https://a", "dev_1", "dev_p", |phone| {
                phone.sent = 9;
                phone.received = 4;
            })
            .expect("update");
        assert!(changed);
        let again = Phones::open(dir.path());
        let phone = again.get("https://a", "dev_1", "dev_p").expect("kept");
        assert_eq!(
            (phone.sent, phone.received, phone.name.as_str()),
            (9, 4, "Celular")
        );
        assert!(phone.keys().expect("keys") == keys());

        assert!(again.remove("https://a", "dev_1", "dev_p").expect("remove"));
        assert!(!again.remove("https://a", "dev_1", "dev_p").expect("again"));
        assert!(
            Phones::open(dir.path())
                .list("https://a", "dev_1")
                .is_empty()
        );
    }

    #[test]
    fn a_broken_file_reads_as_no_phones_and_clear_removes_the_file() {
        let dir = tempfile::tempdir().expect("dir");
        std::fs::write(dir.path().join("mobile.json"), "{not json").expect("write");
        let phones = Phones::open(dir.path());
        assert!(phones.list("u", "c").is_empty());
        phones
            .add("u", "c", Phone::new("p".into(), "x".into(), 1, &keys()))
            .expect("add");
        phones.clear();
        assert!(!dir.path().join("mobile.json").exists());
        assert!(phones.list("u", "c").is_empty());
    }
}
