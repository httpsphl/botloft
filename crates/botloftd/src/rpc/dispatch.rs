//! Routes an authenticated request to its service operation.

use botloft_core::protocol::{error_code, method};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

use super::jsonrpc::{RpcError, empty_params};
use crate::service::{self, ApiResult, bots, crews, deliveries, messages, terminal};
use crate::state::Daemon;

pub fn dispatch(daemon: &Daemon, name: &str, params: Option<Value>) -> Result<Value, RpcError> {
    match name {
        method::SYSTEM_STATUS => reply(service::status(daemon)),
        method::CREWS_LIST => reply(crews::list(daemon)),
        method::CREWS_CREATE => reply(crews::create(daemon, parse(params)?)),
        method::CREWS_RENAME => reply(crews::rename(daemon, parse(params)?)),
        method::CREWS_SET_PAUSED => reply(crews::set_paused(daemon, parse(params)?)),
        method::CREWS_ARCHIVE => reply(crews::archive(daemon, parse(params)?)),
        method::BOTS_LIST => reply(bots::list(daemon, parse(params)?)),
        method::BOTS_CREATE => reply(bots::create(daemon, parse(params)?)),
        method::BOTS_UPDATE => reply(bots::update(daemon, parse(params)?)),
        method::BOTS_SET_PAUSED => reply(bots::set_paused(daemon, parse(params)?)),
        method::BOTS_ARCHIVE => reply(bots::archive(daemon, parse(params)?)),
        method::BOTS_RESTART => reply(bots::restart(daemon, parse(params)?)),
        method::TERMINAL_WRITE => reply(terminal::write(daemon, parse(params)?)),
        method::TERMINAL_RESIZE => reply(terminal::resize(daemon, parse(params)?)),
        method::MESSAGES_SEND => reply(messages::send(daemon, parse(params)?)),
        method::MESSAGES_LIST => reply(messages::list(daemon, parse(params)?)),
        method::DELIVERIES_LIST => reply(deliveries::list(daemon, parse(params)?)),
        method::DELIVERIES_RETRY => reply(deliveries::retry(daemon, parse(params)?)),
        method::SESSION_HELLO => Err(RpcError::new(
            error_code::CONFLICT,
            "this connection is already authenticated",
        )),
        other => Err(RpcError::new(
            error_code::METHOD_NOT_FOUND,
            format!("unknown method {other}"),
        )),
    }
}

pub(super) fn parse<T: DeserializeOwned>(params: Option<Value>) -> Result<T, RpcError> {
    serde_json::from_value(params.unwrap_or_else(empty_params))
        .map_err(|err| RpcError::new(error_code::INVALID_PARAMS, format!("invalid params: {err}")))
}

fn reply<T: Serialize>(result: ApiResult<T>) -> Result<Value, RpcError> {
    let value = result.map_err(RpcError::from)?;
    serde_json::to_value(value)
        .map_err(|err| RpcError::new(error_code::INTERNAL_ERROR, err.to_string()))
}
