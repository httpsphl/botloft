//! Shared site logins (spec 35): the owner signs in to a site once, in a
//! Botloft window, and chooses which agents use that login.

use serde::{Deserialize, Serialize};

use crate::ids::BotId;

/// A site the owner signed in to, and the agents that have the login.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SiteLogin {
    /// The host in lowercase without `www.`, as in spec 21.5: "reddit.com".
    pub site: String,
    /// When the owner last signed in, in Unix milliseconds.
    pub signed_at: i64,
    pub bot_ids: Vec<BotId>,
}

/// What `logins.list` answers, and `logins.changed` brings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LoginList {
    pub logins: Vec<SiteLogin>,
    /// The site of the window open for the owner to sign in, if any.
    pub window: Option<String>,
}

/// `logins.changed`: the list after a change. When a window closed,
/// `closed` names its site and `found` says whether a login was there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LoginsChanged {
    pub list: LoginList,
    pub closed: Option<String>,
    pub found: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LoginsOpenParams {
    /// The page to sign in on: "reddit.com" or "https://www.reddit.com/login".
    pub address: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LoginsOpenResult {
    pub site: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LoginsShareParams {
    pub site: String,
    /// Every agent that should have the login; the others lose it.
    pub bot_ids: Vec<BotId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct LoginsSiteParams {
    pub site: String,
}
