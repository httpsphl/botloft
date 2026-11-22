//! Devices that reach the daemon with a token of their own, such as the app
//! on another computer through a tunnel (spec 42.6).

use serde::{Deserialize, Serialize};

use crate::ids::DeviceId;

/// A device the owner paired.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct Device {
    pub id: DeviceId,
    pub name: String,
    /// Unix time in milliseconds.
    pub paired_at: i64,
    /// When the device last connected, in Unix milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(test, ts(optional))]
    pub last_seen_at: Option<i64>,
}

/// `devices.list`, `devices.revoke` and the `devices.changed` notification:
/// every paired device, the newest first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DeviceList {
    pub devices: Vec<Device>,
}

/// `devices.pair`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DevicesPairParams {
    pub name: String,
}

/// What `devices.pair` answers. The token is shown this once: the daemon
/// keeps only its SHA-256.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DevicePaired {
    pub device: Device,
    pub token: String,
}

impl std::fmt::Debug for DevicePaired {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DevicePaired")
            .field("device", &self.device)
            .field("token", &"<redacted>")
            .finish()
    }
}

/// `devices.revoke`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, derive(ts_rs::TS))]
pub struct DeviceIdParams {
    pub device_id: DeviceId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_token_stays_out_of_debug_output() {
        let paired = DevicePaired {
            device: Device {
                id: DeviceId::generate(),
                name: "Laptop".to_owned(),
                paired_at: 1,
                last_seen_at: None,
            },
            token: "secret-token".to_owned(),
        };
        assert!(!format!("{paired:?}").contains("secret-token"));
    }
}
