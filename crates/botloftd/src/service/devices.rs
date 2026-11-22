//! `devices.*` (spec 42.6): what reaches this daemon with a token of its
//! own, such as the app on another computer through a tunnel.

use botloft_core::protocol::{DeviceIdParams, DeviceList, DevicePaired, DevicesPairParams};
use botloft_core::validate;
use tracing::info;

use super::{ApiError, ApiResult};
use crate::secrets::{self, TokenHash};
use crate::state::{Daemon, Event};

/// `devices.list`
pub fn list(daemon: &Daemon) -> ApiResult<DeviceList> {
    Ok(DeviceList {
        devices: daemon.store().devices()?,
    })
}

/// `devices.pair`: a new device and its token, which only this answer
/// carries. The connection lets only the owner's token ask (`rpc`).
pub fn pair(daemon: &Daemon, params: DevicesPairParams) -> ApiResult<DevicePaired> {
    let name = validate::name("name", &params.name)?;
    let token = secrets::random_token().map_err(ApiError::Token)?;
    let hash = TokenHash::of(&token).to_hex();
    let device = daemon
        .store()
        .insert_device(&name, &hash, daemon.clock.now_ms())?;
    info!("a new device was paired");
    changed(daemon)?;
    Ok(DevicePaired { device, token })
}

/// `devices.revoke`: the token stops working and the device's connections
/// close.
pub fn revoke(daemon: &Daemon, params: DeviceIdParams) -> ApiResult<DeviceList> {
    if !daemon.store().delete_device(&params.device_id)? {
        return Err(ApiError::NotFound(format!("device {}", params.device_id)));
    }
    info!("the owner revoked a device");
    daemon.emit(Event::DeviceRevoked(params.device_id));
    changed(daemon)
}

/// Tells the apps the devices now.
fn changed(daemon: &Daemon) -> ApiResult<DeviceList> {
    let list = list(daemon)?;
    daemon.emit(Event::DevicesChanged(list.clone()));
    Ok(list)
}
