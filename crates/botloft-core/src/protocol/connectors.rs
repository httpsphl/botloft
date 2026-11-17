//! The owner's claude.ai connectors in the agents (spec 34): the ones the
//! account has and the ones each agent may use.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

text_enum!(
    /// How a connector stands, as Claude Code reports it.
    ConnectorState, "connector state" {
        Connected => "connected",
        /// The owner has to sign in again on claude.ai.
        NeedsAuth => "needs_auth",
        Pending => "pending",
        Failed => "failed",
    }
);

/// A connector of the owner's claude.ai account (spec 34.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Connector {
    /// Its plain name, without "claude.ai ": "Google Calendar".
    pub name: String,
    pub state: ConnectorState,
}

/// What `connectors.list` answers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ConnectorList {
    pub connectors: Vec<Connector>,
    /// When they were looked for, in Unix milliseconds.
    pub checked_at: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct ConnectorsListParams {
    /// Look again even when they were found before.
    #[serde(default)]
    pub refresh: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotsSetConnectorsParams {
    pub bot_id: BotId,
    /// Plain names, as `connectors.list` gives them.
    pub names: Vec<String>,
}
