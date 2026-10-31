//! Everything the owner let each bot do, in one place (spec 15.1): the
//! apps on the desktop (24.10), the sites in its browser (21.5), what it
//! does without asking (10.1) and the other crews it reaches (10.4).

use serde::{Deserialize, Serialize};

use super::{AllowRule, CrewAccess, DesktopGrant};
use crate::ids::BotId;

/// The result of `permissions.list`: every bot's, oldest first in each list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Permissions {
    pub desktop: Vec<DesktopGrant>,
    pub sites: Vec<BrowserSite>,
    pub rules: Vec<AllowRule>,
    pub crew_access: Vec<CrewAccess>,
}

/// A site the owner let a bot use in its browser (spec 21.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BrowserSite {
    pub bot_id: BotId,
    /// The host in lowercase, without `www.`; it covers its subdomains.
    pub site: String,
    /// Unix time in milliseconds.
    pub allowed_at: i64,
}

/// A bot's sites after one was taken away: the result of `sites.revoke`
/// and the params of the `bot.sites` notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct BotSites {
    pub bot_id: BotId,
    /// Oldest first.
    pub sites: Vec<BrowserSite>,
}

/// `sites.revoke`: the bot asks for the site again the next time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct SitesRevokeParams {
    pub bot_id: BotId,
    pub site: String,
}
