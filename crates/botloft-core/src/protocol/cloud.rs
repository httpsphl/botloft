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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub email: Option<String>,
    /// Bytes the account's copies take, when the server answered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub used: Option<u64>,
    /// Bytes the account may keep, when the server answered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
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

/// How often the automatic backup looks for something new to send (spec 27.10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub enum AutoBackupEvery {
    #[default]
    Daily,
    Weekly,
}

/// `autobackup.status` and the `autobackup.changed` notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AutoBackupStatus {
    /// This computer can keep the passphrase safely (Windows today).
    pub available: bool,
    pub enabled: bool,
    pub every: AutoBackupEvery,
    /// Unix time in milliseconds of the last copy it sent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub last_ok_at: Option<i64>,
    /// Unix time in milliseconds of the next look; absent when it is off.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub next_at: Option<i64>,
    /// Why the last try failed (a `reason` of 27.5, or `no_passphrase`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub last_error: Option<String>,
}

/// `autobackup.enable`: the passphrase is kept in the system's credential
/// store, and never leaves this computer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AutoBackupEnableParams {
    pub passphrase: String,
    pub every: AutoBackupEvery,
}

/// `autobackup.set_every`
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct AutoBackupEveryParams {
    pub every: AutoBackupEvery,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn what_is_absent_is_left_out_of_the_json_not_sent_as_null() {
        let off = AutoBackupStatus {
            available: true,
            enabled: false,
            every: AutoBackupEvery::Daily,
            last_ok_at: None,
            next_at: None,
            last_error: None,
        };
        let json = serde_json::to_value(&off).expect("json");
        assert_eq!(
            json,
            serde_json::json!({ "available": true, "enabled": false, "every": "daily" })
        );
        let out = CloudStatus {
            url: "u".to_owned(),
            signed_in: false,
            email: None,
            used: None,
            quota: None,
            pending: false,
        };
        let json = serde_json::to_value(&out).expect("json");
        assert!(json.get("email").is_none() && json.get("used").is_none());
    }
}
