//! Where the automatic backup keeps its passphrase (spec 27.10): the
//! system's credential store, behind a trait so tests keep it in memory.

use std::io;
use std::path::Path;
use std::sync::Mutex;

use sha2::{Digest, Sha256};

use crate::platform;

pub trait SecretStore: Send + Sync {
    /// This computer has somewhere safe to keep a secret.
    fn available(&self) -> bool;
    fn save(&self, secret: &str) -> io::Result<()>;
    fn load(&self) -> io::Result<Option<String>>;
    fn delete(&self) -> io::Result<()>;
}

/// The credential store of the system, under a name of this data folder: a
/// development folder and the real one never share a secret.
pub struct SystemSecrets {
    target: String,
}

impl SystemSecrets {
    pub fn for_home(home: &Path) -> Self {
        let digest = Sha256::digest(home.to_string_lossy().to_lowercase().as_bytes());
        let short: String = digest.iter().take(4).map(|b| format!("{b:02x}")).collect();
        Self {
            target: format!("Botloft backup passphrase {short}"),
        }
    }
}

impl SecretStore for SystemSecrets {
    fn available(&self) -> bool {
        platform::secret_available()
    }

    fn save(&self, secret: &str) -> io::Result<()> {
        platform::secret_save(&self.target, secret)
    }

    fn load(&self) -> io::Result<Option<String>> {
        platform::secret_load(&self.target)
    }

    fn delete(&self) -> io::Result<()> {
        platform::secret_delete(&self.target)
    }
}

/// A store in memory, for tests and for a system without one (`available`
/// says whether it keeps anything).
pub struct MemorySecrets {
    available: bool,
    secret: Mutex<Option<String>>,
}

impl MemorySecrets {
    pub fn new() -> Self {
        Self {
            available: true,
            secret: Mutex::new(None),
        }
    }

    /// A system with nowhere to keep a secret.
    pub fn unavailable() -> Self {
        Self {
            available: false,
            secret: Mutex::new(None),
        }
    }
}

impl Default for MemorySecrets {
    fn default() -> Self {
        Self::new()
    }
}

impl SecretStore for MemorySecrets {
    fn available(&self) -> bool {
        self.available
    }

    fn save(&self, secret: &str) -> io::Result<()> {
        *self.secret.lock().unwrap_or_else(|p| p.into_inner()) = Some(secret.to_owned());
        Ok(())
    }

    fn load(&self) -> io::Result<Option<String>> {
        Ok(self
            .secret
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone())
    }

    fn delete(&self) -> io::Result<()> {
        *self.secret.lock().unwrap_or_else(|p| p.into_inner()) = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_data_folder_has_its_own_name() {
        let a = SystemSecrets::for_home(Path::new(r"C:\Users\ana\AppData\Local\Botloft"));
        let same = SystemSecrets::for_home(Path::new(r"c:\users\ana\appdata\local\botloft"));
        let dev = SystemSecrets::for_home(Path::new(r"C:\code\.dev\home"));
        assert_eq!(a.target, same.target);
        assert_ne!(a.target, dev.target);
        assert!(a.target.starts_with("Botloft backup passphrase "));
    }

    #[test]
    fn the_memory_store_keeps_and_forgets() {
        let store = MemorySecrets::new();
        assert_eq!(store.load().expect("empty"), None);
        store.save("secret").expect("save");
        assert_eq!(store.load().expect("load").as_deref(), Some("secret"));
        store.delete().expect("delete");
        assert_eq!(store.load().expect("gone"), None);
        assert!(!MemorySecrets::unavailable().available());
    }
}
