//! `deliveries.*` operations.

use botloft_core::protocol::{DeliveriesListParams, Delivery, DeliveryIdParams};

use super::{ApiError, ApiResult, bots};
use crate::state::{Daemon, Event};

pub fn list(daemon: &Daemon, params: DeliveriesListParams) -> ApiResult<Vec<Delivery>> {
    let store = daemon.store();
    if let Some(bot) = &params.bot_id {
        bots::find(&store, bot)?;
    }
    Ok(store.deliveries(params.state, params.bot_id.as_ref())?)
}

/// Puts a delivery that gave up back in the queue (spec 9.1).
pub fn retry(daemon: &Daemon, params: DeliveryIdParams) -> ApiResult<Delivery> {
    let store = daemon.store();
    let id = &params.delivery_id;
    let Some(current) = store.delivery(id)? else {
        return Err(ApiError::NotFound(format!("delivery {id} does not exist")));
    };
    let Some(retried) = store.retry_delivery(id, daemon.clock.now_ms())? else {
        return Err(ApiError::Conflict(format!(
            "delivery {id} is {}; only dead deliveries can be retried",
            current.state
        )));
    };
    daemon.emit(Event::DeliveryChanged(retried.clone()));
    daemon.courier.wake();
    Ok(retried)
}
