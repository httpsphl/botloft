//! A connection watching one bot's browser (spec 21.7): `browser.watch`
//! and `browser.unwatch`, and the frames that follow.

use botloft_core::protocol::{BrowserWatchParams, method, notification};
use serde_json::Value;

use super::dispatch::parse;
use super::jsonrpc::{self, RpcError};
use crate::browser::Watching;
use crate::service::bots;
use crate::state::Daemon;

#[derive(Default)]
pub(super) struct Watch {
    current: Option<Watching>,
}

impl Watch {
    pub(super) fn request(
        &mut self,
        daemon: &Daemon,
        name: &str,
        params: Option<Value>,
    ) -> Result<Value, RpcError> {
        // Stop the one before, even if the new one is refused.
        self.current = None;
        if name == method::BROWSER_UNWATCH {
            return Ok(Value::Null);
        }
        let params: BrowserWatchParams = parse(params)?;
        bots::find(&daemon.store(), &params.bot_id).map_err(RpcError::from)?;
        self.current = Some(daemon.browsers.watch(&params.bot_id));
        serde_json::to_value(daemon.browsers.view(&params.bot_id)).map_err(|err| {
            RpcError::new(
                botloft_core::protocol::error_code::INTERNAL_ERROR,
                err.to_string(),
            )
        })
    }

    /// The watched browser's next frame, as a notification. Never resolves
    /// while nothing is watched.
    pub(super) async fn next_frame(&mut self) -> Option<String> {
        let Some(watching) = self.current.as_mut() else {
            return std::future::pending().await;
        };
        if watching.frames.changed().await.is_err() {
            self.current = None;
            return None;
        }
        let frame = watching.frames.borrow_and_update().clone()?;
        let params = serde_json::to_value(&*frame).unwrap_or(Value::Null);
        Some(jsonrpc::notification(notification::BROWSER_FRAME, params))
    }
}
