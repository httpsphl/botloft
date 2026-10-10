//! What the RPC methods do, independent of JSON-RPC. Each operation runs
//! under the store lock, so checks and writes cannot interleave.

pub mod archive;
pub mod attachments;
pub mod autobackup;
pub mod backup;
pub mod bots;
pub mod catalog;
pub mod chat;
pub mod cloud;
pub mod crew_access;
pub mod crews;
pub mod delete;
pub mod deliveries;
pub mod desktop;
pub mod files;
pub mod lead;
pub mod mcp;
pub mod mcp_state;
pub mod messages;
pub mod mobile;
pub mod models;
pub mod modes;
pub mod plan;
pub mod questions;
pub mod reactions;
pub mod routines;
pub mod rules;
pub mod screens;
pub mod settings;
pub mod tasks;
pub mod usage;

use botloft_core::protocol::{
    AgentCheck, AgentKind, OwnerAccount, PROTOCOL_VERSION, SystemStatus, error_code,
};
use botloft_core::slug;
use botloft_core::validate::ValidationError;
use botloft_store::StoreError;

use crate::state::Daemon;

/// A failed operation, mapped to the error codes of spec 11.4.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Conflict(String),
    #[error(transparent)]
    Validation(#[from] ValidationError),
    /// A validation the app words itself, by `reason` (spec 20.8).
    #[error("{message}")]
    Rule {
        reason: &'static str,
        message: String,
    },
    #[error("could not update the workspace: {0}")]
    Workspace(#[source] std::io::Error),
    #[error("could not save the settings: {0}")]
    Settings(String),
    #[error("internal error")]
    Internal(#[source] StoreError),
}

impl ApiError {
    pub fn code(&self) -> i32 {
        match self {
            Self::NotFound(_) => error_code::NOT_FOUND,
            Self::Conflict(_) => error_code::CONFLICT,
            Self::Validation(_) | Self::Rule { .. } => error_code::VALIDATION,
            Self::Workspace(_) | Self::Settings(_) | Self::Internal(_) => {
                error_code::INTERNAL_ERROR
            }
        }
    }

    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation(ValidationError(message.into()))
    }
}

impl From<StoreError> for ApiError {
    fn from(err: StoreError) -> Self {
        match err {
            // Checks run before every insert, so this only happens on a race.
            StoreError::Duplicate(what) => Self::Conflict(format!("{what} is already in use")),
            other => Self::Internal(other),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

/// The owner's name, asked of Windows once: it does not change while the
/// daemon runs.
fn owner_name() -> String {
    static NAME: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    NAME.get_or_init(crate::platform::owner_name).clone()
}

pub fn status(daemon: &Daemon) -> ApiResult<SystemStatus> {
    let deliveries = daemon.store().delivery_backlog()?;
    Ok(SystemStatus {
        daemon_version: env!("CARGO_PKG_VERSION").to_owned(),
        protocol: PROTOCOL_VERSION,
        uptime_ms: daemon.uptime_ms(),
        claude_version: daemon.supervisor.claude_version(),
        runtime_error: daemon.supervisor.runtime_error(),
        claude_path: daemon
            .supervisor
            .claude_path()
            .map(|path| path.display().to_string()),
        claude_signed_in: daemon.supervisor.claude_signed_in(),
        account: OwnerAccount {
            name: owner_name(),
            claude: daemon.supervisor.claude_account(),
        },
        deliveries,
        usage: daemon.usage(),
        enabled_agents: [AgentKind::Claude, AgentKind::Agy, AgentKind::Codex]
            .into_iter()
            .filter(|kind| daemon.supervisor.agent_enabled(*kind))
            .collect(),
        agent_checks: agent_checks(daemon),
    })
}

/// Whether the other agents the owner enabled are installed. Asking runs the
/// program, so the answer is kept for a minute.
fn agent_checks(daemon: &Daemon) -> Vec<AgentCheck> {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};
    static CACHE: Mutex<Option<(Instant, Vec<AgentCheck>)>> = Mutex::new(None);
    let mut cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((at, checks)) = cache.as_ref()
        && at.elapsed() < Duration::from_secs(60)
    {
        return checks.clone();
    }
    let mut checks = Vec::new();
    if daemon.supervisor.agent_enabled(AgentKind::Agy)
        && let Some(agent) = crate::agent::of(AgentKind::Agy)
    {
        let version = agent
            .locate(daemon.supervisor.agy_path(), None)
            .ok()
            .and_then(|program| crate::agent::program_version(&program));
        checks.push(AgentCheck {
            agent: AgentKind::Agy,
            version,
        });
    }
    *cache = Some((Instant::now(), checks.clone()));
    checks
}

/// [`slug::unique`] for checks that can fail.
fn pick_slug(base: &str, mut taken: impl FnMut(&str) -> ApiResult<bool>) -> ApiResult<String> {
    let mut failure = None;
    let slug = slug::unique(base, |candidate| match taken(candidate) {
        Ok(taken) => taken,
        Err(err) => {
            failure.get_or_insert(err);
            false
        }
    });
    match failure {
        Some(err) => Err(err),
        None => Ok(slug),
    }
}
