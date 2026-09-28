//! Supervisor configuration, from `config.toml` or set by tests.

use std::time::Duration;

use crate::config::Config;
use crate::runtime::claude::Claude;

#[derive(Debug, Clone)]
pub enum ClaudeSource {
    /// `claude_path` from the config, or `PATH`; checked with `--version`.
    Discover { configured: String },
    /// A known executable, used without running it (tests).
    Fixed(Claude),
}

#[derive(Debug, Clone)]
pub struct SupervisorSettings {
    pub claude: ClaudeSource,
    pub backoff_initial: Duration,
    pub backoff_max: Duration,
    pub fresh_start_if_dies_within: Duration,
    pub ring_buffer_bytes: usize,
}

impl SupervisorSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            claude: ClaudeSource::Discover {
                configured: config.claude_path.clone(),
            },
            backoff_initial: Duration::from_millis(config.supervisor.restart_backoff_initial_ms),
            backoff_max: Duration::from_millis(config.supervisor.restart_backoff_max_ms),
            fresh_start_if_dies_within: Duration::from_secs(
                config.supervisor.fresh_start_if_dies_within_s,
            ),
            ring_buffer_bytes: config.terminal.ring_buffer_bytes,
        }
    }
}
