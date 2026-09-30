//! What the RPC methods do, independent of JSON-RPC. Each operation runs
//! under the store lock, so checks and writes cannot interleave.

pub mod archive;
pub mod attachments;
pub mod bots;
pub mod chat;
pub mod crews;
pub mod delete;
pub mod deliveries;
pub mod files;
pub mod lead;
pub mod messages;
pub mod models;
pub mod modes;
pub mod routines;
pub mod screens;
pub mod settings;
pub mod tasks;

use botloft_core::protocol::{OwnerAccount, PROTOCOL_VERSION, SystemStatus, error_code};
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
    })
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
