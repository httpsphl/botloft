//! `config.toml` (spec 6). Every key is optional; missing keys take the
//! defaults below. Unknown keys are rejected so typos do not go unnoticed.

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// TCP port on 127.0.0.1.
    pub port: u16,
    /// Empty means `%USERPROFILE%\Botloft`.
    pub workspaces_root: String,
    /// Empty means resolve `claude` from PATH.
    pub claude_path: String,
    /// Empty means look for `agy` in its installer's folder and on PATH.
    pub agy_path: String,
    /// The agent for new bots: `"claude"`, or an enabled experimental one
    /// (spec 30). Changed in the app's Settings.
    pub default_agent: String,
    /// Agents not meant for everyone yet, by name (`"agy"`, spec 30). A bot
    /// can only be made for one of these when it is listed here.
    pub experimental_agents: Vec<String>,
    /// Start when the owner signs in to Windows (spec 14).
    pub start_with_windows: bool,
    /// Keep the computer awake while a bot works (spec 14).
    pub keep_awake: bool,
    pub log_level: String,
    pub supervisor: SupervisorConfig,
    pub courier: CourierConfig,
    pub bots: BotsConfig,
    pub browser: BrowserConfig,
    pub tasks: TasksConfig,
    pub cloud: CloudConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            port: 45710,
            workspaces_root: String::new(),
            claude_path: String::new(),
            agy_path: String::new(),
            default_agent: "claude".to_owned(),
            experimental_agents: Vec::new(),
            start_with_windows: true,
            keep_awake: true,
            log_level: "info".to_owned(),
            supervisor: SupervisorConfig::default(),
            courier: CourierConfig::default(),
            bots: BotsConfig::default(),
            browser: BrowserConfig::default(),
            tasks: TasksConfig::default(),
            cloud: CloudConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SupervisorConfig {
    pub restart_backoff_initial_ms: u64,
    pub restart_backoff_max_ms: u64,
    pub fresh_start_if_dies_within_s: u64,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            restart_backoff_initial_ms: 1_000,
            restart_backoff_max_ms: 300_000,
            fresh_start_if_dies_within_s: 15,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CourierConfig {
    pub poll_interval_ms: u64,
    pub lease_ms: u64,
    pub max_attempts: u32,
    pub retry_backoff_initial_ms: u64,
    pub retry_backoff_max_ms: u64,
}

impl Default for CourierConfig {
    fn default() -> Self {
        Self {
            poll_interval_ms: 10_000,
            lease_ms: 15_000,
            max_attempts: 8,
            retry_backoff_initial_ms: 2_000,
            retry_backoff_max_ms: 120_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BotsConfig {
    /// How long a permission request waits for the owner (spec 10.1).
    pub approval_timeout_minutes: u64,
    /// Largest attachment, per file (spec 9.5).
    pub attachment_max_mb: u64,
    /// Most bots a chief's suggestions can bring a crew to (spec 10.2).
    pub max_per_crew: usize,
}

impl Default for BotsConfig {
    fn default() -> Self {
        Self {
            approval_timeout_minutes: 60,
            attachment_max_mb: 20,
            max_per_crew: 12,
        }
    }
}

/// The server of the account unless `[cloud] url` says another one (spec 27).
pub const DEFAULT_CLOUD_URL: &str = "https://botloft.comitium.com.br";

/// The account and the copies in the cloud (spec 27).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CloudConfig {
    /// The server; set it to an empty string to turn the account off.
    pub url: String,
    /// How often a sign-in waiting for its link asks whether it was opened.
    pub poll_ms: u64,
}

impl Default for CloudConfig {
    fn default() -> Self {
        Self {
            url: DEFAULT_CLOUD_URL.to_owned(),
            poll_ms: 2_000,
        }
    }
}

/// The bots' browser (spec 21.9).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct BrowserConfig {
    /// Empty means the Microsoft Edge that comes with Windows.
    pub path: String,
    /// Closes a browser nobody used or watched for this long.
    pub idle_minutes: u64,
    /// Browsers open at the same time.
    pub max_open: usize,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            path: String::new(),
            idle_minutes: 10,
            max_open: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TasksConfig {
    pub max_hops: u32,
    pub default_deadline_minutes: u32,
}

impl Default for TasksConfig {
    fn default() -> Self {
        Self {
            max_hops: 4,
            default_deadline_minutes: 120,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid config {path}: {message}")]
    Invalid { path: PathBuf, message: String },
}

const LOG_LEVELS: &[&str] = &["trace", "debug", "info", "warn", "error"];

impl Config {
    /// Reads the file at `path`. A missing file yields the defaults unless
    /// the path was given explicitly (`required`).
    pub fn load(path: &Path, required: bool) -> Result<Self, ConfigError> {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound && !required => {
                return Ok(Self::default());
            }
            Err(source) => {
                return Err(ConfigError::Read {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        Self::parse(&text).map_err(|message| ConfigError::Invalid {
            path: path.to_owned(),
            message,
        })
    }

    pub fn parse(text: &str) -> Result<Self, String> {
        let config: Self = toml::from_str(text).map_err(|err| err.message().to_owned())?;
        if config.port == 0 {
            return Err("port must be between 1 and 65535".to_owned());
        }
        if config.browser.max_open == 0 || config.browser.idle_minutes == 0 {
            return Err("browser.max_open and browser.idle_minutes must be at least 1".to_owned());
        }
        if !LOG_LEVELS.contains(&config.log_level.as_str()) {
            return Err(format!(
                "log_level must be one of {}",
                LOG_LEVELS.join(", ")
            ));
        }
        Ok(config)
    }

    /// The configured workspaces root, if one was set.
    pub fn workspaces_root(&self) -> Option<PathBuf> {
        let root = self.workspaces_root.trim();
        (!root.is_empty()).then(|| PathBuf::from(root))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_file_gives_spec_defaults() {
        let config = Config::parse("").expect("parse");
        assert_eq!(config, Config::default());
        assert_eq!(config.port, 45710);
        assert_eq!(config.courier.max_attempts, 8);
        assert_eq!(config.workspaces_root(), None);
    }

    #[test]
    fn keys_override_defaults() {
        let config = Config::parse(
            "port = 45999\nworkspaces_root = 'D:\\bots'\n[courier]\nmax_attempts = 3\n",
        )
        .expect("parse");
        assert_eq!(config.port, 45999);
        assert_eq!(config.courier.max_attempts, 3);
        assert_eq!(config.courier.lease_ms, 15_000);
        assert_eq!(config.workspaces_root(), Some(PathBuf::from("D:\\bots")));
    }

    #[test]
    fn the_account_server_defaults_to_ours_and_can_be_changed_or_turned_off() {
        let url = |text: &str| Config::parse(text).expect("parse").cloud.url;
        assert_eq!(url(""), "https://botloft.comitium.com.br");
        assert_eq!(url("[cloud]\npoll_ms = 500"), DEFAULT_CLOUD_URL);
        assert_eq!(
            url("[cloud]\nurl = 'https://nuvem.exemplo.org'"),
            "https://nuvem.exemplo.org"
        );
        assert_eq!(url("[cloud]\nurl = ''"), "");
    }

    #[test]
    fn rejects_unknown_keys_and_bad_values() {
        assert!(Config::parse("prot = 1").is_err());
        assert!(Config::parse("[courier]\nmax_attempt = 1").is_err());
        assert!(Config::parse("port = 0").is_err());
        assert!(Config::parse("log_level = 'loud'").is_err());
        assert!(Config::parse("[browser]\nmax_open = 0").is_err());
    }

    #[test]
    fn missing_file_is_fine_unless_required() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("config.toml");
        assert_eq!(
            Config::load(&path, false).expect("defaults"),
            Config::default()
        );
        assert!(matches!(
            Config::load(&path, true),
            Err(ConfigError::Read { .. })
        ));
    }
}
