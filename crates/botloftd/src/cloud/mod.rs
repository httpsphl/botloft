//! The account and the copies in the cloud (spec 27.5). Only the daemon talks
//! to the server: the app never sees the token. Nothing here runs until the
//! owner asks, and nothing of it goes in the log but the kind of failure.

mod client;
mod creds;
pub(crate) mod signin;
mod transfer;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::task::AbortHandle;

pub use client::{Me, Server};
pub use creds::Credentials;
pub use transfer::{Progress, download, upload};

use crate::config::Config;

/// What `[cloud]` in `config.toml` sets (spec 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudSettings {
    /// The server; empty until there is one.
    pub url: String,
    /// How often a waiting sign-in asks whether the link was opened.
    pub poll: Duration,
}

impl Default for CloudSettings {
    fn default() -> Self {
        Self {
            url: String::new(),
            poll: Duration::from_secs(2),
        }
    }
}

impl CloudSettings {
    pub fn from_config(config: &Config) -> Self {
        Self {
            url: config.cloud.url.trim().to_owned(),
            poll: Duration::from_millis(config.cloud.poll_ms),
        }
    }
}

/// A problem with the cloud, with a `reason` the app words (spec 20.8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CloudError {
    pub reason: &'static str,
    pub message: String,
}

impl CloudError {
    pub(crate) fn no_server(message: &str) -> Self {
        Self {
            reason: "no_server",
            message: message.to_owned(),
        }
    }

    pub(crate) fn offline() -> Self {
        Self {
            reason: "offline",
            message: "the server could not be reached".to_owned(),
        }
    }

    pub(crate) fn not_signed_in() -> Self {
        Self {
            reason: "not_signed_in",
            message: "sign in to the account first".to_owned(),
        }
    }

    pub(crate) fn other(message: String) -> Self {
        Self {
            reason: "cloud_error",
            message,
        }
    }

    /// The server's own `reason`; one this build does not know stays generic.
    pub(crate) fn from_server(reason: &str, message: String) -> Self {
        const KNOWN: [&str; 11] = [
            "unauthorized",
            "not_found",
            "bad_email",
            "rate_limited",
            "mail_failed",
            "length_required",
            "bad_length",
            "bad_hash",
            "too_big",
            "quota",
            "bad_range",
        ];
        Self {
            reason: KNOWN
                .iter()
                .find(|known| **known == reason)
                .copied()
                .unwrap_or("cloud_error"),
            message,
        }
    }
}

impl std::fmt::Display for CloudError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

/// The daemon's side of the cloud.
pub struct Cloud {
    pub(crate) settings: CloudSettings,
    pending: Arc<Mutex<Option<AbortHandle>>>,
}

impl Cloud {
    pub fn new(settings: CloudSettings) -> Self {
        Self {
            settings,
            pending: Arc::default(),
        }
    }

    pub fn url(&self) -> &str {
        &self.settings.url
    }

    /// The server, if there is one set up.
    pub fn server(&self) -> Result<Server, CloudError> {
        Server::new(&self.settings.url)
    }

    /// What this computer holds of the account, if it is for this server.
    pub fn credentials(&self, secrets_dir: &Path) -> Option<Credentials> {
        creds::load(secrets_dir).filter(|credentials| credentials.url == self.settings.url)
    }

    pub fn forget(&self, secrets_dir: &Path) {
        creds::remove(secrets_dir);
    }

    /// A sign-in waits for the link.
    pub fn is_pending(&self) -> bool {
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_some()
    }

    pub fn cancel_signin(&self) {
        if let Some(handle) = self
            .pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            handle.abort();
        }
    }
}
