//! The settings the owner changes in the app (spec 6, 11.2). They live in
//! `config.toml` and are written back in place, so the owner's own lines
//! and comments stay, and they take effect at once.

use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use botloft_core::protocol::{AgentKind, Settings, SettingsUpdateParams};
use tokio::sync::watch;

use crate::config::Config;

/// The agents the "experimental agents" setting switches on, by their name
/// in `config.toml`.
pub const EXPERIMENTAL_AGENTS: [&str; 2] = ["agy", "codex"];

pub struct LiveSettings {
    /// Where they are saved; `None` keeps them in memory (tests).
    file: Option<PathBuf>,
    current: Mutex<Current>,
    keep_awake: watch::Sender<bool>,
}

struct Current {
    settings: Settings,
    /// `settings.approval_wait_minutes`, or shorter in tests.
    approval_wait: Duration,
    /// `settings.sleep_after_minutes`, or shorter in tests; `None` never.
    sleep_after: Option<Duration>,
}

impl LiveSettings {
    pub fn new(file: Option<PathBuf>, config: &Config) -> Self {
        let minutes = u32::try_from(config.bots.approval_timeout_minutes).unwrap_or(u32::MAX);
        let sleep = u32::try_from(config.bots.sleep_after_minutes).unwrap_or(u32::MAX);
        let settings = Settings {
            start_with_windows: config.start_with_windows,
            keep_awake: config.keep_awake,
            approval_wait_minutes: minutes,
            sleep_after_minutes: sleep,
            // A name this build does not know is Claude Code.
            default_agent: config.default_agent.parse().unwrap_or(AgentKind::Claude),
            show_experimental_agents: !config.experimental_agents.is_empty(),
        };
        Self {
            file,
            current: Mutex::new(Current {
                settings,
                approval_wait: minutes_to_wait(minutes),
                sleep_after: minutes_to_sleep(sleep),
            }),
            keep_awake: watch::Sender::new(settings.keep_awake),
        }
    }

    /// Requests wait `wait` for the owner, even under a minute (tests).
    #[must_use]
    pub fn with_approval_wait(self, wait: Duration) -> Self {
        {
            let mut current = self.lock();
            current.approval_wait = wait;
            current.settings.approval_wait_minutes =
                u32::try_from(wait.as_secs() / 60).unwrap_or(u32::MAX);
        }
        self
    }

    /// Bots sleep after `after` idle, even under a minute, or never
    /// (tests).
    #[must_use]
    pub fn with_sleep_after(self, after: Option<Duration>) -> Self {
        {
            let mut current = self.lock();
            current.sleep_after = after;
            current.settings.sleep_after_minutes = after.map_or(0, |after| {
                u32::try_from(after.as_secs() / 60).unwrap_or(u32::MAX)
            });
        }
        self
    }

    /// Changes the sleep time at once, even under a minute (tests).
    pub fn set_sleep_after(&self, after: Option<Duration>) {
        let mut current = self.lock();
        current.sleep_after = after;
        current.settings.sleep_after_minutes = after.map_or(0, |after| {
            u32::try_from(after.as_secs() / 60).unwrap_or(u32::MAX)
        });
    }

    pub fn get(&self) -> Settings {
        self.lock().settings
    }

    /// How long a permission request waits for the owner (spec 10.1).
    pub fn approval_wait(&self) -> Duration {
        self.lock().approval_wait
    }

    /// How long a bot stays idle before it sleeps (spec 7.7); `None` never.
    pub fn sleep_after(&self) -> Option<Duration> {
        self.lock().sleep_after
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
            current.settings.start_with_windows = start;
        }
        if let Some(keep_awake) = update.keep_awake {
            current.settings.keep_awake = keep_awake;
            self.keep_awake.send_replace(keep_awake);
        }
        if let Some(minutes) = update.approval_wait_minutes {
            current.settings.approval_wait_minutes = minutes;
            current.approval_wait = minutes_to_wait(minutes);
        }
        if let Some(minutes) = update.sleep_after_minutes {
            current.settings.sleep_after_minutes = minutes;
            current.sleep_after = minutes_to_sleep(minutes);
        }
        if let Some(agent) = update.default_agent {
            current.settings.default_agent = agent;
        }
        if let Some(show) = update.show_experimental_agents {
            current.settings.show_experimental_agents = show;
        }
        Ok(current.settings)
    }

    /// Saves the program of `kind` in `config.toml` (`agy_path` or
    /// `codex_path`); empty goes back to looking for it.
    pub fn save_program(&self, kind: AgentKind, path: &str) -> io::Result<()> {
        let key = match kind {
            AgentKind::Agy => "agy_path",
            AgentKind::Codex => "codex_path",
            AgentKind::Claude => return Ok(()),
        };
        let _guard = self.lock();
        match &self.file {
            Some(file) => write_text(file, key, path),
            None => Ok(()),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Current> {
        self.current
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

fn minutes_to_wait(minutes: u32) -> Duration {
    Duration::from_secs(u64::from(minutes) * 60)
}

/// 0 is never.
fn minutes_to_sleep(minutes: u32) -> Option<Duration> {
    (minutes > 0).then(|| minutes_to_wait(minutes))
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
        set(config.as_table_mut(), "start_with_windows", start);
    }
    if let Some(keep_awake) = update.keep_awake {
        set(config.as_table_mut(), "keep_awake", keep_awake);
    }
    if let Some(agent) = update.default_agent {
        set(config.as_table_mut(), "default_agent", agent.as_str());
    }
    if let Some(show) = update.show_experimental_agents {
        let mut names = toml_edit::Array::new();
        if show {
            names.extend(EXPERIMENTAL_AGENTS);
        }
        set(config.as_table_mut(), "experimental_agents", names);
    }
    if let Some(minutes) = update.approval_wait_minutes {
        let bots = config["bots"]
            .or_insert(toml_edit::table())
            .as_table_like_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bots is not a table"))?;
        set(bots, "approval_timeout_minutes", i64::from(minutes));
    }
    if let Some(minutes) = update.sleep_after_minutes {
        let bots = config["bots"]
            .or_insert(toml_edit::table())
            .as_table_like_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "bots is not a table"))?;
        set(bots, "sleep_after_minutes", i64::from(minutes));
    }
    let staged = file.with_extension("toml.new");
    std::fs::write(&staged, config.to_string())?;
    std::fs::rename(&staged, file)
}

/// Sets one text key of `config.toml` and leaves the rest as it was.
fn write_text(file: &Path, key: &str, value: &str) -> io::Result<()> {
    let text = match std::fs::read_to_string(file) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => String::new(),
        Err(err) => return Err(err),
    };
    let mut config: toml_edit::DocumentMut = text
        .parse()
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err))?;
    set(config.as_table_mut(), key, value);
    let staged = file.with_extension("toml.new");
    std::fs::write(&staged, config.to_string())?;
    std::fs::rename(&staged, file)
}

/// Sets `key` in `table`, keeping a comment after the old value.
fn set(table: &mut dyn toml_edit::TableLike, key: &str, value: impl Into<toml_edit::Value>) {
    let mut value = value.into();
    if let Some(old) = table.get(key).and_then(toml_edit::Item::as_value) {
        *value.decor_mut() = old.decor().clone();
    }
    table.insert(key, toml_edit::Item::Value(value));
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
                keep_awake: false,
                approval_wait_minutes: 30,
                sleep_after_minutes: 30,
                default_agent: AgentKind::Claude,
                show_experimental_agents: false,
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
                approval_wait_minutes: Some(240),
                sleep_after_minutes: Some(0),
                default_agent: None,
                show_experimental_agents: None,
            })
            .expect("save");
        let text = std::fs::read_to_string(&file).expect("read");
        assert!(text.contains(
            "keep_awake = true
"
        ));
        assert!(text.contains("approval_timeout_minutes = 240 # half an hour"));
        let config = Config::parse(&text).expect("parse");
        assert!(config.keep_awake && !config.start_with_windows);
        assert_eq!(config.bots.approval_timeout_minutes, 240);
        // 0 is never: agents stay up (spec 7.7).
        assert_eq!(config.bots.sleep_after_minutes, 0);
        assert_eq!(settings.sleep_after(), None);
        assert_eq!(
            settings.get(),
            Settings {
                start_with_windows: false,
                keep_awake: true,
                approval_wait_minutes: 240,
                sleep_after_minutes: 0,
                default_agent: AgentKind::Claude,
                show_experimental_agents: false,
            }
        );
        assert_eq!(settings.approval_wait(), Duration::from_secs(4 * 3600));
    }

    #[test]
    fn the_experimental_agents_are_listed_in_the_file_and_taken_out_again() {
        let dir = tempfile::tempdir().expect("dir");
        let file = dir.path().join("config.toml");
        let settings = LiveSettings::new(Some(file.clone()), &Config::default());
        assert!(!settings.get().show_experimental_agents);
        let on = SettingsUpdateParams {
            show_experimental_agents: Some(true),
            ..SettingsUpdateParams::default()
        };
        assert!(settings.save(on).expect("save").show_experimental_agents);
        let config = Config::parse(&std::fs::read_to_string(&file).expect("read")).expect("parse");
        assert_eq!(config.experimental_agents, ["agy", "codex"]);
        // A daemon started now shows them.
        assert!(
            LiveSettings::new(None, &config)
                .get()
                .show_experimental_agents
        );

        let off = SettingsUpdateParams {
            show_experimental_agents: Some(false),
            ..SettingsUpdateParams::default()
        };
        assert!(!settings.save(off).expect("save").show_experimental_agents);
        let config = Config::parse(&std::fs::read_to_string(&file).expect("read")).expect("parse");
        assert!(config.experimental_agents.is_empty());
    }

    #[test]
    fn an_agents_program_is_saved_as_its_own_key_and_cleared() {
        let dir = tempfile::tempdir().expect("dir");
        let file = dir.path().join("config.toml");
        std::fs::write(
            &file,
            "# mine
port = 45999
",
        )
        .expect("write");
        let settings = LiveSettings::new(Some(file.clone()), &Config::default());
        settings
            .save_program(AgentKind::Codex, "C:/tools/codex.exe")
            .expect("save");
        let text = std::fs::read_to_string(&file).expect("read");
        assert!(text.starts_with(
            "# mine
port = 45999
"
        ));
        let config = Config::parse(&text).expect("valid");
        assert_eq!(config.codex_path, "C:/tools/codex.exe");
        assert_eq!(config.agy_path, "");
        settings.save_program(AgentKind::Codex, "").expect("clear");
        let config = Config::parse(&std::fs::read_to_string(&file).expect("read")).expect("valid");
        assert_eq!(config.codex_path, "");
    }

    #[test]
    fn a_new_wait_goes_in_the_bots_table() {
        let dir = tempfile::tempdir().expect("dir");
        let file = dir.path().join("config.toml");
        let settings = LiveSettings::new(Some(file.clone()), &Config::default());
        settings
            .save(SettingsUpdateParams {
                approval_wait_minutes: Some(15),
                ..SettingsUpdateParams::default()
            })
            .expect("save");
        assert_eq!(
            std::fs::read_to_string(&file).expect("read"),
            "[bots]\napproval_timeout_minutes = 15\n"
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
