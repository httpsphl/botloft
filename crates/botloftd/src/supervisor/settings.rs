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
    /// `agy_path` from the config; empty looks in the usual places (spec 30).
    pub agy_path: String,
    /// `codex_path` from the config; empty looks in the usual places.
    pub codex_path: String,
    /// Agents the owner switched on although they are experimental.
    pub experimental_agents: Vec<String>,
    pub backoff_initial: Duration,
    pub backoff_max: Duration,
    pub fresh_start_if_dies_within: Duration,
    /// A new process is taken as ready after living this long; Claude Code
    /// says nothing until the first message (spec 7.2).
    pub ready_after: Duration,
}

impl SupervisorSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            claude: ClaudeSource::Discover {
                configured: config.claude_path.clone(),
            },
            agy_path: config.agy_path.clone(),
            codex_path: config.codex_path.clone(),
            experimental_agents: config.experimental_agents.clone(),
            backoff_initial: Duration::from_millis(config.supervisor.restart_backoff_initial_ms),
            backoff_max: Duration::from_millis(config.supervisor.restart_backoff_max_ms),
            fresh_start_if_dies_within: Duration::from_secs(
                config.supervisor.fresh_start_if_dies_within_s,
            ),
            ready_after: Duration::from_millis(1500),
        }
    }
}
