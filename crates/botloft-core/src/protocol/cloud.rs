//! The account and the copies in the cloud (spec 27.5).

use serde::{Deserialize, Serialize};

/// `cloud.status`: where the owner stands with the account.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudStatus {
    /// The server the app talks to; empty when none is set up.
    pub url: String,
    pub signed_in: bool,
    #[cfg_attr(test, ts(optional))]
    pub email: Option<String>,
    /// Bytes the account's copies take, when the server answered.
    #[cfg_attr(test, ts(optional))]
    pub used: Option<u64>,
    /// Bytes the account may keep, when the server answered.
    #[cfg_attr(test, ts(optional))]
    pub quota: Option<u64>,
    /// A sign-in waits for the owner to open the link in the e-mail.
    pub pending: bool,
}

/// `cloud.signin`: the e-mail the link goes to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudSigninParams {
    pub email: String,
    /// The app's language (`en`, `pt-BR`, `es`), for the e-mail.
    #[serde(default)]
    #[cfg_attr(test, ts(optional))]
    pub locale: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudSigninStarted {
    /// Seconds the link in the e-mail works.
    pub wait: u32,
}

/// `cloud.upload`: the passphrase seals the copy, and never leaves here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudUploadParams {
    pub passphrase: String,
}

/// A copy the account keeps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudCopy {
    pub id: String,
    /// Bytes.
    pub size: u64,
    /// Unix time in milliseconds.
    pub created: i64,
}

/// `cloud.copies`, newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudCopies {
    pub copies: Vec<CloudCopy>,
}

/// `cloud.download` and `cloud.delete`: which copy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudCopyParams {
    pub id: String,
}

/// A copy fetched, where the app goes on with `backup.stage`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudDownloaded {
    pub path: String,
}

/// `cloud.signed_in`: the owner opened the link and this computer is in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudSignedIn {
    pub email: String,
}

/// Which way the bytes of a [`CloudProgress`] go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum CloudDirection {
    Upload,
    Download,
}

/// `cloud.progress`: how much of a copy went up or came down.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct CloudProgress {
    pub direction: CloudDirection,
    pub sent: u64,
    pub total: u64,
}
