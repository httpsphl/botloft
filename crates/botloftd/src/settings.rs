//! The settings the owner changes in the app (spec 6, 11.2). They live in
//! `config.toml` and are written back in place, so the owner's own lines
//! and comments stay, and they take effect at once.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

use botloft_core::protocol::{Settings, SettingsUpdateParams};
use tokio::sync::watch;

use crate::config::Config;

pub struct LiveSettings {
    /// Where they are saved; `None` keeps them in memory (tests).
    file: Option<PathBuf>,
    current: Mutex<Settings>,
    keep_awake: watch::Sender<bool>,
}

impl LiveSettings {
    pub fn new(file: Option<PathBuf>, config: &Config) -> Self {
        let current = Settings {
            start_with_windows: config.start_with_windows,
            keep_awake: config.keep_awake,
        };
        Self {
            file,
            current: Mutex::new(current),
            keep_awake: watch::Sender::new(current.keep_awake),
        }
    }

    pub fn get(&self) -> Settings {
        *self.lock()
    }

    /// Follows `keep_awake` (spec 14).
    pub fn keep_awake(&self) -> watch::Receiver<bool> {
        self.keep_awake.subscribe()
    }

    /// Saves the settings in `update` and applies them.
    pub fn save(&self, update: SettingsUpdateParams) -> io::Result<Settings> {
        let mut current = self.lock();
        if let Some(file) = &self.file {
            write(file, update)?;
        }
        if let Some(start) = update.start_with_windows {
            current.start_with_windows = start;
        }
        if let Some(keep_awake) = update.keep_awake {
            current.keep_awake = keep_awake;
            self.keep_awake.send_replace(keep_awake);
        }
        Ok(*current)
    }

    fn lock(&self) -> MutexGuard<'_, Settings> {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Sets the changed keys in `config.toml` and leaves the rest as it was.
/// The new file replaces the old one whole, so a crash never leaves half.
fn write(file: &Path, update: SettingsUpdateParams) -> io::Result<()> {
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err),
    };
    let mut config: toml_edit::DocumentMut = text
        .parse()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    if let Some(start) = update.start_with_windows {
        config["start_with_windows"] = toml_edit::value(start);
    }
    if let Some(keep_awake) = update.keep_awake {
        config["keep_awake"] = toml_edit::value(keep_awake);
    }
    let staged = file.with_extension("toml.new");
    std::fs::write(&staged, config.to_string())?;
    std::fs::rename(&staged, file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saving_keeps_the_owners_lines_and_comments() {
        let dir = tempfile::tempdir().expect("dir");
        let file = dir.path().join("config.toml");
        let mine =
            "# my port\nport = 45999\n\n[bots]\napproval_timeout_minutes = 30 # half an hour\n";
        std::fs::write(&file, mine).expect("write");
        let settings = LiveSettings::new(Some(file.clone()), &Config::parse(mine).expect("parse"));
        let mut awake = settings.keep_awake();

        let saved = settings
            .save(SettingsUpdateParams {
                keep_awake: Some(false),
                ..SettingsUpdateParams::default()
            })
            .expect("save");
        assert_eq!(
            saved,
            Settings {
                start_with_windows: true,
                keep_awake: false
            }
        );
        assert!(awake.has_changed().expect("sender") && !*awake.borrow_and_update());

        let text = std::fs::read_to_string(&file).expect("read");
        assert!(text.starts_with("# my port\nport = 45999\nkeep_awake = false\n"));
        assert!(text.contains("approval_timeout_minutes = 30 # half an hour"));
        assert!(!text.contains("start_with_windows"), "only what changed");
        let config = Config::parse(&text).expect("still valid");
        assert!(!config.keep_awake && config.start_with_windows && config.port == 45999);

        settings
            .save(SettingsUpdateParams {
                start_with_windows: Some(false),
                keep_awake: Some(true),
            })
            .expect("save");
        let config = Config::parse(&std::fs::read_to_string(&file).expect("read")).expect("parse");
        assert!(config.keep_awake && !config.start_with_windows);
        assert_eq!(
            settings.get(),
            Settings {
                start_with_windows: false,
                keep_awake: true
            }
        );
    }

    #[test]
    fn a_missing_file_is_created_with_just_the_change() {
        let dir = tempfile::tempdir().expect("dir");
        let file = dir.path().join("config.toml");
        let settings = LiveSettings::new(Some(file.clone()), &Config::default());
        settings
            .save(SettingsUpdateParams {
                start_with_windows: Some(false),
                ..SettingsUpdateParams::default()
            })
            .expect("save");
        assert_eq!(
            std::fs::read_to_string(&file).expect("read"),
            "start_with_windows = false\n"
        );
    }
}
