//! `mobile.*` (spec 28.5). Those that ask the server wait for the network, so
//! the connection answers them aside (`rpc::ASIDE`).

use std::future::Future;

use botloft_core::protocol::{
    MobilePairConfirmParams, MobilePairIdParams, MobilePairStarted, MobileRevokeParams,
    MobileStatus,
};

use super::cloud::fail;
use super::{ApiError, ApiResult};
use crate::mobile::Confirm;
use crate::state::Daemon;

/// The service runs on the blocking pool, where the network waits belong.
fn run<T>(work: impl Future<Output = T>) -> T {
    tokio::runtime::Handle::current().block_on(work)
}

/// `mobile.status`: nothing is asked of the server.
pub fn status(daemon: &Daemon) -> MobileStatus {
    daemon.mobile.status(daemon)
}

/// `mobile.pair_start`: a code for a phone to scan.
pub fn pair_start(daemon: &Daemon) -> ApiResult<MobilePairStarted> {
    run(daemon.mobile.pair_start(daemon)).map_err(|err| fail(daemon, err))
}

/// `mobile.pair_cancel`
pub fn pair_cancel(daemon: &Daemon, params: MobilePairIdParams) -> ApiResult<()> {
    run(daemon.mobile.pair_cancel(daemon, &params.pair_id)).map_err(|err| fail(daemon, err))
}

/// `mobile.pair_confirm`: the owner compared the codes.
pub fn pair_confirm(daemon: &Daemon, params: MobilePairConfirmParams) -> ApiResult<()> {
    run(daemon
        .mobile
        .pair_confirm(daemon, &params.pair_id, params.accept))
    .map_err(|err| match err {
        Confirm::NothingToConfirm => ApiError::Conflict(
            "there is no phone waiting on that code; it may have run out".to_owned(),
        ),
        Confirm::Cloud(err) => fail(daemon, err),
    })
}

/// `mobile.revoke`: the phone loses its access at once.
pub fn revoke(daemon: &Daemon, params: MobileRevokeParams) -> ApiResult<MobileStatus> {
    let removed =
        run(daemon.mobile.revoke(daemon, &params.phone_id)).map_err(|err| fail(daemon, err))?;
    if !removed {
        return Err(ApiError::NotFound(format!("phone {}", params.phone_id)));
    }
    Ok(daemon.mobile.status(daemon))
}
